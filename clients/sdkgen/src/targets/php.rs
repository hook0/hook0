//! Emits the generated half of the PHP SDK.
//!
//! The package is published with no copy of the OpenAPI snapshot beside it, so the types, the
//! problems and the request layer travel as committed source rather than as a build artefact.
//! Everything the API declares — one class per named schema, one backed enumeration per closed list
//! of strings, one exception per problem the error contract can report, one method per operation —
//! is written here; everything the API does not declare — how a request reaches the network, how a
//! send is retried, how a webhook signature is verified — is hand-written beside this directory and
//! never regenerated.
//!
//! The two halves meet at two seams and nowhere else. The generated code reads its decoding helpers
//! from the hand-written runtime beside it, and it issues its requests through the transport it is
//! handed: nothing here knows what a socket is, and nothing beside it knows what the API declares.
//!
//! One declaration per file, named after what it declares, because that is the arrangement an
//! autoloader resolves a class through: a caller reaches `Hook0\Generated\Application` and the
//! loader opens `Application.php` without an index having to name it. A type the document stopped
//! declaring therefore takes its file with it rather than lingering in a shared one.
//!
//! What is written is already linted. An emitted operation holds no local variable of its own — the
//! path, the query and the answer are all expressions — and PHP reads what follows `function`, `->`
//! and `::` as a name rather than as a keyword, so the vocabulary a name is spelled out of the way
//! of is narrow and stated per kind rather than applied everywhere.
//!
//! An emitted reader is the one exception, and it is deliberate: it gathers what it read into a
//! local and spreads that into the constructor, rather than reading each member inside the
//! constructor call. Both spellings say the same thing, and only one of them survives a member that
//! does not read. A callable built inside the argument list of a call that is itself an argument of
//! a `new` leaves the language unwinding a half-built value when that call throws, which is a
//! segmentation fault on releases in service today — reachable, for a reader, by an API that answers
//! a document missing a member it declares. The three names such a method holds — the document, its
//! members, and what was read out of them — are the emitter's own and are never derived from what
//! the API declares, so nothing the document carries can be assigned over.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, two arguments of one method spelled alike, a
//! scalar no annotation covers — stops the emission rather than yielding a smaller SDK.

use std::collections::BTreeMap;

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, spell};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Scalar, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{Contract, LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "php";

/// Where the generated half of the package lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written. The
/// autoloader maps the root namespace onto `src`, so the generated tree is the one directory below
/// it that carries the generated namespace.
const ROOT: &str = "clients/php/src/Generated";

/// Namespace the package is reached under.
const ROOT_NAMESPACE: &str = "Hook0";

/// Namespace everything written here lands in, under the package's own.
const GENERATED_NAMESPACE: &str = "Generated";

/// Class the hand-written decoding helpers are reached through.
const RUNTIME_CLASS: &str = "Runtime";

/// Class an emitted operation group issues its requests through.
const TRANSPORT_CLASS: &str = "Transport";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix an exception carries, so a problem and a type spelling the same word stay apart.
const EXCEPTION_SUFFIX: &str = "Error";

/// Constant naming the exception each problem is raised as.
const PROBLEMS_CONSTANT: &str = "PROBLEMS";

/// What an emitted group calls the helper that raises what the API reported and hands back nothing.
const CHECK_HELPER: &str = "checkAnswer";

/// What an emitted group calls the helper that raises what the API reported and reads what it
/// answered.
const READ_HELPER: &str = "readAnswer";

/// What an emitted type calls the constructor reading one out of what the API answered.
const READ_METHOD: &str = "fromJson";

/// What an emitted type calls the method writing one back the way the API reads it.
const WRITE_METHOD: &str = "toArray";

/// What an emitted type calls the comparison two values of it are held against each other with.
const EQUALITY_METHOD: &str = "equals";

/// What the base exception calls the helper turning an answer into the problem it names.
const RAISE_HELPER: &str = "raiseForStatus";

/// What the base exception calls the helper reading a problem out of a body.
const PROBLEM_HELPER: &str = "problemOf";

/// What the argument carrying the body an operation reads is called.
const BODY_ARGUMENT: &str = "body";

/// What an emitted reader gathers the members it read into before it builds the value.
///
/// It is one of the three names an emitted method holds — the document it was handed, the members
/// of that document, and this — and none of the three is ever derived from what the API declares,
/// so no name the document carries can be assigned over.
const READ_LOCAL: &str = "read";

/// Longest fragment of a snapshot description a comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Longest line the emitted source carries.
///
/// A description the document writes as one paragraph is folded across as many comment lines as it
/// takes, and an argument list that would not fit is written one argument per line. Both are what
/// the linter asks for, and neither is something a pass over the emitted source should have to work
/// out afterwards.
const MAX_LINE_CHARS: usize = 120;

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// What every declaration of the package sits at, and every step further in.
const MEMBER_INDENT: &str = "    ";
const BODY_INDENT: &str = "        ";
const ARGUMENT_INDENT: &str = "            ";

/// Failure the hand-written runtime reports when what the API answered is not what it declares.
const DECODE_ERROR_CLASS: &str = "DecodeError";

/// The names the scaffolding around the emitted code answers to, which no type of the document may
/// answer to as well.
///
/// Each one is a hand-written class an emitted file imports by name, and an import naming a class
/// the same file declares is refused by the language outright. Claiming them alongside everything
/// the model names reports the collision as one rather than emitting a file that cannot be loaded.
/// Sorted, as they are claimed in order.
const SCAFFOLDING: [&str; 3] = [DECODE_ERROR_CLASS, RUNTIME_CLASS, TRANSPORT_CLASS];

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan the autoloader would still resolve.
        ownership: Ownership::Directory,
        contract: Contract::Whole,
        language: super::php(),
        emit,
    }
}

/// Everything the generated half of the package is made of.
fn emit(language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;
    let banner = banner(language.comment, &update_command(NAME), &limits)?;

    let enums = model.enumerations(&limits)?;
    let types = Types::read(model, &enums, language, &limits)?;

    let mut files = Vec::new();

    for (name, values) in &enums {
        let declared = types.enumeration(name)?;
        files.push(file(
            declared,
            &banner,
            &[],
            &enumeration(declared, values, language, &limits)?,
            language,
            &limits,
        )?);
    }

    for (name, object) in &model.schemas {
        let declared = types.schema(name)?;
        files.push(file(
            declared,
            &banner,
            &[RUNTIME_CLASS],
            &structure(declared, object, &types, language, &limits)?,
            language,
            &limits,
        )?);
    }

    files.push(file(
        &types.problem_base,
        &banner,
        &[DECODE_ERROR_CLASS, RUNTIME_CLASS],
        &base_exception(model, &types, language, &limits)?,
        language,
        &limits,
    )?);

    for (value, declared) in &types.problems {
        files.push(file(
            declared,
            &banner,
            &[],
            &problem_exception(declared, value, &types.problem_base),
            language,
            &limits,
        )?);
    }

    for entity in model.entities.entities() {
        let declared = types.group(&entity.name)?;
        files.push(file(
            declared,
            &banner,
            &[RUNTIME_CLASS, TRANSPORT_CLASS],
            &group(entity, &types, language, &limits)?,
            language,
            &limits,
        )?);
    }

    FileTree::build(files, &limits)
}

/// One file: the opening tag, the banner, the strictness the package is read under, the namespace
/// it lands in, what it imports from the hand-written half, and the declaration itself.
///
/// The opening tag is the one line the language puts before everything, so the banner sits under it
/// rather than above it: the first thing someone opening the file reads is still what wrote it.
fn file(
    declared: &str,
    banner: &str,
    imported: &[&str],
    body: &str,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<EmittedFile, Error> {
    let mut imports = String::new();
    for class in imported {
        imports.push_str(&format!("use {ROOT_NAMESPACE}\\{class};\n"));
    }
    if !imports.is_empty() {
        imports.push('\n');
    }

    Ok(EmittedFile {
        // The stem is the declared name itself rather than a rendering of it: an autoloader turns
        // the class name into the file name character for character, and a name that was spelled
        // out of the way of the language's vocabulary would not survive being rendered again.
        path: RelativePath::build(&format!("{declared}.{}", language.extension), limits)?,
        contents: format!(
            "<?php\n\n{banner}\ndeclare(strict_types=1);\n\n\
             namespace {ROOT_NAMESPACE}\\{GENERATED_NAMESPACE};\n\n{imports}{body}"
        ),
    })
}

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as two files the autoloader would resolve one class through.
struct Types {
    /// Class each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Enumeration each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Exception each problem is raised as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Exception every problem is a kind of.
    problem_base: String,
    /// Enumeration the discriminant of the error contract is read through.
    problem_enum: String,
    /// Operation groups, by entity name.
    groups: BTreeMap<String, String>,
    /// What the language calls each scalar the model carries.
    scalars: ScalarNames,
}

impl Types {
    fn read(
        model: &ApiModel,
        enums: &BTreeMap<String, Vec<String>>,
        language: &LanguageSpec,
        limits: &Limits,
    ) -> Result<Self, Error> {
        let mut claimed: BTreeMap<String, String> = BTreeMap::new();
        let mut claim = |name: String, origin: &str| -> Result<String, Error> {
            if let Some(first) = claimed.get(&name) {
                return Err(Error::SchemaNameCollision {
                    name: preview(&name),
                    first: preview(first),
                    second: preview(origin),
                });
            }
            claimed.insert(name.clone(), origin.to_owned());
            Ok(name)
        };

        for declared in SCAFFOLDING {
            claim(declared.to_owned(), "the scaffolding this target writes")?;
        }

        let mut schemas = BTreeMap::new();
        for name in model.schemas.keys() {
            let declared = claim(type_name(name, language, limits)?, name)?;
            schemas.insert(name.clone(), declared);
        }

        let mut declared_enums = BTreeMap::new();
        for name in enums.keys() {
            let declared = claim(type_name(name, language, limits)?, name)?;
            declared_enums.insert(name.clone(), declared);
        }

        let problem_enum = enum_of(&model.errors, &declared_enums)?;

        let base = format!(
            "{}{EXCEPTION_SUFFIX}",
            type_name(&model.errors.schema, language, limits)?
        );
        let problem_base = claim(base, &model.errors.schema)?;

        let mut problems = BTreeMap::new();
        for value in model.errors.catalogue.values() {
            let name = format!("{}{EXCEPTION_SUFFIX}", type_name(value, language, limits)?);
            problems.insert(value.clone(), claim(name, value)?);
        }

        let mut groups = BTreeMap::new();
        for entity in model.entities.entities() {
            let stem = type_name(&entity.name, language, limits)?;
            let declared = claim(format!("{stem}{GROUP_SUFFIX}"), &entity.name)?;
            groups.insert(entity.name.clone(), declared);
        }

        Ok(Self {
            schemas,
            enums: declared_enums,
            problems,
            problem_base,
            problem_enum,
            groups,
            scalars: language.scalars,
        })
    }

    fn schema(&self, name: &str) -> Result<&str, Error> {
        self.schemas
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(name),
            })
    }

    fn enumeration(&self, name: &str) -> Result<&str, Error> {
        self.enums
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(name),
            })
    }

    fn group(&self, entity: &str) -> Result<&str, Error> {
        self.groups
            .get(entity)
            .map(String::as_str)
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(entity),
            })
    }
}

/// The closed list the discriminant of the error contract is read through.
///
/// The catalogue is the values of one closed list of strings the error schema declares, so the
/// enumeration already exists among the types: it is found rather than declared twice.
fn enum_of(errors: &ErrorModel, enums: &BTreeMap<String, String>) -> Result<String, Error> {
    let Shape::Enum { name, .. } = &discriminant_field(errors)?.shape else {
        return Err(Error::ErrorSchemaWithoutCatalogue {
            schema: preview(&errors.schema),
        });
    };

    enums
        .get(name)
        .cloned()
        .ok_or_else(|| Error::UnresolvableReference {
            reference: preview(name),
        })
}

/// The member of the error schema that tells one problem from another.
fn discriminant_field(errors: &ErrorModel) -> Result<&Field, Error> {
    errors
        .shape
        .fields
        .iter()
        .find(|field| field.name == errors.discriminant)
        .ok_or_else(|| Error::ErrorSchemaWithoutCatalogue {
            schema: preview(&errors.schema),
        })
}

/// One closed list of strings, as an enumeration backed by the values the API answers.
///
/// Backed by strings so nothing has to be mapped on the way in or on the way out, and a real
/// enumeration rather than a set of constants so a value the API does not declare cannot be written
/// where one of them is asked for.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut members: BTreeMap<String, &str> = BTreeMap::new();
    let mut cases = String::new();

    for value in values {
        let member = constant_name(value, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        cases.push_str(&format!(
            "{MEMBER_INDENT}case {member} = {};\n",
            literal(value)
        ));
    }

    Ok(format!(
        "/**\n * One of the values the API answers with.\n */\n\
         enum {declared}: string\n{{\n{cases}}}\n"
    ))
}

/// One named schema, as the value a caller reads and writes.
fn structure(
    declared: &str,
    object: &ObjectShape,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let ordered = ordered_fields(object);

    let mut named = Vec::with_capacity(ordered.len());
    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    for field in &ordered {
        let name = member_name(&field.name, language.casing.field, limits)?;
        if let Some(first) = claimed.insert(name.clone(), &field.name) {
            return Err(Error::SchemaNameCollision {
                name: preview(&name),
                first: preview(first),
                second: preview(&field.name),
            });
        }
        named.push((name, *field));
    }

    let mut source = format!(
        "/**\n * The `{}` the API declares.\n */\nfinal class {declared}\n{{\n",
        comment(&object.name)
    );

    let mut documented = String::new();
    for (name, field) in &named {
        documented.push_str(&folded(
            MEMBER_INDENT,
            &format!(
                "@param {} ${name} carries `{}`{}{}",
                documented_type(&field.shape, field.required, types)?,
                comment(&field.name),
                listed(&field.shape, types)?,
                described(field.description.as_deref())
            ),
        ));
    }
    if !documented.is_empty() {
        source.push_str(&format!(
            "{MEMBER_INDENT}/**\n{documented}{MEMBER_INDENT} */\n"
        ));
    }

    let promoted: Vec<String> = named
        .iter()
        .map(|(name, field)| {
            Ok(format!(
                "public readonly {} ${name}{}",
                native_type(&field.shape, field.required, types)?,
                if field.required { "" } else { " = null" }
            ))
        })
        .collect::<Result<Vec<String>, Error>>()?;
    source.push_str(&declaration(
        MEMBER_INDENT,
        "__construct",
        &promoted,
        "",
        true,
    ));
    source.push_str(&format!("{MEMBER_INDENT}}}\n\n"));

    source.push_str(&format!(
        "{MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * Read one out of what the API answered.\n\
         {MEMBER_INDENT} *\n\
         {MEMBER_INDENT} * Every member is read before the value is built, and the members are then\n\
         {MEMBER_INDENT} * spread into the constructor under the names it declares. A member that\n\
         {MEMBER_INDENT} * does not read stops the read where it is, rather than while a half-built\n\
         {MEMBER_INDENT} * value is on the stack.\n\
         {MEMBER_INDENT} *\n\
         {MEMBER_INDENT} * @param mixed $value the JSON document the API answered\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}public static function {READ_METHOD}(mixed $value): self\n\
         {MEMBER_INDENT}{{\n\
         {BODY_INDENT}$fields = {RUNTIME_CLASS}::asFields($value, {});\n\
         {BODY_INDENT}${READ_LOCAL} = [\n",
        literal(&object.name)
    ));
    for (name, field) in &named {
        let called = format!(
            "{} => {RUNTIME_CLASS}::{}",
            literal(name),
            if field.required { "read" } else { "maybe" }
        );
        let arguments = [
            "$fields".to_owned(),
            literal(&field.name),
            reader(&field.shape, types, 0)?,
        ];
        source.push_str(&call(ARGUMENT_INDENT, &called, &arguments, ","));
    }
    source.push_str(&format!(
        "{BODY_INDENT}];\n\n{BODY_INDENT}return new self(...${READ_LOCAL});\n{MEMBER_INDENT}}}\n\n"
    ));

    source.push_str(&format!(
        "{MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * Write one back the way the API reads it.\n\
         {MEMBER_INDENT} *\n\
         {MEMBER_INDENT} * @return array<string, mixed>\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}public function {WRITE_METHOD}(): array\n\
         {MEMBER_INDENT}{{\n\
         {BODY_INDENT}$out = [];\n"
    ));
    for (name, field) in &named {
        let written = writer(&field.shape, &format!("$this->{name}"), 0)?;
        let assigned = format!("$out[{}] = {written};", literal(&field.name));
        if field.required {
            source.push_str(&format!("{BODY_INDENT}{assigned}\n"));
            continue;
        }

        // A member the document does not require is written only when it carries something.
        source.push_str(&format!(
            "{BODY_INDENT}if ($this->{name} !== null) {{\n\
             {ARGUMENT_INDENT}{assigned}\n\
             {BODY_INDENT}}}\n"
        ));
    }
    source.push_str(&format!(
        "\n{BODY_INDENT}return $out;\n{MEMBER_INDENT}}}\n\n"
    ));

    // Held against each other as the API reads them rather than member by member: a value written
    // back is made of arrays, strings and numbers whatever the members were, which is one
    // comparison for every shape a type can carry.
    source.push_str(&format!(
        "{MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * Whether that value carries the same members as this one.\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}public function {EQUALITY_METHOD}(mixed $other): bool\n\
         {MEMBER_INDENT}{{\n\
         {BODY_INDENT}return $other instanceof self\n\
         {ARGUMENT_INDENT}&& {RUNTIME_CLASS}::encode($this->{WRITE_METHOD}()) \
         === {RUNTIME_CLASS}::encode($other->{WRITE_METHOD}());\n\
         {MEMBER_INDENT}}}\n}}\n"
    ));

    Ok(source)
}

/// Fields in the one order a class declares them: what the document requires, then what it does
/// not, so that reading a constructor says what has to be passed before what may be left out.
fn ordered_fields(object: &ObjectShape) -> Vec<&Field> {
    let mut ordered: Vec<&Field> = object
        .fields
        .iter()
        .filter(|field| field.required)
        .collect();
    ordered.extend(object.fields.iter().filter(|field| !field.required));
    ordered
}

/// One documentation comment line, folded so that no line of it crosses [`MAX_LINE_CHARS`].
///
/// A description the document writes as one paragraph is one line here until it no longer fits;
/// what is left runs on under the continuation indent every documentation tool reads as belonging
/// to the tag above it. Folding happens between words, and a single word longer than a line is
/// written whole rather than cut in half — a name is worth more than a margin.
fn folded(indent: &str, text: &str) -> String {
    let continuation = format!("{indent} *   ");
    let opening = format!("{indent} * ");

    let mut lines = String::new();
    let mut line = opening.clone();
    let mut empty = true;

    for word in text.split(' ').filter(|word| !word.is_empty()) {
        if !empty && line.chars().count() + 1 + word.chars().count() > MAX_LINE_CHARS {
            lines.push_str(&line);
            lines.push('\n');
            line = continuation.clone();
            empty = true;
        }
        if !empty {
            line.push(' ');
        }
        line.push_str(word);
        empty = false;
    }

    if empty {
        line.push_str("undocumented by the API");
    }
    lines.push_str(&line);
    lines.push('\n');
    lines
}

/// One call, on one line when it fits and one argument per line when it does not.
///
/// Which of the two a call is written as depends only on how long it is, so the same model always
/// yields the same bytes; and a call that would have run past the margin is broken here rather than
/// left for something downstream to reformat.
fn call(indent: &str, called: &str, arguments: &[String], trailing: &str) -> String {
    let single = format!("{indent}{called}({}){trailing}", arguments.join(", "));
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }

    let mut source = format!("{indent}{called}(\n");
    for argument in arguments {
        source.push_str(&format!("{indent}{MEMBER_INDENT}{argument},\n"));
    }
    source.push_str(&format!("{indent}){trailing}\n"));
    source
}

/// One function declaration and the brace opening its body.
///
/// A list that fits is written on one line, which puts the brace on the line under it; one that
/// does not is written one argument per line, which puts the brace beside the closing parenthesis.
/// Both are what the linter asks for, and which of the two applies depends only on how long the
/// declaration is.
fn declaration(
    indent: &str,
    name: &str,
    declared: &[String],
    returns: &str,
    always_folded: bool,
) -> String {
    let single = format!(
        "{indent}public function {name}({}){returns}",
        declared.join(", ")
    );
    let folds = always_folded && !declared.is_empty();

    if !folds && single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n{indent}{{\n");
    }

    let mut source = format!("{indent}public function {name}(\n");
    for argument in declared {
        source.push_str(&format!("{indent}{MEMBER_INDENT}{argument},\n"));
    }
    source.push_str(&format!("{indent}){returns} {{\n"));
    source
}

/// Which closed list a member is drawn from, when it is drawn from one.
///
/// The type already says it, so this only points at where the values are written down.
fn listed(shape: &Shape, types: &Types) -> Result<String, Error> {
    let Shape::Enum { name, .. } = shape else {
        return Ok(String::new());
    };
    Ok(format!(", one of `{}`", types.enumeration(name)?))
}

/// What a field says about itself beyond the name it carries, when the document says anything.
fn described(description: Option<&str>) -> String {
    match description {
        Some(text) => format!(": {}", comment(text)),
        None => ".".to_owned(),
    }
}

/// One problem the API reports, as an exception a caller can name on its own.
fn problem_exception(declared: &str, value: &str, base: &str) -> String {
    format!(
        "/**\n * The API reported `{}`.\n */\nfinal class {declared} extends {base}\n{{\n}}\n",
        comment(value)
    )
}

/// The failure every problem the API reports is a kind of, and what turns an answer into one.
fn base_exception(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    let discriminant = member_name(&model.errors.discriminant, language.casing.field, limits)?;
    let base = &types.problem_base;
    let catalogue = &types.problem_enum;

    // The discriminant is what says whether a body named a problem at all, and a document may
    // declare it as a member it does not require, so a body naming none reaches the lookup as a
    // key nothing answers to rather than as a member read off nothing.
    let named = match discriminant_field(&model.errors)?.required {
        true => format!("$problem->{discriminant}->value"),
        false => format!("$problem->{discriminant}?->value"),
    };

    let mut entries = String::new();
    for (value, declared) in &types.problems {
        let member = constant_name(value, language, limits)?;
        entries.push_str(&format!(
            "{BODY_INDENT}{catalogue}::{member}->value => {declared}::class,\n"
        ));
    }

    Ok(format!(
        "/**\n \
         * A failure the API answered with, whether or not it could be read as a problem.\n \
         */\n\
         class {base} extends \\RuntimeException\n\
         {{\n\
         {MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * The exception each problem the API names is raised as.\n\
         {MEMBER_INDENT} *\n\
         {MEMBER_INDENT} * Keyed by the values the catalogue declares rather than by the strings\n\
         {MEMBER_INDENT} * themselves, so the spelling of a problem lives in one place and a value\n\
         {MEMBER_INDENT} * the document stopped naming takes its entry with it.\n\
         {MEMBER_INDENT} *\n\
         {MEMBER_INDENT} * @var array<string, class-string<{base}>>\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}private const {PROBLEMS_CONSTANT} = [\n{entries}{MEMBER_INDENT}];\n\n\
         {MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * @param int $status what the API answered under\n\
         {MEMBER_INDENT} * @param {schema}|null $problem the document it answered, when this client could read one\n\
         {MEMBER_INDENT} * @param string $detail what to say about the failure\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}public function __construct(\n\
         {BODY_INDENT}public readonly int $status,\n\
         {BODY_INDENT}public readonly ?{schema} $problem,\n\
         {BODY_INDENT}string $detail,\n\
         {MEMBER_INDENT}) {{\n\
         {BODY_INDENT}parent::__construct($detail);\n\
         {MEMBER_INDENT}}}\n\n\
         {MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * Raise what the API reported, when what it answered was not a success.\n\
         {MEMBER_INDENT} *\n\
         {MEMBER_INDENT} * @param int $status what the API answered under\n\
         {MEMBER_INDENT} * @param string $payload the body it answered\n\
         {MEMBER_INDENT} * @throws {base} whichever problem the body names, and this one when it names none\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}public static function {RAISE_HELPER}(int $status, string $payload): void\n\
         {MEMBER_INDENT}{{\n\
         {BODY_INDENT}if ($status >= {LOWEST_SUCCESS} && $status < {LOWEST_REDIRECTION}) {{\n\
         {ARGUMENT_INDENT}return;\n\
         {BODY_INDENT}}}\n\n\
         {BODY_INDENT}$problem = self::{PROBLEM_HELPER}($payload);\n\
         {BODY_INDENT}if ($problem === null || !isset(self::{PROBLEMS_CONSTANT}[{named}])) {{\n\
         {ARGUMENT_INDENT}throw new self($status, $problem, {RUNTIME_CLASS}::unreadable($status, $payload));\n\
         {BODY_INDENT}}}\n\n\
         {BODY_INDENT}$raised = self::{PROBLEMS_CONSTANT}[{named}];\n\n\
         {BODY_INDENT}throw new $raised($status, $problem, {RUNTIME_CLASS}::reported($status, $problem));\n\
         {MEMBER_INDENT}}}\n\n\
         {MEMBER_INDENT}/**\n\
         {MEMBER_INDENT} * The problem a body names, when this client can read one out of it.\n\
         {MEMBER_INDENT} */\n\
         {MEMBER_INDENT}private static function {PROBLEM_HELPER}(string $payload): ?{schema}\n\
         {MEMBER_INDENT}{{\n\
         {BODY_INDENT}try {{\n\
         {ARGUMENT_INDENT}return {schema}::{READ_METHOD}({RUNTIME_CLASS}::decodePayload($payload));\n\
         {BODY_INDENT}}} catch ({DECODE_ERROR_CLASS}) {{\n\
         {ARGUMENT_INDENT}return null;\n\
         {BODY_INDENT}}}\n\
         {MEMBER_INDENT}}}\n\
         }}\n"
    ))
}

/// One class per entity the API declares, one method per operation it groups under it.
fn group(
    entity: &Entity,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let declared = types.group(&entity.name)?;

    let mut source = format!(
        "/**\n * What the API declares under `{}`, issued through the transport it is handed.\n */\n\
         final class {declared}\n{{\n\
         {MEMBER_INDENT}public function __construct(private readonly {TRANSPORT_CLASS} $transport)\n\
         {MEMBER_INDENT}{{\n{MEMBER_INDENT}}}\n",
        comment(&entity.name)
    );

    for method in &entity.methods {
        source.push('\n');
        source.push_str(&operation(method, types, language, limits)?);
    }

    // A class carries the helper its own operations reach, and not the other one. Both were
    // written out regardless of what the entity answers, which left `checkAnswer` private and
    // uncalled in the seven groups the API only ever answers a value from — code nothing in the
    // file can reach, and that a suite could only ever cover by naming it.
    if entity.answers_nothing() {
        source.push_str(&format!(
            "\n{MEMBER_INDENT}/**\n\
             {MEMBER_INDENT} * Raise what the API reported, and answer nothing when it reported nothing.\n\
             {MEMBER_INDENT} *\n\
             {MEMBER_INDENT} * @param array{{0: int, 1: string}} $answered the status and the body the transport answered\n\
             {MEMBER_INDENT} */\n\
             {MEMBER_INDENT}private function {CHECK_HELPER}(array $answered): void\n\
             {MEMBER_INDENT}{{\n\
             {BODY_INDENT}{}::{RAISE_HELPER}($answered[0], $answered[1]);\n\
             {MEMBER_INDENT}}}\n",
            types.problem_base
        ));
    }
    if entity.answers_a_value() {
        source.push_str(&format!(
            "\n{MEMBER_INDENT}/**\n\
             {MEMBER_INDENT} * Raise what the API reported, or read back the value it answered.\n\
             {MEMBER_INDENT} *\n\
             {MEMBER_INDENT} * @param array{{0: int, 1: string}} $answered the status and the body the transport answered\n\
             {MEMBER_INDENT} * @param \\Closure $reader what turns that body into the value the API declares\n\
             {MEMBER_INDENT} */\n\
             {MEMBER_INDENT}private function {READ_HELPER}(array $answered, \\Closure $reader): mixed\n\
             {MEMBER_INDENT}{{\n\
             {BODY_INDENT}{}::{RAISE_HELPER}($answered[0], $answered[1]);\n\n\
             {BODY_INDENT}return $reader({RUNTIME_CLASS}::decodePayload($answered[1]));\n\
             {MEMBER_INDENT}}}\n",
            types.problem_base
        ));
    }
    source.push_str("}\n");

    Ok(source)
}

/// One argument of an emitted method: what it is called, what it carries, and where it travels.
struct Argument<'a> {
    name: String,
    annotated: &'static str,
    parameter: &'a Parameter,
}

fn operation(
    method: &Method,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let name = method_name(&method.verb_text, language, limits)?;
    let operation = &method.operation;

    let mut path_parameters = Vec::new();
    let mut query_parameters = Vec::new();
    for parameter in &operation.parameters {
        match parameter.location {
            ParameterLocation::Path => path_parameters.push(parameter),
            ParameterLocation::Query => query_parameters.push(parameter),
            ParameterLocation::Header | ParameterLocation::Cookie => {
                return Err(Error::UnsupportedParameter {
                    operation_id: preview(&method.operation_id),
                    parameter: preview(&parameter.name),
                });
            }
        }
    }

    // A path parameter is required by construction, and one that carries no value would leave the
    // path template with a hole in it, so it is asked for before anything that may be left out.
    let mut required: Vec<Argument<'_>> = Vec::new();
    let mut optional: Vec<Argument<'_>> = Vec::new();
    for parameter in path_parameters.iter().chain(query_parameters.iter()) {
        let argument = Argument {
            name: member_name(&parameter.name, language.casing.parameter, limits)?,
            annotated: scalar_type(parameter)?,
            parameter,
        };
        if parameter.required || parameter.location == ParameterLocation::Path {
            required.push(argument);
        } else {
            optional.push(argument);
        }
    }

    let body_argument = match method.request.as_ref() {
        Some(_) => Some(member_name(
            BODY_ARGUMENT,
            language.casing.parameter,
            limits,
        )?),
        None => None,
    };
    refuse_arguments_spelled_alike(method, &required, &optional, body_argument.as_deref())?;

    let returned = match method.success.as_ref() {
        Some((_, Some(shape))) => Some(shape),
        _ => None,
    };

    let mut documented = folded(MEMBER_INDENT, &summary(method));
    documented.push_str(&format!("{MEMBER_INDENT} *\n"));
    for argument in &required {
        documented.push_str(&folded(
            MEMBER_INDENT,
            &format!(
                "@param {} ${} carries `{}`{}",
                argument.annotated,
                argument.name,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    if let (Some(body), Some(shape)) = (body_argument.as_deref(), method.request.as_ref()) {
        documented.push_str(&folded(
            MEMBER_INDENT,
            &format!(
                "@param {} ${body} what the operation reads",
                documented_type(shape, true, types)?
            ),
        ));
    }
    for argument in &optional {
        documented.push_str(&folded(
            MEMBER_INDENT,
            &format!(
                "@param {}|null ${} carries `{}`{}",
                argument.annotated,
                argument.name,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    if let Some(shape) = returned {
        documented.push_str(&folded(
            MEMBER_INDENT,
            &format!("@return {}", documented_type(shape, true, types)?),
        ));
    }

    let mut declared = Vec::new();
    for argument in &required {
        declared.push(format!("{} ${}", argument.annotated, argument.name));
    }
    if let (Some(body), Some(shape)) = (body_argument.as_deref(), method.request.as_ref()) {
        declared.push(format!("{} ${body}", native_type(shape, true, types)?));
    }
    for argument in &optional {
        declared.push(format!("?{} ${} = null", argument.annotated, argument.name));
    }

    let returns = match returned {
        Some(shape) => format!(": {}", native_type(shape, true, types)?),
        None => ": void".to_owned(),
    };

    let mut source = format!("{MEMBER_INDENT}/**\n{documented}{MEMBER_INDENT} */\n");
    source.push_str(&declaration(
        MEMBER_INDENT,
        &name,
        &declared,
        &returns,
        false,
    ));

    let issued = format!(
        "{ARGUMENT_INDENT}$this->transport->request(\n\
         {ARGUMENT_INDENT}{MEMBER_INDENT}{},\n{}{}\
         {ARGUMENT_INDENT}{MEMBER_INDENT}{},\n\
         {ARGUMENT_INDENT}),\n",
        literal(operation.method.as_str()),
        path(operation.path.as_str(), &required),
        query(&required, &optional),
        match method.request.as_ref() {
            None => "null".to_owned(),
            Some(shape) => writer(
                shape,
                &format!("${}", body_argument.as_deref().unwrap_or(BODY_ARGUMENT)),
                0
            )?,
        }
    );

    match returned {
        Some(shape) => {
            source.push_str(&format!(
                "{BODY_INDENT}return $this->{READ_HELPER}(\n{issued}{ARGUMENT_INDENT}{},\n\
                 {BODY_INDENT});\n",
                reader(shape, types, 0)?
            ));
        }
        None => {
            source.push_str(&format!(
                "{BODY_INDENT}$this->{CHECK_HELPER}(\n{issued}{BODY_INDENT});\n"
            ));
        }
    }
    source.push_str(&format!("{MEMBER_INDENT}}}\n"));

    Ok(source)
}

/// Refuses two arguments of one method that would be spelled the same way.
///
/// PHP has one namespace for the arguments of a function, so a path parameter and a query parameter
/// the document spells `event-id` and `event_id` would be one argument, and whichever one lost would
/// travel carrying the other one's value.
fn refuse_arguments_spelled_alike(
    method: &Method,
    required: &[Argument<'_>],
    optional: &[Argument<'_>],
    body: Option<&str>,
) -> Result<(), Error> {
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();
    if let Some(body) = body {
        claimed.insert(body, "the body the operation reads");
    }

    for argument in required.iter().chain(optional.iter()) {
        if let Some(first) = claimed.insert(&argument.name, &argument.parameter.name) {
            return Err(Error::SchemaNameCollision {
                name: preview(&argument.name),
                first: preview(first),
                second: preview(&format!(
                    "{} of {}",
                    argument.parameter.name, method.operation_id
                )),
            });
        }
    }

    Ok(())
}

/// Where the request lands, as one expression.
///
/// A path carrying no parameter is the template itself; one carrying parameters is the template and
/// the values to fill it with, which the runtime writes as segments naming nothing else.
fn path(template: &str, required: &[Argument<'_>]) -> String {
    let filled: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Path)
        .collect();

    if filled.is_empty() {
        return format!("{ARGUMENT_INDENT}{MEMBER_INDENT}{},\n", literal(template));
    }

    let mut source = format!(
        "{ARGUMENT_INDENT}{MEMBER_INDENT}{RUNTIME_CLASS}::path(\n\
         {ARGUMENT_INDENT}{BODY_INDENT}{},\n\
         {ARGUMENT_INDENT}{BODY_INDENT}[\n",
        literal(template)
    );
    for argument in &filled {
        source.push_str(&format!(
            "{ARGUMENT_INDENT}{ARGUMENT_INDENT}{} => ${},\n",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str(&format!(
        "{ARGUMENT_INDENT}{BODY_INDENT}],\n{ARGUMENT_INDENT}{MEMBER_INDENT}),\n"
    ));
    source
}

/// What travels in the query string, as one expression.
///
/// What the document requires is always sent; what it does not is sent only when the caller passed
/// it, which the runtime decides rather than the emitted method.
fn query(required: &[Argument<'_>], optional: &[Argument<'_>]) -> String {
    let asked: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
        .collect();

    if asked.is_empty() && optional.is_empty() {
        return format!("{ARGUMENT_INDENT}{MEMBER_INDENT}[],\n");
    }

    format!(
        "{ARGUMENT_INDENT}{MEMBER_INDENT}{RUNTIME_CLASS}::query(\n{}{}\
         {ARGUMENT_INDENT}{MEMBER_INDENT}),\n",
        pairs(&asked),
        pairs(&optional.iter().collect::<Vec<&Argument<'_>>>())
    )
}

/// The name a parameter travels under and the argument carrying its value, as a list of pairs.
///
/// One pair per line whatever the count: a list assembled on one line is as long as the parameters
/// the operation happens to declare, and that is not something a margin should depend on.
fn pairs(arguments: &[&Argument<'_>]) -> String {
    if arguments.is_empty() {
        return format!("{ARGUMENT_INDENT}{BODY_INDENT}[],\n");
    }

    let mut source = format!("{ARGUMENT_INDENT}{BODY_INDENT}[\n");
    for argument in arguments {
        source.push_str(&format!(
            "{ARGUMENT_INDENT}{ARGUMENT_INDENT}[{}, ${}],\n",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str(&format!("{ARGUMENT_INDENT}{BODY_INDENT}],\n"));
    source
}

/// The type a value of that shape is declared as, which the language checks at every call.
///
/// Optionality is membership in `required` and nothing else. `mixed` already admits nothing, so a
/// value of a shape the document does not describe is never marked nullable on top: the language
/// refuses `?mixed` outright.
fn native_type(shape: &Shape, required: bool, types: &Types) -> Result<String, Error> {
    let declared = declared_type(shape, types, 0)?;
    if required || declared == JSON_TYPE {
        return Ok(declared);
    }
    Ok(format!("?{declared}"))
}

/// What a value of that shape carries, as a documentation comment says it.
///
/// The language has no way to say what a list holds or what a map is keyed by, so that is said
/// here, beside the declaration rather than instead of it.
fn documented_type(shape: &Shape, required: bool, types: &Types) -> Result<String, Error> {
    let declared = detailed_type(shape, types, 0)?;
    if required || declared == JSON_TYPE {
        return Ok(declared);
    }
    Ok(format!("{declared}|null"))
}

/// What the language calls a value it knows nothing else about.
const JSON_TYPE: &str = "mixed";

/// What the language calls a list and a map alike, neither of which it can say anything more about.
const COLLECTION_TYPE: &str = "array";

fn declared_type(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => types.scalars.of(scalar).to_owned(),
        Shape::Array(_) | Shape::Map(_) => COLLECTION_TYPE.to_owned(),
        Shape::Enum { name, .. } => types.enumeration(name)?.to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => JSON_TYPE.to_owned(),
    })
}

fn detailed_type(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Array(inner) => format!("list<{}>", detailed_type(inner, types, depth + 1)?),
        Shape::Map(inner) => {
            format!("array<string, {}>", detailed_type(inner, types, depth + 1)?)
        }
        _ => declared_type(shape, types, depth)?,
    })
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_type(parameter: &Parameter) -> Result<&'static str, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "string",
        "integer" => "int",
        "number" => "float",
        "boolean" => "bool",
        declared => {
            return Err(Error::UnknownSchemaType {
                subject: preview(&parameter.name),
                declared: preview(declared),
            });
        }
    })
}

/// What reads a value of that shape out of what the API answered.
fn reader(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => format!("{RUNTIME_CLASS}::{}(...)", scalar_reader(scalar)),
        Shape::Array(inner) => format!(
            "{RUNTIME_CLASS}::listOf({})",
            reader(inner, types, depth + 1)?
        ),
        Shape::Map(inner) => format!(
            "{RUNTIME_CLASS}::mapOf({})",
            reader(inner, types, depth + 1)?
        ),
        Shape::Enum { name, .. } => format!(
            "{RUNTIME_CLASS}::memberOf({}::class)",
            types.enumeration(name)?
        ),
        Shape::Named(name) => format!("{}::{READ_METHOD}(...)", types.schema(name)?),
        Shape::Object(object) => format!("{}::{READ_METHOD}(...)", types.schema(&object.name)?),
        Shape::Json => format!("{RUNTIME_CLASS}::jsonValue(...)"),
    })
}

fn scalar_reader(scalar: &Scalar) -> &'static str {
    match scalar {
        Scalar::String | Scalar::Url => "text",
        Scalar::Uuid => "uuid",
        Scalar::DateTime => "dateTime",
        Scalar::Date => "date",
        Scalar::Integer32 | Scalar::Integer64 => "integer",
        Scalar::Number => "float",
        Scalar::Boolean => "boolean",
    }
}

/// What writes `subject` back the way the API reads it.
fn writer(shape: &Shape, subject: &str, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        // How much of a moment survives being written is a decision, not a spelling: it is made
        // once, in the hand-written runtime, rather than emitted into every type that carries one.
        Shape::Scalar(Scalar::DateTime) => format!("{RUNTIME_CLASS}::moment({subject})"),
        Shape::Scalar(Scalar::Date) => format!("{RUNTIME_CLASS}::day({subject})"),
        Shape::Scalar(_) | Shape::Json => subject.to_owned(),
        Shape::Enum { .. } => format!("{subject}->value"),
        Shape::Named(_) | Shape::Object(_) => format!("{subject}->{WRITE_METHOD}()"),
        Shape::Array(inner) => {
            let item = format!("$item{depth}");
            match writer(inner, &item, depth + 1)? {
                written if written == item => subject.to_owned(),
                written => format!("array_map(static fn ({item}) => {written}, {subject})"),
            }
        }
        // An open-keyed object carrying nothing is still an object, and a language whose one array
        // stands for both a list and a map writes an empty one as `[]` unless it is told otherwise.
        Shape::Map(inner) => {
            let value = format!("$value{depth}");
            match writer(inner, &value, depth + 1)? {
                written if written == value => format!("{RUNTIME_CLASS}::mapping({subject})"),
                written => format!(
                    "{RUNTIME_CLASS}::mapping(array_map(static fn ({value}) => {written}, {subject}))"
                ),
            }
        }
    })
}

fn deep_enough(depth: usize) -> Result<(), Error> {
    if depth > Limits::DEFAULT.max_shape_depth {
        return Err(Error::SchemaTooDeep {
            subject: preview("a value of the model"),
            limit: Limits::DEFAULT.max_shape_depth,
        });
    }
    Ok(())
}

/// The name a type is declared under, out of the way of the words PHP keeps for a class name.
fn type_name(text: &str, language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    spell(text, language.casing.type_name, language.reserved, limits)
}

/// The name a method is spelled under.
///
/// PHP reads what follows `function`, `->` and `::` as a name rather than as a keyword, so a method
/// is spelled out of the way of far less than a class is: an operation the document calls `list` is
/// a method called `list`, and escaping it would spell `list_` in a published package for a
/// collision the language does not have. What a method still has to stay clear of is what the
/// emitted declaration around it already answers to.
fn method_name(text: &str, language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    spell(text, language.casing.method, super::php_shadowed(), limits)
}

/// The name a member — a property, an argument or an enumeration case — is spelled under.
///
/// A property is read after `->`, an argument is written after `$`, and an enumeration case is read
/// after `::`, none of which the language reads a keyword in. What they do collide with is the one
/// word a class constant may not be spelled as and the names the emitted declaration itself
/// carries, which is what [`super::php_shadowed`] holds.
fn member_name(text: &str, case: Case, limits: &Limits) -> Result<String, Error> {
    spell(text, case, super::php_shadowed(), limits)
}

/// The name an enumeration case is spelled under.
fn constant_name(text: &str, language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    member_name(text, language.casing.constant, limits)
}

/// What a method says about itself, from what the document says about the operation.
fn summary(method: &Method) -> String {
    let described = method
        .operation
        .summary
        .as_ref()
        .or(method.operation.description.as_ref());

    match described {
        Some(text) => comment(text),
        None => format!(
            "`{}` on `{}`.",
            comment(method.operation.method.as_str()),
            comment(&method.operation.path)
        ),
    }
}

/// Snapshot text, as a comment may carry it.
///
/// The snapshot is untrusted input travelling into source: a run of whitespace becomes one space so
/// nothing leaves the line the comment sits on, `*/` is spaced apart so nothing closes the block it
/// sits in, and what is left is cut at a fixed budget.
fn comment(text: &str) -> String {
    let collapsed: String = text
        .chars()
        .map(|character| {
            if character.is_whitespace() || character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();

    let mut rendered = String::new();
    let mut spaced = false;
    for character in collapsed.chars() {
        if rendered.chars().count() >= MAX_COMMENT_CHARS {
            break;
        }
        if character == ' ' {
            if !rendered.is_empty() {
                spaced = true;
            }
            continue;
        }
        if spaced {
            rendered.push(' ');
            spaced = false;
        }
        if character == '/' && rendered.ends_with('*') {
            rendered.push(' ');
        }
        rendered.push(character);
    }

    if rendered.is_empty() {
        return "undocumented by the API".to_owned();
    }
    rendered
}

/// Snapshot text, as a string literal may carry it.
///
/// A single-quoted PHP string reads nothing but `\\` and `\'`, which is what makes it the right
/// quoting for text that interpolates nothing — and what makes it unable to carry a control
/// character at all. Text carrying one is written double-quoted instead, where every byte has a
/// spelling, and `$` is spelled out of the way there since the language would otherwise read what
/// follows it as something to evaluate.
fn literal(text: &str) -> String {
    if !text.chars().any(char::is_control) {
        let escaped: String = text
            .chars()
            .map(|character| match character {
                '\\' => "\\\\".to_owned(),
                '\'' => "\\'".to_owned(),
                plain => plain.to_string(),
            })
            .collect();
        return format!("'{escaped}'");
    }

    let mut rendered = String::from("\"");
    for character in text.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '$' => rendered.push_str("\\$"),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            // Spelled as the code point rather than as the byte it would be written as, so that a
            // literal never carries a byte that is not text.
            control if control.is_control() => {
                rendered.push_str(&format!("\\u{{{:04x}}}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }

    rendered.push('"');
    rendered
}
