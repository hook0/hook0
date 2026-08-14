//! Emits the generated half of the Lua SDK.
//!
//! The rock is published with no copy of the OpenAPI snapshot beside it, so the types, the problems
//! and the request layer travel as committed source rather than as a build artefact. Everything the
//! API declares — one table per named schema, one table of constants per closed list of strings, one
//! error kind per problem the error contract can report, one method per operation — is written here;
//! everything the API does not declare — how a request reaches the network, how a send is retried,
//! how a webhook signature is verified — is hand-written beside this directory and never
//! regenerated.
//!
//! The two halves meet at two seams and nowhere else. The generated code reads its decoding helpers
//! from the hand-written runtime module, and it calls whatever object it is handed as a transport:
//! nothing here knows what a socket is, and nothing beside it knows what the API declares.
//!
//! Every declaration is a member of the one table its file answers with, rather than a file-local:
//! Lua resolves a name that is not in scope yet as a global, so a type referring to one declared
//! further down the same file would read a global that is never assigned — and a chunk may hold two
//! hundred locals at most, which the surface of a growing API would eventually cross. Naming
//! everything through its module is what makes the order of declarations stop mattering.
//!
//! What is written is already linted. An emitted method holds no local of its own — the path, the
//! query and the answer are all expressions — so an operation whose parameter is spelled like one of
//! the emitter's own names cannot quietly be assigned over.
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
pub const NAME: &str = "lua";

/// Where the generated half of the rock lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written. The rockspec
/// maps a module name onto a source path, so what a file is called on disk and what a caller
/// requires it as are two different things: these land under `src` and are installed as
/// `hook0.generated.*`.
const ROOT: &str = "clients/lua/src/generated";

/// Module the generated code reads its decoding helpers from, as a caller requires it.
const RUNTIME_MODULE: &str = "hook0.runtime";

/// Module the hand-written error kinds are declared in.
const ERRORS_MODULE: &str = "hook0.errors";

/// What the generated modules are reached under, once the rock is installed.
const GENERATED_PREFIX: &str = "hook0.generated";

/// The files this target writes, each one holding one layer of the surface.
const MODELS_FILE: &str = "models";
const ERRORS_FILE: &str = "errors";
const API_FILE: &str = "api";

/// File requiring every other one, so that reaching the generated half is one `require` whatever
/// the API grows. Its name never changes, which is what keeps the hand-written half from having to
/// be edited when a layer is added.
const INDEX_FILE: &str = "all";

/// Local the types are declared as members of, and what `models` answers with.
const MODELS_TABLE: &str = "Models";

/// Local the problems are declared as members of, and what `errors` answers with.
const GENERATED_TABLE: &str = "Generated";

/// Local the operation groups are declared as members of, and what `api` answers with.
const API_TABLE: &str = "Api";

/// Local the decoding helpers are reached through.
const RUNTIME_TABLE: &str = "Runtime";

/// Local the hand-written error kinds are reached through.
const ERRORS_TABLE: &str = "Errors";

/// Member naming the error kind each problem is raised as.
const PROBLEMS_MEMBER: &str = "PROBLEMS";

/// Member every closed list of strings declares beside its values.
const VALUES_MEMBER: &str = "VALUES";

/// Member raising whatever the API reported, when what it answered was not a success.
const RAISE_MEMBER: &str = "raise_for_status";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix an error kind carries, so a problem and a type spelling the same word stay apart.
const ERROR_SUFFIX: &str = "Error";

/// What an emitted group calls the helper that raises what the API reported and hands back nothing.
const CHECK_HELPER: &str = "check_answer";

/// What an emitted group calls the helper that raises what the API reported and reads what it
/// answered.
const READ_HELPER: &str = "read_answer";

/// What the argument carrying the body an operation reads is called.
const BODY_ARGUMENT: &str = "body";

/// What the table a constructor reads its members out of is called.
const FIELDS_ARGUMENT: &str = "fields";

/// Longest fragment of a snapshot description a comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Longest line the emitted source carries, which is what `luacheck` accepts by default.
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
/// as the collision it is rather than emitted as a member that shadows the hand-written module every
/// decoder reaches through.
const SCAFFOLDING: [&str; 8] = [
    API_TABLE,
    ERRORS_TABLE,
    GENERATED_TABLE,
    MODELS_TABLE,
    PROBLEMS_MEMBER,
    RUNTIME_TABLE,
    CHECK_HELPER,
    READ_HELPER,
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
        language: super::lua(),
        emit,
    }
}

/// Everything the generated half of the rock is made of.
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

/// One file: the banner and the body.
fn file(
    stem: &str,
    banner: &str,
    body: &str,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<EmittedFile, Error> {
    Ok(EmittedFile {
        path: RelativePath::build(&format!("{stem}.{}", language.extension), limits)?,
        contents: format!("{banner}\n{body}"),
    })
}

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a member that silently replaces another one when the module is required.
struct Types {
    /// Table each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Table each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Error kind each problem is raised as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Kind every problem is a kind of.
    problem_base: String,
    /// Table the discriminant of the error contract is read through.
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
            "{}{ERROR_SUFFIX}",
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
                "{}{ERROR_SUFFIX}",
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
/// table already exists among the types: it is found rather than declared twice.
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

    for (name, values) in enums {
        body.push('\n');
        body.push_str(&enumeration(
            types.enumeration(name)?,
            values,
            language,
            limits,
        )?);
    }
    for (name, object) in &model.schemas {
        body.push('\n');
        body.push_str(&structure(
            types.schema(name)?,
            object,
            types,
            language,
            limits,
        )?);
    }

    Ok(format!(
        "local {RUNTIME_TABLE} = require({})\n\n\
         --- Everything the API declares, as the values a caller reads and writes.\n\
         local {MODELS_TABLE} = {{}}\n{body}\n\
         return {MODELS_TABLE}\n",
        literal(RUNTIME_MODULE)
    ))
}

/// One closed list of strings, as a table holding one member per value it admits.
///
/// The values travel as the strings the API answers, so nothing has to be mapped on the way in or
/// on the way out; the table is what names them and what says which ones there are.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    // The member listing the values is claimed first, so a value spelling it is reported rather
    // than assigned over.
    let mut members: BTreeMap<String, &str> = BTreeMap::new();
    members.insert(VALUES_MEMBER.to_owned(), "the list this table declares");

    let mut declarations = String::new();
    let mut listed = String::new();

    for value in values {
        let member = ident(value, language.casing.constant, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        declarations.push_str(&format!("  {member} = {},\n", literal(value)));
        listed.push_str(&format!("  {MODELS_TABLE}.{declared}.{member},\n"));
    }

    Ok(format!(
        "--- One of the values the API answers with.\n\
         {MODELS_TABLE}.{declared} = {{\n{declarations}}}\n\n\
         --- Every value the API declares for this list.\n\
         {MODELS_TABLE}.{declared}.{VALUES_MEMBER} = {{\n{listed}}}\n\n\
         --- Whether the API declares that value.\n\
         --- @param value string\n\
         --- @return boolean\n\
         function {MODELS_TABLE}.{declared}.member(value)\n  \
         return {RUNTIME_TABLE}.declares({MODELS_TABLE}.{declared}.{VALUES_MEMBER}, value)\n\
         end\n"
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

    let held = format!("{MODELS_TABLE}.{declared}");
    let mut source = format!(
        "--- The `{}` the API declares.\n\
         {held} = {{}}\n\
         {held}.__index = {held}\n\
         {held}.__eq = {RUNTIME_TABLE}.equality\n\n",
        comment(&object.name)
    );

    source.push_str("--- Build one out of the members it carries.\n");
    for (name, field) in &named {
        source.push_str(&folded(
            "",
            &format!(
                "@param {FIELDS_ARGUMENT}.{name} {} carries `{}`{}{}",
                annotation(&field.shape, field.required, types)?,
                comment(&field.name),
                listed(&field.shape, types)?,
                described(field.description.as_deref())
            ),
        ));
    }
    source.push_str(&format!(
        "--- @return {declared}\n\
         function {held}.new({FIELDS_ARGUMENT})\n  \
         return setmetatable({{\n"
    ));
    for (name, _) in &named {
        source.push_str(&format!("    {name} = {FIELDS_ARGUMENT}.{name},\n"));
    }
    source.push_str(&format!("  }}, {held})\nend\n\n"));

    source.push_str(&format!(
        "--- Read one out of what the API answered.\n\
         --- @param value table the JSON document the API answered\n\
         --- @return {declared}\n\
         function {held}.from_json(value)\n  \
         local {FIELDS_ARGUMENT} = {RUNTIME_TABLE}.as_fields(value, {})\n  \
         return {held}.new({{\n",
        literal(&object.name)
    ));
    for (name, field) in &named {
        let called = format!(
            "{RUNTIME_TABLE}.{}",
            if field.required { "read" } else { "maybe" }
        );
        let arguments = [
            FIELDS_ARGUMENT.to_owned(),
            literal(&field.name),
            reader(&field.shape, types, 0)?,
        ];
        source.push_str(&call(
            "    ",
            &format!("{name} = {called}"),
            &arguments,
            ",",
        ));
    }
    source.push_str("  })\nend\n\n");

    source.push_str(&format!(
        "--- Write one back the way the API reads it.\n\
         --- @return table\n\
         function {held}:to_table()\n  \
         return {RUNTIME_TABLE}.document({{\n"
    ));
    for (name, field) in &named {
        let written = writer(&field.shape, &format!("self.{name}"), 0)?;
        source.push_str(&assignment("    ", &field.name, &written));
    }
    source.push_str("  })\nend\n");

    Ok(source)
}

/// One member of the document a value is written back as, on one line when it fits.
///
/// The key is written as a literal rather than as an identifier: what the API reads is the name the
/// document spells, which is not always a name Lua would accept after a dot.
fn assignment(indent: &str, key: &str, written: &str) -> String {
    let single = format!("{indent}[{}] = {written},", literal(key));
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }
    format!("{indent}[{}] =\n{indent}  {written},\n", literal(key))
}

/// Fields in the one order a constructor documents them: what the document requires, then what it
/// does not, so that reading one says what has to be passed before what may be left out.
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
    let continuation = format!("{indent}---   ");
    let opening = format!("{indent}--- ");

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
fn arguments(name: &str, declared: &[String]) -> String {
    if declared.is_empty() {
        return format!("function {name}()\n");
    }

    let single = format!("function {name}({})", declared.join(", "));
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }

    let mut source = format!("function {name}(\n");
    for (index, argument) in declared.iter().enumerate() {
        source.push_str(&format!(
            "  {argument}{}\n",
            if index + 1 == declared.len() { "" } else { "," }
        ));
    }
    source.push_str(")\n");
    source
}

/// Which closed list a member is drawn from, when it is drawn from one.
///
/// A value of such a list travels as a plain string, so nothing in the source says which strings it
/// may be; the table that declares them is named here instead.
fn listed(shape: &Shape, types: &Types) -> Result<String, Error> {
    let Shape::Enum { name, .. } = shape else {
        return Ok(String::new());
    };
    Ok(format!(
        ", one of `{MODELS_TABLE}.{}.{VALUES_MEMBER}`",
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

/// One error kind, declared under the kind it derives from.
fn kind(declared: &str, named: &str, parent: &str) -> String {
    call(
        "",
        &format!("{GENERATED_TABLE}.{declared} = {ERRORS_TABLE}.kind"),
        &[literal(named), parent.to_owned()],
        "",
    )
}

/// The problems the API reports, one error kind each.
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
        "--- A failure the API answered with, whether or not it could be read as a problem.\n\
         ---\n\
         --- A raised value carries the status the API answered under and, when this client could\n\
         --- read one, the problem document it answered.\n\
         {}",
        kind(base, base, &format!("{ERRORS_TABLE}.ClientError"))
    );

    for (value, declared) in &types.problems {
        source.push_str(&format!(
            "\n--- The API reported `{}`.\n{}",
            comment(value),
            kind(declared, declared, &format!("{GENERATED_TABLE}.{base}"))
        ));
    }

    // Keyed by the members the catalogue declares rather than by the strings themselves: the
    // spelling of a problem lives in one place, and a value the document stopped naming takes the
    // entry with it instead of leaving one nothing can ever match.
    source.push_str(&format!(
        "\n--- The failure each problem the API names is raised as.\n\
         {GENERATED_TABLE}.{PROBLEMS_MEMBER} = {{\n"
    ));
    for (value, declared) in &types.problems {
        let member = ident(value, language.casing.constant, language, limits)?;
        source.push_str(&format!(
            "  [{MODELS_TABLE}.{catalogue}.{member}] = {GENERATED_TABLE}.{declared},\n"
        ));
    }
    source.push_str("}\n");

    // The discriminant is what says whether a body named a problem at all, and a document may
    // declare it as a member it does not require, so what it carries is looked up rather than
    // trusted to name an entry of the catalogue.
    source.push_str(&format!(
        "\n--- Raise what the API reported, when what it answered was not a success.\n\
         --- @param status integer what the API answered under\n\
         --- @param payload string the body it answered\n\
         --- @return nil\n\
         function {GENERATED_TABLE}.{RAISE_MEMBER}(status, payload)\n  \
         if status >= {LOWEST_SUCCESS} and status < {LOWEST_REDIRECTION} then\n    \
         return\n  \
         end\n\n  \
         local problem = {RUNTIME_TABLE}.problem_of({MODELS_TABLE}.{schema}, payload)\n  \
         if problem == nil or {GENERATED_TABLE}.{PROBLEMS_MEMBER}[problem.{discriminant}] == nil then\n    \
         {ERRORS_TABLE}.raise(\n      \
         {GENERATED_TABLE}.{base},\n      \
         {RUNTIME_TABLE}.unreadable(status, payload),\n      \
         status,\n      \
         problem\n    \
         )\n  \
         end\n\n  \
         {ERRORS_TABLE}.raise(\n    \
         {GENERATED_TABLE}.{PROBLEMS_MEMBER}[problem.{discriminant}],\n    \
         {RUNTIME_TABLE}.reported(status, problem),\n    \
         status,\n    \
         problem\n  \
         )\n\
         end\n"
    ));

    Ok(format!(
        "local {ERRORS_TABLE} = require({})\n\
         local {MODELS_TABLE} = require({})\n\
         local {RUNTIME_TABLE} = require({})\n\n\
         --- The failures the API reports, one kind per problem it can name.\n\
         local {GENERATED_TABLE} = {{}}\n\n\
         {source}\n\
         return {GENERATED_TABLE}\n",
        literal(ERRORS_MODULE),
        literal(&generated_module(MODELS_FILE)),
        literal(RUNTIME_MODULE),
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

    for entity in model.entities.entities() {
        body.push('\n');
        body.push_str(&group(entity, types, language, limits)?);
    }

    Ok(format!(
        "local {GENERATED_TABLE} = require({})\n\
         local {MODELS_TABLE} = require({})\n\
         local {RUNTIME_TABLE} = require({})\n\n\
         --- One table per entity the API declares, one method per operation it groups under it.\n\
         local {API_TABLE} = {{}}\n\n\
         --- Raise what the API reported, and answer nothing when it reported nothing.\n\
         --- @param status integer what the API answered under\n\
         --- @param payload string the body it answered\n\
         --- @return nil\n\
         local function {CHECK_HELPER}(status, payload)\n  \
         {GENERATED_TABLE}.{RAISE_MEMBER}(status, payload)\n\
         end\n\n\
         --- Raise what the API reported, or read back the value it answered.\n\
         --- @param reader function what turns that body into the value the API declares\n\
         --- @param status integer what the API answered under\n\
         --- @param payload string the body it answered\n\
         --- @return any\n\
         local function {READ_HELPER}(reader, status, payload)\n  \
         {GENERATED_TABLE}.{RAISE_MEMBER}(status, payload)\n  \
         return reader({RUNTIME_TABLE}.decode_payload(payload))\n\
         end\n{body}\n\
         return {API_TABLE}\n",
        literal(&generated_module(ERRORS_FILE)),
        literal(&generated_module(MODELS_FILE)),
        literal(RUNTIME_MODULE),
    ))
}

fn group(
    entity: &Entity,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let declared = types.group(&entity.name)?;
    let held = format!("{API_TABLE}.{declared}");

    let mut source = format!(
        "--- What the API declares under `{}`, issued through the transport it is handed.\n\
         {held} = {{}}\n\
         {held}.__index = {held}\n\n\
         --- @param transport table what one request is issued through\n\
         --- @return {declared}\n\
         function {held}.new(transport)\n  \
         return setmetatable({{ transport = transport }}, {held})\n\
         end\n",
        comment(&entity.name)
    );

    for method in &entity.methods {
        source.push('\n');
        source.push_str(&operation(method, &held, types, language, limits)?);
    }

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
    held: &str,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let name = ident(&method.verb_text, language.casing.method, language, limits)?;
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

    let mut source = folded("", &summary(method));
    for argument in &required {
        source.push_str(&folded(
            "",
            &format!(
                "@param {} {} carries `{}`{}",
                argument.name,
                argument.annotated,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    if let (Some(body), Some(shape)) = (body_argument.as_deref(), method.request.as_ref()) {
        source.push_str(&folded(
            "",
            &format!(
                "@param {body} {} what the operation reads",
                annotation(shape, true, types)?
            ),
        ));
    }
    for argument in &optional {
        source.push_str(&folded(
            "",
            &format!(
                "@param {} {}|nil carries `{}`{}",
                argument.name,
                argument.annotated,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    source.push_str(&format!(
        "--- @return {}\n",
        returned.clone().unwrap_or_else(|| "nil".to_owned())
    ));

    let mut declared = Vec::new();
    for argument in &required {
        declared.push(argument.name.clone());
    }
    if let Some(body) = body_argument.as_deref() {
        declared.push(body.to_owned());
    }
    for argument in &optional {
        declared.push(argument.name.clone());
    }
    source.push_str(&arguments(&format!("{held}:{name}"), &declared));

    let issued = format!(
        "self.transport:request(\n      {},\n{}{}      {}\n    )",
        literal(operation.method.as_str()),
        path(operation.path.as_str(), &required),
        query(&required, &optional),
        match method.request.as_ref() {
            None => "nil".to_owned(),
            Some(shape) => writer(shape, body_argument.as_deref().unwrap_or(BODY_ARGUMENT), 0)?,
        }
    );

    match method.success.as_ref() {
        Some((_, Some(shape))) => {
            source.push_str(&format!(
                "  return {READ_HELPER}(\n    {},\n    {issued}\n  )\n",
                reader(shape, types, 0)?
            ));
        }
        _ => source.push_str(&format!("  return {CHECK_HELPER}(\n    {issued}\n  )\n")),
    }
    source.push_str("end\n");

    Ok(source)
}

/// Refuses two arguments of one method that would be spelled the same way.
///
/// Lua has one namespace for the arguments of a method, so a path parameter and a query parameter
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
        return format!("      {},\n", literal(template));
    }

    let mut source = format!("      {RUNTIME_TABLE}.path({}, {{\n", literal(template));
    for argument in &filled {
        source.push_str(&format!(
            "        [{}] = {},\n",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str("      }),\n");
    source
}

/// What travels in the query string, as one expression.
///
/// Every parameter the operation declares is listed; the runtime is what leaves out the ones the
/// caller passed nothing for, so what the emitted method says does not depend on what it was
/// called with.
fn query(required: &[Argument<'_>], optional: &[Argument<'_>]) -> String {
    let asked: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
        .chain(optional.iter())
        .collect();

    if asked.is_empty() {
        return "      nil,\n".to_owned();
    }

    let mut source = format!("      {RUNTIME_TABLE}.query({{\n");
    for argument in &asked {
        source.push_str(&format!(
            "        {{ {}, {} }},\n",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str("      }),\n");
    source
}

/// The type a value of that shape carries, as a documentation comment names one.
///
/// Lua says nothing about types in the source itself, so this is what a reader — and every tool
/// that reads documentation — is told. Optionality is membership in `required` and nothing else.
fn annotation(shape: &Shape, required: bool, types: &Types) -> Result<String, Error> {
    let declared = annotated(shape, types, 0)?;
    if required {
        return Ok(declared);
    }
    Ok(format!("{declared}|nil"))
}

fn annotated(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => types.scalars.of(scalar).to_owned(),
        Shape::Array(inner) => format!("{}[]", annotated(inner, types, depth + 1)?),
        Shape::Map(inner) => {
            format!("table<string, {}>", annotated(inner, types, depth + 1)?)
        }
        // A value of a closed list travels as the string the API answers, and that is what it is;
        // which strings it may be is said beside it, by naming the table that declares them.
        Shape::Enum { .. } => "string".to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => "any".to_owned(),
    })
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "string",
        "integer" => "integer",
        "number" => "number",
        "boolean" => "boolean",
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
        Shape::Scalar(scalar) => format!("{RUNTIME_TABLE}.{}", scalar_reader(scalar)),
        Shape::Array(inner) => {
            format!("{RUNTIME_TABLE}.list({})", reader(inner, types, depth + 1)?)
        }
        Shape::Map(inner) => format!("{RUNTIME_TABLE}.map({})", reader(inner, types, depth + 1)?),
        Shape::Enum { name, .. } => format!(
            "{RUNTIME_TABLE}.member_of({MODELS_TABLE}.{})",
            types.enumeration(name)?
        ),
        Shape::Named(name) => format!("{MODELS_TABLE}.{}.from_json", types.schema(name)?),
        Shape::Object(object) => {
            format!("{MODELS_TABLE}.{}.from_json", types.schema(&object.name)?)
        }
        Shape::Json => format!("{RUNTIME_TABLE}.JSON_VALUE"),
    })
}

fn scalar_reader(scalar: &Scalar) -> &'static str {
    match scalar {
        // Every one of these travels as the text the API answered: Lua carries no type for an
        // identifier, a moment or a day, and text is what has to go back out unchanged.
        Scalar::String | Scalar::Url | Scalar::Uuid | Scalar::DateTime | Scalar::Date => "TEXT",
        Scalar::Integer32 | Scalar::Integer64 => "INTEGER",
        Scalar::Number => "NUMBER",
        Scalar::Boolean => "BOOLEAN",
    }
}

/// What writes `subject` back the way the API reads it.
fn writer(shape: &Shape, subject: &str, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(_) | Shape::Enum { .. } | Shape::Json => subject.to_owned(),
        Shape::Named(_) | Shape::Object(_) => format!("{RUNTIME_TABLE}.written({subject})"),
        // A list and a map are rebuilt whatever they hold: a table a caller assembled by hand says
        // nothing about whether it is one or the other, and the runtime is what marks it so the
        // document that goes out is the one the API reads.
        Shape::Array(inner) => {
            let item = format!("item{depth}");
            format!(
                "{RUNTIME_TABLE}.written_list({subject}, {})",
                written_with(inner, &item, depth)?
            )
        }
        Shape::Map(inner) => {
            let value = format!("value{depth}");
            format!(
                "{RUNTIME_TABLE}.written_map({subject}, {})",
                written_with(inner, &value, depth)?
            )
        }
    })
}

/// How the items of a list or the values of a map are written back.
///
/// A value that travels as it stands, and one written by the same helper every emitted type is
/// written by, are each named rather than wrapped in a function that does nothing else: both are
/// what a Lua linter asks for, and neither is something a pass over the emitted source should have
/// to work out afterwards.
fn written_with(shape: &Shape, subject: &str, depth: usize) -> Result<String, Error> {
    let written = writer(shape, subject, depth + 1)?;

    if written == subject {
        return Ok(format!("{RUNTIME_TABLE}.itself"));
    }
    if written == format!("{RUNTIME_TABLE}.written({subject})") {
        return Ok(format!("{RUNTIME_TABLE}.written"));
    }
    Ok(format!("function({subject}) return {written} end"))
}

/// What the rock requires to reach everything the generator writes.
fn index() -> String {
    format!(
        "--- Everything the generator writes, reached through one require.\n\
         return {{\n  \
         api = require({}),\n  \
         errors = require({}),\n  \
         models = require({}),\n\
         }}\n",
        literal(&generated_module(API_FILE)),
        literal(&generated_module(ERRORS_FILE)),
        literal(&generated_module(MODELS_FILE)),
    )
}

/// How one generated file is required, once the rock is installed.
fn generated_module(stem: &str) -> String {
    format!("{GENERATED_PREFIX}.{stem}")
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
/// Lua reads no interpolation inside a quoted string, so only the quote, the escape itself and what
/// is not text have to be spelled out. A control character is spelled as the code point it is
/// rather than as the byte it would be written as, so that a literal never carries a byte that is
/// not text — and a code point above the first block is one the decimal escape could not hold.
fn literal(text: &str) -> String {
    let mut rendered = String::from("\"");

    for character in text.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            control if control.is_control() => {
                rendered.push_str(&format!("\\u{{{:04x}}}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }

    rendered.push('"');
    rendered
}
