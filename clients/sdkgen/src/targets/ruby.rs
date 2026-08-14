//! Emits the generated half of the Ruby SDK.
//!
//! The gem is published with no copy of the OpenAPI snapshot beside it, so the types, the problems
//! and the request layer travel as committed source rather than as a build artefact. Everything the
//! API declares — one class per named schema, one module of constants per closed list of strings,
//! one exception per problem the error contract can report, one method per operation — is written
//! here; everything the API does not declare — how a request reaches the network, how a send is
//! retried, how a webhook signature is verified — is hand-written beside this directory and never
//! regenerated.
//!
//! The two halves meet at two seams and nowhere else. The generated code reads its decoding helpers
//! from the hand-written runtime module above it, and it calls whatever object it is handed as a
//! transport: nothing here knows what a socket is, and nothing beside it knows what the API
//! declares.
//!
//! What is written is already linted. An emitted method holds no local variable of its own — the
//! path, the query and the answer are all expressions — so an operation whose parameter is spelled
//! like one of the emitter's own locals cannot quietly be assigned over. That is what keeps the
//! reserved vocabulary to the words a *member* would shadow, and leaves fields named `body`,
//! `status` or `payload` spelled the way the API spells them.
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
use crate::targets::{LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "ruby";

/// Where the generated half of the gem lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written. A gem keeps
/// its source under `lib` and its tests outside it, so the generated tree sits three segments deep
/// rather than directly under the client.
const ROOT: &str = "clients/ruby/lib/hook0/generated";

/// File the generated code reads its decoding helpers from, one directory above the generated
/// package. Written as a path relative to the file requiring it, which is how Ruby names a
/// neighbour without going through the load path.
const RUNTIME_FILE: &str = "../runtime";

/// The files this target writes, each one holding one layer of the surface.
const MODELS_FILE: &str = "models";
const ERRORS_FILE: &str = "errors";
const API_FILE: &str = "api";

/// File requiring every other one, so that reaching the generated half is one `require` whatever
/// the API grows. Its name never changes, which is what keeps the hand-written half from having to
/// be edited when a layer is added.
const INDEX_FILE: &str = "all";

/// Namespace the gem is reached under.
const GEM_MODULE: &str = "Hook0";

/// Namespace everything written here lands in, under the gem's own.
const GENERATED_MODULE: &str = "Generated";

/// Module the hand-written decoding helpers are reached through.
const RUNTIME_MODULE: &str = "Runtime";

/// Constant naming the exception each problem is raised as.
const PROBLEMS_CONSTANT: &str = "PROBLEMS";

/// Constant every closed list of strings declares beside its values.
const VALUES_CONSTANT: &str = "VALUES";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix an exception carries, so a problem and a type spelling the same word stay apart.
const EXCEPTION_SUFFIX: &str = "Error";

/// What an emitted group calls the helper that raises what the API reported and hands back nothing.
const CHECK_HELPER: &str = "check_answer";

/// What an emitted group calls the helper that raises what the API reported and reads what it
/// answered.
const READ_HELPER: &str = "read_answer";

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

/// The names the scaffolding around the emitted code answers to, which no type of the document may
/// answer to as well.
///
/// They are claimed alongside everything the model names, so a schema called `Runtime` is reported
/// as the collision it is rather than emitted as a constant that shadows the hand-written module
/// every decoder reaches through. Sorted, as they are claimed in order.
const SCAFFOLDING: [&str; 4] = [
    GEM_MODULE,
    GENERATED_MODULE,
    PROBLEMS_CONSTANT,
    RUNTIME_MODULE,
];

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan nothing requires.
        ownership: Ownership::Directory,
        language: super::ruby(),
        emit,
    }
}

/// Everything the generated half of the gem is made of.
fn emit(language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;
    let banner = banner(language.comment, &update_command(NAME), &limits)?;

    let enums = model.enumerations(&limits)?;
    let types = Types::read(model, &enums, language, &limits)?;

    let files = vec![
        file(
            MODELS_FILE,
            &banner,
            &models(model, &enums, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            ERRORS_FILE,
            &banner,
            &errors(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            API_FILE,
            &banner,
            &requests(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(INDEX_FILE, &banner, &index(), language, &limits)?,
    ];

    FileTree::build(files, &limits)
}

/// One file: the banner, the pragma that freezes its literals, and the body.
///
/// Ruby reads a pragma out of the comments a file opens with, so it sits under the banner rather
/// than above it: the first line of an emitted file says what wrote it, which is what someone
/// opening the file has to read first.
fn file(
    stem: &str,
    banner: &str,
    body: &str,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<EmittedFile, Error> {
    Ok(EmittedFile {
        path: RelativePath::build(&format!("{stem}.{}", language.extension), limits)?,
        contents: format!("{banner}# frozen_string_literal: true\n\n{body}"),
    })
}

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a constant that silently replaces another one when the file is required.
struct Types {
    /// Class each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Module each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Exception each problem is raised as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Exception every problem is a kind of.
    problem_base: String,
    /// Module the discriminant of the error contract is read through.
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
            let declared = claim(
                ident(name, language.casing.type_name, language, limits)?,
                name,
            )?;
            schemas.insert(name.clone(), declared);
        }

        let mut declared_enums = BTreeMap::new();
        for name in enums.keys() {
            let declared = claim(
                ident(name, language.casing.type_name, language, limits)?,
                name,
            )?;
            declared_enums.insert(name.clone(), declared);
        }

        let problem_enum = enum_of(&model.errors, &declared_enums)?;

        let base = format!(
            "{}{EXCEPTION_SUFFIX}",
            ident(
                &model.errors.schema,
                language.casing.type_name,
                language,
                limits
            )?
        );
        let problem_base = claim(base, &model.errors.schema)?;

        let mut problems = BTreeMap::new();
        for value in model.errors.catalogue.values() {
            let name = format!(
                "{}{EXCEPTION_SUFFIX}",
                ident(value, language.casing.type_name, language, limits)?
            );
            problems.insert(value.clone(), claim(name, value)?);
        }

        let mut groups = BTreeMap::new();
        for entity in model.entities.entities() {
            let stem = ident(&entity.name, language.casing.type_name, language, limits)?;
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
/// module already exists among the types: it is found rather than declared twice.
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

/// The types every body the API reads and writes is made of.
fn models(
    model: &ApiModel,
    enums: &BTreeMap<String, Vec<String>>,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut body = String::new();
    let mut first = true;

    for (name, values) in enums {
        separated(&mut body, &mut first);
        body.push_str(&enumeration(
            types.enumeration(name)?,
            values,
            language,
            limits,
        )?);
    }
    for (name, object) in &model.schemas {
        separated(&mut body, &mut first);
        body.push_str(&structure(
            types.schema(name)?,
            object,
            types,
            language,
            limits,
        )?);
    }

    Ok(format!(
        "require \"date\"\nrequire \"time\"\n\nrequire_relative \"{RUNTIME_FILE}\"\n\n\
         module {GEM_MODULE}\n  \
         # Everything the API declares, as the values a caller reads and writes.\n  \
         module {GENERATED_MODULE}\n{body}  end\nend\n"
    ))
}

/// A blank line between two declarations, and none before the first.
fn separated(body: &mut String, first: &mut bool) {
    if *first {
        *first = false;
        return;
    }
    body.push('\n');
}

/// One closed list of strings, as a module holding one constant per value it admits.
///
/// The values travel as the strings the API answers, so nothing has to be mapped on the way in or
/// on the way out; the module is what names them and what says which ones there are.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    // The constant listing the values is claimed first, so a value spelling it is reported rather
    // than assigned over.
    let mut members: BTreeMap<String, &str> = BTreeMap::new();
    members.insert(VALUES_CONSTANT.to_owned(), "the list this module declares");

    let mut declarations = String::new();
    let mut listed = String::new();

    for (index, value) in values.iter().enumerate() {
        let member = ident(value, language.casing.constant, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        declarations.push_str(&format!("      {member} = {}\n", literal(value)));
        listed.push_str(&format!("        {member}"));
        listed.push_str(if index + 1 == values.len() {
            "\n"
        } else {
            ",\n"
        });
    }

    Ok(format!(
        "    # One of the values the API answers with.\n    \
         module {declared}\n{declarations}\n      \
         # Every value the API declares for this list.\n      \
         {VALUES_CONSTANT} = [\n{listed}      ].freeze\n\n      \
         # Whether the API declares that value.\n      \
         def self.member?(value)\n        \
         {VALUES_CONSTANT}.include?(value)\n      \
         end\n    end\n"
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
        let name = ident(&field.name, language.casing.field, language, limits)?;
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
        "    # The `{}` the API declares.\n    class {declared}\n",
        comment(&object.name)
    );

    source.push_str("      attr_reader ");
    for (index, (name, _)) in named.iter().enumerate() {
        if index > 0 {
            source.push_str("                  ");
        }
        source.push_str(&format!(
            ":{name}{}\n",
            if index + 1 == named.len() { "" } else { "," }
        ));
    }
    if named.is_empty() {
        // A schema declaring no member still needs the call to read as one.
        source.push('\n');
    }

    source.push('\n');
    for (name, field) in &named {
        source.push_str(&folded(
            "      ",
            &format!(
                "@param {name} [{}] carries `{}`{}{}",
                annotation(&field.shape, field.required, types)?,
                comment(&field.name),
                listed(&field.shape, types)?,
                described(field.description.as_deref())
            ),
        ));
    }
    let members: Vec<String> = named
        .iter()
        .map(|(name, field)| format!("{name}:{}", if field.required { "" } else { " nil" }))
        .collect();
    source.push_str(&arguments("      ", "initialize", &members));
    for (name, _) in &named {
        source.push_str(&format!("        @{name} = {name}\n"));
    }
    source.push_str("        freeze\n      end\n\n");

    source.push_str(&format!(
        "      # Read one out of what the API answered.\n      \
         #\n      \
         # @param value [Object] the JSON document the API answered\n      \
         # @return [{declared}]\n      \
         def self.from_json(value)\n        \
         fields = {RUNTIME_MODULE}.as_fields(value, {})\n        \
         new(\n",
        literal(&object.name)
    ));
    for (index, (name, field)) in named.iter().enumerate() {
        let trailing = if index + 1 == named.len() { "" } else { "," };
        let called = format!(
            "{RUNTIME_MODULE}.{}",
            if field.required { "read" } else { "maybe" }
        );
        let arguments = [
            "fields".to_owned(),
            literal(&field.name),
            reader(&field.shape, types, 0)?,
        ];
        source.push_str(&call(
            "          ",
            &format!("{name}: {called}"),
            &arguments,
            trailing,
        ));
    }
    source.push_str("        )\n      end\n\n");

    source.push_str(
        "      # Write one back the way the API reads it.\n      \
         #\n      \
         # @return [Hash{String => Object}]\n      \
         def to_h\n        out = {}\n",
    );
    for (name, field) in &named {
        let written = writer(&field.shape, &format!("@{name}"), 0)?;
        let assigned = format!("out[{}] = {written}", literal(&field.name));
        if field.required {
            source.push_str(&format!("        {assigned}\n"));
            continue;
        }

        // A member the document does not require is written only when it carries something. That
        // reads as one line until it no longer fits, which is the very rule the linter applies.
        let guarded = format!("        {assigned} unless @{name}.nil?");
        if guarded.chars().count() <= MAX_LINE_CHARS {
            source.push_str(&format!("{guarded}\n"));
        } else {
            source.push_str(&format!(
                "        unless @{name}.nil?\n          {assigned}\n        end\n"
            ));
        }
    }
    source.push_str("        out\n      end\n\n");

    source.push_str(&format!(
        "      # Whether that value carries the same members as this one.\n      \
         #\n      \
         # @param other [Object]\n      \
         # @return [Boolean]\n      \
         def ==(other)\n        \
         other.is_a?({declared}) && to_h == other.to_h\n      \
         end\n      \
         alias eql? ==\n\n      \
         # A value two equal instances share, so that either may key a hash.\n      \
         #\n      \
         # @return [Integer]\n      \
         def hash\n        to_h.hash\n      end\n    end\n"
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

/// One documentation comment, folded so that no line of it crosses [`MAX_LINE_CHARS`].
///
/// A description the document writes as one paragraph is one line here until it no longer fits;
/// what is left runs on under the continuation indent every documentation tool reads as belonging
/// to the tag above it. Folding happens between words, and a single word longer than a line is
/// written whole rather than cut in half — a name is worth more than a margin.
fn folded(indent: &str, text: &str) -> String {
    let continuation = format!("{indent}#   ");
    let opening = format!("{indent}# ");

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
    for (index, argument) in arguments.iter().enumerate() {
        source.push_str(&format!(
            "{indent}  {argument}{}\n",
            if index + 1 == arguments.len() {
                ""
            } else {
                ","
            }
        ));
    }
    source.push_str(&format!("{indent}){trailing}\n"));
    source
}

/// An argument list, on one line when it fits and one argument per line when it does not.
fn arguments(indent: &str, name: &str, declared: &[String]) -> String {
    let joined = declared.join(", ");
    let single = format!("{indent}def {name}({joined})\n");
    if declared.is_empty() {
        return format!("{indent}def {name}\n");
    }
    if single.chars().count() - 1 <= MAX_LINE_CHARS {
        return single;
    }

    let mut source = format!("{indent}def {name}(\n");
    for (index, argument) in declared.iter().enumerate() {
        source.push_str(&format!(
            "{indent}  {argument}{}\n",
            if index + 1 == declared.len() { "" } else { "," }
        ));
    }
    source.push_str(&format!("{indent})\n"));
    source
}

/// Which closed list a member is drawn from, when it is drawn from one.
///
/// A value of such a list travels as a plain string, so nothing in the source says which strings it
/// may be; the module that declares them is named here instead.
fn listed(shape: &Shape, types: &Types) -> Result<String, Error> {
    let Shape::Enum { name, .. } = shape else {
        return Ok(String::new());
    };
    Ok(format!(
        ", one of `{}::{VALUES_CONSTANT}`",
        types.enumeration(name)?
    ))
}

/// What a field says about itself beyond the name it carries, when the document says anything.
fn described(description: Option<&str>) -> String {
    match description {
        Some(text) => format!(": {}", comment(text)),
        None => ".".to_owned(),
    }
}

/// The problems the API reports, one exception each.
fn errors(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    let discriminant = ident(&model.errors.discriminant, Case::Snake, language, limits)?;
    let base = &types.problem_base;
    let catalogue = &types.problem_enum;

    let mut source = format!(
        "    # A failure the API answered with, whether or not it could be read as a problem.\n    \
         class {base} < StandardError\n      \
         attr_reader :status, :problem\n\n      \
         # @param status [Integer] what the API answered under\n      \
         # @param problem [{schema}, nil] the document it answered, when this client could read one\n      \
         # @param detail [String] what to say about the failure\n      \
         def initialize(status, problem, detail)\n        \
         super(detail)\n        \
         @status = status\n        \
         @problem = problem\n      \
         end\n    end\n"
    );

    for (value, declared) in &types.problems {
        source.push_str(&format!(
            "\n    # The API reported `{}`.\n    class {declared} < {base}; end\n",
            comment(value)
        ));
    }

    // Keyed by the constants the catalogue declares rather than by the strings themselves: the
    // spelling of a problem lives in one place, and a value the document stopped naming takes the
    // entry with it instead of leaving one nothing can ever match.
    source.push_str(&format!(
        "\n    # The exception each problem the API names is raised as.\n    \
         {PROBLEMS_CONSTANT} = {{\n"
    ));
    for (index, (value, declared)) in types.problems.iter().enumerate() {
        let member = ident(value, language.casing.constant, language, limits)?;
        source.push_str(&format!("      {catalogue}::{member} => {declared}"));
        source.push_str(if index + 1 == types.problems.len() {
            "\n"
        } else {
            ",\n"
        });
    }
    source.push_str("    }.freeze\n");

    // The discriminant is what says whether a body named a problem at all, and a document may
    // declare it as a member it does not require, so what it carries is looked up rather than
    // trusted to name an entry of the catalogue.
    source.push_str(&format!(
        "\n    # Raise what the API reported, when what it answered was not a success.\n    \
         #\n    \
         # @param status [Integer] what the API answered under\n    \
         # @param payload [String] the body it answered\n    \
         # @return [void]\n    \
         # @raise [{base}] whichever problem the body names, and the base one when it names none\n    \
         def self.raise_for_status(status, payload)\n      \
         return if status >= {LOWEST_SUCCESS} && status < {LOWEST_REDIRECTION}\n\n      \
         begin\n        \
         problem = {schema}.from_json({RUNTIME_MODULE}.decode_payload(payload))\n      \
         rescue {RUNTIME_MODULE}::DecodeError\n        \
         problem = nil\n      \
         end\n\n      \
         if problem.nil? || !{PROBLEMS_CONSTANT}.key?(problem.{discriminant})\n        \
         raise {base}.new(status, problem, {RUNTIME_MODULE}.unreadable(status, payload))\n      \
         end\n\n      \
         raise {PROBLEMS_CONSTANT}.fetch(problem.{discriminant})\
         .new(status, problem, {RUNTIME_MODULE}.reported(status, problem))\n    \
         end\n"
    ));

    Ok(format!(
        "require_relative \"{RUNTIME_FILE}\"\nrequire_relative \"{MODELS_FILE}\"\n\n\
         module {GEM_MODULE}\n  \
         # The failures the API reports, one exception per problem it can name.\n  \
         module {GENERATED_MODULE}\n{source}  end\nend\n"
    ))
}

/// One method per operation, grouped by the entity its operation id names.
fn requests(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut body = String::new();
    let mut first = true;

    for entity in model.entities.entities() {
        separated(&mut body, &mut first);
        body.push_str(&group(entity, types, language, limits)?);
    }

    Ok(format!(
        "require_relative \"{RUNTIME_FILE}\"\nrequire_relative \"{ERRORS_FILE}\"\n\
         require_relative \"{MODELS_FILE}\"\n\n\
         module {GEM_MODULE}\n  \
         # One class per entity the API declares, one method per operation it groups under it.\n  \
         module {GENERATED_MODULE}\n{body}  end\nend\n"
    ))
}

fn group(
    entity: &Entity,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let declared = types.group(&entity.name)?;

    let mut source = format!(
        "    # What the API declares under `{}`, issued through the transport it is handed.\n    \
         class {declared}\n      \
         # @param transport [Object] what one request is issued through\n      \
         def initialize(transport)\n        \
         @transport = transport\n      \
         end\n",
        comment(&entity.name)
    );

    for method in &entity.methods {
        source.push('\n');
        source.push_str(&operation(method, types, language, limits)?);
    }

    source.push_str(&format!(
        "\n      private\n\n      \
         # Raise what the API reported, and answer nothing when it reported nothing.\n      \
         #\n      \
         # @param answered [Array] the status and the body the transport answered\n      \
         # @return [void]\n      \
         def {CHECK_HELPER}(answered)\n        \
         status, payload = answered\n        \
         {GENERATED_MODULE}.raise_for_status(status, payload)\n        \
         nil\n      \
         end\n\n      \
         # Raise what the API reported, or read back the value it answered.\n      \
         #\n      \
         # @param answered [Array] the status and the body the transport answered\n      \
         # @param reader [#call] what turns that body into the value the API declares\n      \
         # @return [Object]\n      \
         def {READ_HELPER}(answered, reader)\n        \
         status, payload = answered\n        \
         {GENERATED_MODULE}.raise_for_status(status, payload)\n        \
         reader.call({RUNTIME_MODULE}.decode_payload(payload))\n      \
         end\n    end\n"
    ));

    Ok(source)
}

/// One argument of an emitted method: what it is called, what it carries, and where it travels.
struct Argument<'a> {
    name: String,
    annotated: String,
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
            name: ident(&parameter.name, language.casing.parameter, language, limits)?,
            annotated: scalar_annotation(parameter)?,
            parameter,
        };
        if parameter.required || parameter.location == ParameterLocation::Path {
            required.push(argument);
        } else {
            optional.push(argument);
        }
    }

    let body_argument = match method.request.as_ref() {
        Some(_) => Some(ident(BODY_ARGUMENT, Case::Snake, language, limits)?),
        None => None,
    };
    refuse_arguments_spelled_alike(method, &required, &optional, body_argument.as_deref())?;

    let returned = match method.success.as_ref() {
        Some((_, Some(shape))) => Some(annotation(shape, true, types)?),
        _ => None,
    };

    let mut source = folded("      ", &summary(method));
    source.push_str("      #\n");
    for argument in &required {
        source.push_str(&folded(
            "      ",
            &format!(
                "@param {} [{}] carries `{}`{}",
                argument.name,
                argument.annotated,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    if let (Some(body), Some(shape)) = (body_argument.as_deref(), method.request.as_ref()) {
        source.push_str(&folded(
            "      ",
            &format!(
                "@param {body} [{}] what the operation reads",
                annotation(shape, true, types)?
            ),
        ));
    }
    for argument in &optional {
        source.push_str(&folded(
            "      ",
            &format!(
                "@param {} [{}, nil] carries `{}`{}",
                argument.name,
                argument.annotated,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    source.push_str(&format!(
        "      # @return [{}]\n",
        returned.clone().unwrap_or_else(|| "void".to_owned())
    ));

    let mut declared = Vec::new();
    for argument in &required {
        declared.push(argument.name.clone());
    }
    if let Some(body) = body_argument.as_deref() {
        declared.push(body.to_owned());
    }
    for argument in &optional {
        declared.push(format!("{}: nil", argument.name));
    }
    source.push_str(&arguments("      ", &name, &declared));

    let issued = format!(
        "        @transport.request(\n          {},\n{}{}          {}\n        )",
        literal(operation.method.as_str()),
        path(operation.path.as_str(), &required),
        query(&required, &optional),
        match method.request.as_ref() {
            None => "nil".to_owned(),
            Some(shape) => writer(shape, body_argument.as_deref().unwrap_or(BODY_ARGUMENT), 0)?,
        }
    );

    // The call the helper is handed sits one step further in than the helper itself, which is what
    // the emitted source has to say for a linter to leave it alone.
    let handed = format!("  {}", issued.replace('\n', "\n  "));
    match method.success.as_ref() {
        Some((_, Some(shape))) => {
            source.push_str(&format!(
                "        {READ_HELPER}(\n{handed},\n          {}\n        )\n",
                reader(shape, types, 0)?
            ));
        }
        _ => source.push_str(&format!("        {CHECK_HELPER}(\n{handed}\n        )\n")),
    }
    source.push_str("      end\n");

    Ok(source)
}

/// What the argument carrying the body an operation reads is called.
const BODY_ARGUMENT: &str = "body";

/// Refuses two arguments of one method that would be spelled the same way.
///
/// Ruby has one namespace for the arguments of a method, so a path parameter and a query parameter
/// the document spells `event-id` and `event_id` would be one argument, and whichever one lost
/// would travel carrying the other one's value.
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
        return format!("          {},\n", literal(template));
    }

    let mut source = format!(
        "          {RUNTIME_MODULE}.path(\n            {},\n",
        literal(template)
    );
    for (index, argument) in filled.iter().enumerate() {
        source.push_str(&format!(
            "            {} => {}{}\n",
            literal(&argument.parameter.name),
            argument.name,
            if index + 1 == filled.len() { "" } else { "," }
        ));
    }
    source.push_str("          ),\n");
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
        return "          [],\n".to_owned();
    }

    format!(
        "          {RUNTIME_MODULE}.query(\n{}{}          ),\n",
        pairs(&asked, ","),
        pairs(&optional.iter().collect::<Vec<&Argument<'_>>>(), "")
    )
}

/// The name a parameter travels under and the argument carrying its value, as a list of pairs.
///
/// One pair per line whatever the count: a list assembled on one line is as long as the parameters
/// the operation happens to declare, and that is not something a margin should depend on.
fn pairs(arguments: &[&Argument<'_>], trailing: &str) -> String {
    if arguments.is_empty() {
        return format!("            []{trailing}\n");
    }

    let mut source = String::from("            [\n");
    for (index, argument) in arguments.iter().enumerate() {
        source.push_str(&format!(
            "              [{}, {}]{}\n",
            literal(&argument.parameter.name),
            argument.name,
            if index + 1 == arguments.len() {
                ""
            } else {
                ","
            }
        ));
    }
    source.push_str(&format!("            ]{trailing}\n"));
    source
}

/// The type a value of that shape carries, as a documentation comment names one.
///
/// Ruby says nothing about types in the source itself, so this is what a reader — and every tool
/// that reads documentation — is told. Optionality is membership in `required` and nothing else.
fn annotation(shape: &Shape, required: bool, types: &Types) -> Result<String, Error> {
    let declared = annotated(shape, types, 0)?;
    if required {
        return Ok(declared);
    }
    Ok(format!("{declared}, nil"))
}

fn annotated(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => types.scalars.of(scalar).to_owned(),
        Shape::Array(inner) => format!("Array<{}>", annotated(inner, types, depth + 1)?),
        Shape::Map(inner) => {
            format!("Hash{{String => {}}}", annotated(inner, types, depth + 1)?)
        }
        // A value of a closed list travels as the string the API answers, and that is what it is;
        // which strings it may be is said beside it, by naming the module that declares them.
        Shape::Enum { .. } => "String".to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => "Object".to_owned(),
    })
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "String",
        "integer" => "Integer",
        "number" => "Float",
        "boolean" => "Boolean",
        declared => {
            return Err(Error::UnknownSchemaType {
                subject: preview(&parameter.name),
                declared: preview(declared),
            });
        }
    }
    .to_owned())
}

/// What reads a value of that shape out of what the API answered.
fn reader(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => format!("{RUNTIME_MODULE}::{}", scalar_reader(scalar)),
        Shape::Array(inner) => format!(
            "{RUNTIME_MODULE}.list({})",
            reader(inner, types, depth + 1)?
        ),
        Shape::Map(inner) => format!("{RUNTIME_MODULE}.map({})", reader(inner, types, depth + 1)?),
        Shape::Enum { name, .. } => {
            format!("{RUNTIME_MODULE}.member_of({})", types.enumeration(name)?)
        }
        Shape::Named(name) => format!("{}.method(:from_json)", types.schema(name)?),
        Shape::Object(object) => format!("{}.method(:from_json)", types.schema(&object.name)?),
        Shape::Json => format!("{RUNTIME_MODULE}::JSON_VALUE"),
    })
}

fn scalar_reader(scalar: &Scalar) -> &'static str {
    match scalar {
        Scalar::String | Scalar::Url => "TEXT",
        Scalar::Uuid => "UUID",
        Scalar::DateTime => "DATE_TIME",
        Scalar::Date => "DATE",
        Scalar::Integer32 | Scalar::Integer64 => "INTEGER",
        Scalar::Number => "FLOAT",
        Scalar::Boolean => "BOOLEAN",
    }
}

/// What writes `subject` back the way the API reads it.
fn writer(shape: &Shape, subject: &str, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        // How much of a moment survives being written is a decision, not a spelling: it is made
        // once, in the hand-written runtime, rather than emitted into every type that carries one.
        Shape::Scalar(Scalar::DateTime) => format!("{RUNTIME_MODULE}.moment({subject})"),
        Shape::Scalar(Scalar::Date) => format!("{RUNTIME_MODULE}.day({subject})"),
        Shape::Scalar(_) | Shape::Enum { .. } | Shape::Json => subject.to_owned(),
        Shape::Named(_) | Shape::Object(_) => format!("{subject}.to_h"),
        Shape::Array(inner) => {
            let item = format!("item{depth}");
            match method_of(inner, &item, depth)? {
                Written::Itself => subject.to_owned(),
                Written::Method(called) => format!("{subject}.map(&:{called})"),
                Written::Expression(written) => {
                    format!("{subject}.map {{ |{item}| {written} }}")
                }
            }
        }
        Shape::Map(inner) => {
            let value = format!("value{depth}");
            match method_of(inner, &value, depth)? {
                Written::Itself => subject.to_owned(),
                Written::Method(called) => format!("{subject}.transform_values(&:{called})"),
                Written::Expression(written) => {
                    format!("{subject}.transform_values {{ |{value}| {written} }}")
                }
            }
        }
    })
}

/// How the items of a list or the values of a map are written back.
///
/// A value that travels as it stands needs no block at all, and one that is written by calling a
/// method on it is spelled as that method rather than as a block that does nothing else: both are
/// what a Ruby linter asks for, and neither is something a pass over the emitted source should have
/// to work out afterwards.
enum Written {
    Itself,
    Method(String),
    Expression(String),
}

fn method_of(shape: &Shape, subject: &str, depth: usize) -> Result<Written, Error> {
    let written = writer(shape, subject, depth + 1)?;

    if written == subject {
        return Ok(Written::Itself);
    }
    if let Some(called) = written.strip_prefix(&format!("{subject}."))
        && called.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return Ok(Written::Method(called.to_owned()));
    }
    Ok(Written::Expression(written))
}

/// What the gem requires to reach everything the generator writes.
fn index() -> String {
    format!(
        "require_relative \"{API_FILE}\"\nrequire_relative \"{ERRORS_FILE}\"\n\
         require_relative \"{MODELS_FILE}\"\n"
    )
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

/// The name a method is spelled under.
///
/// A method is spelled out of the way of less than a member is. Ruby reads what follows `def` and
/// `.` as a name rather than as a keyword, so `def next` is a method and `applications.next(id)`
/// calls it; escaping such a name would spell `next_` in a published gem for a collision the
/// language does not have. What a method still has to stay clear of is what every object already
/// answers to: a group declaring `class` or `hash` replaces the method of that name on itself.
fn method_name(text: &str, language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    spell(
        text,
        language.casing.method,
        super::ruby_method_reserved(),
        limits,
    )
}

/// The name that text is spelled under, out of the way of the language's own vocabulary.
fn ident(
    text: &str,
    case: Case,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    spell(text, case, language.reserved, limits)
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
/// nothing leaves the line the comment sits on, and what is left is cut at a fixed budget.
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
        rendered.push(character);
    }

    if rendered.is_empty() {
        return "undocumented by the API".to_owned();
    }
    rendered
}

/// Snapshot text, as a string literal may carry it.
///
/// A double-quoted Ruby string reads `#{…}`, `#$…` and `#@…` as something to evaluate, so a `#` is
/// spelled out of the way when it is followed by any of the three — and left alone otherwise, since
/// escaping what needs none is itself something a linter reports.
fn literal(text: &str) -> String {
    let mut rendered = String::from("\"");
    let mut characters = text.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            '#' => {
                let interpolates = matches!(characters.peek(), Some('{') | Some('$') | Some('@'));
                rendered.push_str(if interpolates { "\\#" } else { "#" });
            }
            // Spelled as the code point rather than as the byte it would be written as, so that a
            // literal never carries a byte that is not text.
            control if control.is_control() => {
                rendered.push_str(&format!("\\u{:04x}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }

    rendered.push('"');
    rendered
}
