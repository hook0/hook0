//! Emits the generated half of the Zig SDK.
//!
//! The package is published with no copy of the OpenAPI snapshot beside it, so the types, the
//! problems and the request layer travel as committed source rather than as a build artefact.
//! Everything the API declares — one struct per named schema, one namespace of constants per closed
//! list of strings, one error per problem the error contract can report, one method per operation —
//! is written here; everything the API does not declare — how a request reaches the network, how a
//! send is retried, how a webhook signature is verified — is hand-written beside this directory and
//! never regenerated.
//!
//! The two halves meet at two seams and nowhere else. The generated code reads its decoding helpers
//! from the hand-written runtime beside it, and it issues every request through a
//! `runtime.Transport`, which is a pointer and a function: nothing here knows what a socket is, and
//! nothing beside it knows what the API declares.
//!
//! Allocation is explicit, and that is what makes this target different from the other nine. Every
//! method takes an allocator and answers a `runtime.Owned(T)`, which is the value together with the
//! arena everything it points into was allocated from: one `deinit` frees the body that arrived, the
//! document it was parsed into and every slice read out of it. A method that answers nothing takes
//! the allocator all the same and frees its arena on the way out. Nothing is reachable after its
//! arena is gone, and nothing outlives it by accident.
//!
//! One thing outlives it on purpose: what a failure reported. A Zig error carries no payload, so the
//! status, the problem document and the message of a failed call are written into the group that
//! made it — which is read after the call has returned and everything the call allocated is gone.
//! That is why a group is built with an allocator of its own and freed with `deinit`: what a failure
//! said is read into an arena taken from there, the next failure frees the one before it, and the
//! last one is freed with the group.
//!
//! What is written is already formatted. `zig fmt` is not run over the output — the emitter writes
//! what `zig fmt` would write, and the pipeline checks that with `zig fmt --check`, so a difference
//! is a defect in this file rather than something a pass downstream tidies up.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, two arguments of one method spelled alike, a
//! scalar no type covers — stops the emission rather than yielding a smaller SDK.

use std::collections::BTreeMap;

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, spell};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Scalar, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{Contract, Decoding, LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "zig";

/// Where the generated half of the package lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written. Both halves
/// are one Zig module rooted at `src`, which is what lets a generated file reach the runtime above
/// it by relative path.
const ROOT: &str = "clients/zig/src/generated";

/// File the generated code reads its decoding helpers from, written as the path a file of this
/// directory reaches it by.
const RUNTIME_IMPORT: &str = "../runtime.zig";

/// What the generated code calls the runtime it imports.
const RUNTIME: &str = "runtime";

/// The files this target writes, each one holding one layer of the surface.
const MODELS_FILE: &str = "models";
const ERRORS_FILE: &str = "errors";
const API_FILE: &str = "api";

/// File reaching every other one, so that a caller of the package names one import whatever the API
/// grows. Its name never changes, which is what keeps the hand-written half from having to be edited
/// when a layer is added.
const INDEX_FILE: &str = "root";

/// What the generated code calls each of its own files when it imports one.
const MODELS: &str = "models";
const ERRORS: &str = "errors";

/// Declaration naming the error each problem is answered as.
const PROBLEMS_DECLARATION: &str = "problems";

/// Declaration every closed list of strings carries beside its values.
const VALUES_DECLARATION: &str = "values";

/// Function raising whatever the API reported, when what it answered was not a success.
const RAISE_FUNCTION: &str = "raiseForStatus";

/// The error set every problem the API can report is one of.
///
/// Not `Problem`: the document declares a schema of that name, and a target whose scaffolding took
/// it would be one where the type the API answers and the errors it reports could not both be
/// named.
const PROBLEM_SET: &str = "Failure";

/// What one failure of the API said about itself, beyond which error it is.
const REPORTED_TYPE: &str = "Reported";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// What an emitted group holds the transport it issues requests through under.
const TRANSPORT_FIELD: &str = "transport";

/// What an emitted group holds the last failure the API reported under.
const REPORTED_FIELD: &str = "reported";

/// What an emitted group holds the allocator that failure is read into under.
const ALLOCATOR_FIELD: &str = "allocator";

/// What the argument carrying the body an operation reads is called.
const BODY_ARGUMENT: &str = "body";

/// What the allocator every method takes is called.
const ALLOCATOR_ARGUMENT: &str = "allocator";

/// Longest fragment of a snapshot description a comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Longest line the emitted source carries.
///
/// `zig fmt` does not wrap a line for being long, so this is the emitter's own margin rather than
/// the formatter's: a description the document writes as one paragraph is folded across as many
/// comment lines as it takes, and a call that would not fit is written one argument per line.
const MAX_LINE_CHARS: usize = 100;

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// The names the scaffolding around the emitted code answers to, which no type of the document may
/// answer to as well.
const SCAFFOLDING: [&str; 6] = [
    ERRORS,
    MODELS,
    PROBLEMS_DECLARATION,
    PROBLEM_SET,
    REPORTED_TYPE,
    RUNTIME,
];

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan nothing imports.
        ownership: Ownership::Directory,
        contract: Contract::Whole,
        decoding: Decoding::Modelled,
        language: super::zig(),
        emit,
    }
}

/// Everything the generated half of the package is made of.
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
/// rather than as a declaration the compiler refuses halfway through a build.
struct Types {
    /// Struct each named schema is declared as, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Namespace each closed list of strings is declared as.
    enums: BTreeMap<String, String>,
    /// Error each problem is answered as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Namespace the discriminant of the error contract is read through.
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

        // An error of a Zig error set is named the way a type is, and the set itself is what tells
        // one problem apart from a type spelling the same word, so no suffix is needed here.
        let mut problems = BTreeMap::new();
        for value in model.errors.catalogue.values() {
            let name = ident(value, language.casing.type_name, language, limits)?;
            problems.insert(value.clone(), name);
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
        "const std = @import(\"std\");\n\n\
         const {RUNTIME} = @import(\"{RUNTIME_IMPORT}\");\n\n\
         /// This file, which is the namespace every type below is reached through.\n\
         ///\n\
         /// A Zig file is a struct, so naming it here is what lets one type name another declared\n\
         /// further down without the order of declarations mattering — and it is the same spelling\n\
         /// the other two generated files use, which is what keeps one reader for all three.\n\
         const {MODELS} = @This();\n{body}"
    ))
}

/// One closed list of strings, as a namespace holding one constant per value it admits.
///
/// The values travel as the strings the API answers, so nothing has to be mapped on the way in or on
/// the way out; the namespace is what names them and what says which ones there are.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    // The declaration listing the values is claimed first, so a value spelling it is reported rather
    // than declared twice.
    let mut members: BTreeMap<String, &str> = BTreeMap::new();
    members.insert(
        VALUES_DECLARATION.to_owned(),
        "the list this namespace declares",
    );

    let mut declarations = String::new();
    let mut listed = Vec::new();

    for value in values {
        let member = ident(value, language.casing.constant, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        declarations.push_str(&format!(
            "    pub const {member}: []const u8 = {};\n",
            literal(value)
        ));
        listed.push(member);
    }

    Ok(format!(
        "/// One of the values the API answers with.\n\
         pub const {declared} = struct {{\n{declarations}\n    \
         /// Every value the API declares for this list.\n    \
         pub const {VALUES_DECLARATION} = [_][]const u8{}\n\n    \
         /// Whether the API declares that value.\n    \
         pub fn member(value: []const u8) bool {{\n        \
         return {RUNTIME}.declares(&{VALUES_DECLARATION}, value);\n    \
         }}\n\
         }};\n",
        braced("    ", &listed, ";")
    ))
}

/// A list of values, on one line when it fits and one value per line when it does not.
fn braced(indent: &str, values: &[String], trailing: &str) -> String {
    let single = format!("{indent}{{ {} }}{trailing}", values.join(", "));
    if values.is_empty() {
        return format!("{{}}{trailing}");
    }
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{{ {} }}{trailing}", values.join(", "));
    }

    let mut source = String::from("{\n");
    for value in values {
        source.push_str(&format!("{indent}    {value},\n"));
    }
    source.push_str(&format!("{indent}}}{trailing}"));
    source
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
        "/// The `{}` the API declares.\n\
         pub const {declared} = struct {{\n",
        comment(&object.name)
    );

    for (name, field) in &named {
        source.push_str(&folded(
            "    ",
            &format!(
                "carries `{}`{}{}",
                comment(&field.name),
                listed(&field.shape, types)?,
                described(field.description.as_deref())
            ),
        ));
        source.push_str(&format!(
            "    {name}: {},\n",
            annotation(&field.shape, field.required, types)?
        ));
    }
    if named.is_empty() {
        // A schema declaring no member at all is still a type, and Zig writes an empty one this way.
        source.push('\n');
    }

    source.push_str(&format!(
        "\n    /// Read one out of what the API answered.\n    \
         pub fn fromJson(\n        \
         {ALLOCATOR_ARGUMENT}: std.mem.Allocator,\n        \
         value: std.json.Value,\n    \
         ) {RUNTIME}.DecodeError!{declared} {{\n        \
         const fields = try {RUNTIME}.asFields(value, {});\n        \
         return .{{\n",
        literal(&object.name)
    ));
    for (name, field) in &named {
        let called = format!(
            "{RUNTIME}.{}",
            if field.required { "read" } else { "maybe" }
        );
        let arguments = [
            ALLOCATOR_ARGUMENT.to_owned(),
            "fields".to_owned(),
            literal(&field.name),
            reader(&field.shape, types, 0)?,
        ];
        source.push_str(&call(
            "            ",
            &format!(".{name} = try {called}"),
            &arguments,
            ",",
        ));
    }
    source.push_str("        };\n    }\n");

    source.push_str(&format!(
        "\n    /// Write one back the way the API reads it.\n    \
         pub fn toJson(\n        \
         self: {declared},\n        \
         {ALLOCATOR_ARGUMENT}: std.mem.Allocator,\n    \
         ) {RUNTIME}.WriteError!std.json.Value {{\n        \
         var out: std.json.ObjectMap = .empty;\n"
    ));
    for (name, field) in &named {
        let arguments = [
            "&out".to_owned(),
            ALLOCATOR_ARGUMENT.to_owned(),
            literal(&field.name),
            format!("self.{name}"),
        ];
        source.push_str(&call(
            "        ",
            &format!("try {RUNTIME}.put"),
            &arguments,
            ";",
        ));
    }
    source.push_str("        return .{ .object = out };\n    }\n};\n");

    Ok(source)
}

/// Fields in the one order a struct declares them: what the document requires, then what it does
/// not, so that reading one says what always arrives before what may not.
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
fn folded(indent: &str, text: &str) -> String {
    let opening = format!("{indent}/// ");

    let mut lines = String::new();
    let mut line = opening.clone();
    let mut empty = true;

    for word in text.split(' ').filter(|word| !word.is_empty()) {
        if !empty && line.chars().count() + 1 + word.chars().count() > MAX_LINE_CHARS {
            lines.push_str(&line);
            lines.push('\n');
            line = opening.clone();
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
/// Both shapes are what `zig fmt` writes: it keeps a call on one line when the source has it on one
/// line and it fits, and writes one argument per line with a trailing comma otherwise.
fn call(indent: &str, called: &str, arguments: &[String], trailing: &str) -> String {
    let single = format!("{indent}{called}({}){trailing}", arguments.join(", "));
    if single.chars().count() <= MAX_LINE_CHARS && !single.contains('\n') {
        return format!("{single}\n");
    }

    let mut source = format!("{indent}{called}(\n");
    for argument in arguments {
        source.push_str(&format!("{indent}    {argument},\n"));
    }
    source.push_str(&format!("{indent}){trailing}\n"));
    source
}

/// Which closed list a member is drawn from, when it is drawn from one.
fn listed(shape: &Shape, types: &Types) -> Result<String, Error> {
    let Shape::Enum { name, .. } = shape else {
        return Ok(String::new());
    };
    Ok(format!(
        ", one of `{MODELS}.{}.{VALUES_DECLARATION}`",
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

/// The problems the API reports, one error each.
fn errors(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    let discriminant = ident(&model.errors.discriminant, Case::Snake, language, limits)?;
    let catalogue = &types.problem_enum;

    let mut source = format!(
        "/// Every problem the API can report, one error each.\n\
         ///\n\
         /// A Zig error carries no value, so what the API said beyond which problem it was — the\n\
         /// status it answered under and the document it answered — is written into the\n\
         /// [{REPORTED_TYPE}] the caller handed in. That is the same separation Go's clients make\n\
         /// between the error and the value carrying it, rather than the exception hierarchy the\n\
         /// languages with one use.\n\
         pub const {PROBLEM_SET} = error{{\n"
    );
    for declared in types.problems.values() {
        source.push_str(&format!("    {declared},\n"));
    }
    source.push_str(
        "    /// The API answered a failure whose body names no problem this client knows.\n    \
         Unreadable,\n\
         };\n",
    );

    source.push_str(&format!(
        "\n/// What one failure of the API said about itself, beyond which error it is.\n\
         ///\n\
         /// Everything it points into is held in an arena of its own, which is what lets it be read\n\
         /// after the call that drew it has freed everything that call allocated. The next failure\n\
         /// written here frees what the last one held, and `deinit` frees the last one of all.\n\
         pub const {REPORTED_TYPE} = struct {{\n    \
         /// What the API answered under.\n    \
         status: u16 = 0,\n    \
         /// The problem document it answered, when this client could read one.\n    \
         problem: ?{MODELS}.{schema} = null,\n    \
         /// What to say about the failure.\n    \
         detail: []const u8 = \"\",\n    \
         /// Where `problem` and `detail` were read into, which nothing else refers to.\n    \
         held: ?{RUNTIME}.Kept = null,\n\n    \
         /// What a group carries before anything has failed.\n    \
         pub const empty: {REPORTED_TYPE} = .{{}};\n\n    \
         /// Frees what the last failure reported, leaving this as it started.\n    \
         pub fn deinit(self: *{REPORTED_TYPE}) void {{\n        \
         if (self.held) |held| held.deinit();\n        \
         self.* = .empty;\n    \
         }}\n\
         }};\n"
    ));

    // Keyed by the constants the catalogue declares rather than by the strings themselves: the
    // spelling of a problem lives in one place, and a value the document stopped naming takes the
    // entry with it instead of leaving one nothing can ever match.
    source.push_str(&format!(
        "\n/// The error each problem the API names is answered as.\n\
         pub const {PROBLEMS_DECLARATION} = [_]struct {{ id: []const u8, raised: {PROBLEM_SET} }}{{\n"
    ));
    for (value, declared) in &types.problems {
        let member = ident(value, language.casing.constant, language, limits)?;
        source.push_str(&format!(
            "    .{{ .id = {MODELS}.{catalogue}.{member}, .raised = error.{declared} }},\n"
        ));
    }
    source.push_str("};\n");

    // The discriminant is what says whether a body named a problem at all, and a document may
    // declare it as a member it does not require, so what it carries is looked up rather than
    // trusted to name an entry of the catalogue.
    source.push_str(&format!(
        "\n/// The error a problem document names, when what it names is one this client knows.\n\
         fn raisedBy(problem: ?{MODELS}.{schema}) ?{PROBLEM_SET} {{\n    \
         const named = problem orelse return null;\n    \
         for ({PROBLEMS_DECLARATION}) |entry| {{\n        \
         if (std.mem.eql(u8, entry.id, named.{discriminant})) return entry.raised;\n    \
         }}\n    \
         return null;\n\
         }}\n"
    ));

    // The body is copied before it is read rather than read where it lies: what a document is
    // parsed into points into the bytes it was parsed from, and those bytes belong to the call,
    // which frees them on its way out.
    source.push_str(&format!(
        "\n/// What one failure said about itself, read into an arena of its own.\n\
         ///\n\
         /// Everything the answer points into — the copy of the body, the document read out of it\n\
         /// and the message written about it — is in that arena, so what the caller allocated for\n\
         /// the request it drew is none of it.\n\
         fn reportedOf(\n    \
         {ALLOCATOR_ARGUMENT}: std.mem.Allocator,\n    \
         status: u16,\n    \
         payload: []const u8,\n\
         ) std.mem.Allocator.Error!{REPORTED_TYPE} {{\n    \
         const held: {RUNTIME}.Kept = try .init({ALLOCATOR_ARGUMENT});\n    \
         errdefer held.deinit();\n\n    \
         const into = held.arena.allocator();\n    \
         const carried = try into.dupe(u8, {RUNTIME}.retained(payload));\n    \
         const problem = {RUNTIME}.problemOf({MODELS}.{schema}, into, carried);\n    \
         const detail = if (raisedBy(problem) != null)\n        \
         try {RUNTIME}.reported(into, status, carried)\n    \
         else\n        \
         try {RUNTIME}.unreadable(into, status, carried);\n\n    \
         return .{{ .status = status, .problem = problem, .detail = detail, .held = held }};\n\
         }}\n"
    ));

    source.push_str(&format!(
        "\n/// Answer what the API reported, when what it answered was not a success.\n\
         ///\n\
         /// What the failure said about itself is written into `{REPORTED_FIELD}`, which is what a\n\
         /// caller reads once the error has told it there is something to read. The allocator is\n\
         /// the one that memory is taken from, and it has to be one outliving the call: what the\n\
         /// call itself allocated is gone by the time the caller reads any of this.\n\
         pub fn {RAISE_FUNCTION}(\n    \
         {ALLOCATOR_ARGUMENT}: std.mem.Allocator,\n    \
         status: u16,\n    \
         payload: []const u8,\n    \
         {REPORTED_FIELD}: *{REPORTED_TYPE},\n\
         ) ({PROBLEM_SET} || std.mem.Allocator.Error)!void {{\n    \
         if (status >= {LOWEST_SUCCESS} and status < {LOWEST_REDIRECTION}) return;\n\n    \
         const read = try reportedOf({ALLOCATOR_ARGUMENT}, status, payload);\n    \
         {REPORTED_FIELD}.deinit();\n    \
         {REPORTED_FIELD}.* = read;\n\n    \
         return raisedBy(read.problem) orelse error.Unreadable;\n\
         }}\n"
    ));

    Ok(format!(
        "const std = @import(\"std\");\n\n\
         const {RUNTIME} = @import(\"{RUNTIME_IMPORT}\");\n\
         const {MODELS} = @import(\"{MODELS_FILE}.zig\");\n\n\
         {source}"
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
        "const std = @import(\"std\");\n\n\
         const {RUNTIME} = @import(\"{RUNTIME_IMPORT}\");\n\
         const {ERRORS} = @import(\"{ERRORS_FILE}.zig\");\n\
         const {MODELS} = @import(\"{MODELS_FILE}.zig\");\n{body}"
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
        "/// What the API declares under `{}`, issued through the transport it is built on.\n\
         pub const {declared} = struct {{\n    \
         /// What one request is issued through.\n    \
         {TRANSPORT_FIELD}: {RUNTIME}.Transport,\n    \
         /// Where what a failure of this group reported is read into.\n    \
         ///\n    \
         /// Not the allocator a call is handed: that one frees what the call allocated on its way\n    \
         /// out, and what the failure reported is read after the call has returned.\n    \
         {ALLOCATOR_FIELD}: std.mem.Allocator,\n    \
         /// What the last failure of this group reported, which an error alone cannot carry.\n    \
         {REPORTED_FIELD}: {ERRORS}.{REPORTED_TYPE} = .empty,\n\n\
         {}        \
         return .{{ \
         .{ALLOCATOR_FIELD} = {ALLOCATOR_FIELD}, \
         .{TRANSPORT_FIELD} = {TRANSPORT_FIELD} \
         }};\n    \
         }}\n\n    \
         /// Frees what the last failure of this group reported.\n    \
         pub fn deinit(self: *{declared}) void {{\n        \
         self.{REPORTED_FIELD}.deinit();\n    \
         }}\n",
        comment(&entity.name),
        signature(
            "    ",
            "init",
            &[
                format!("{ALLOCATOR_FIELD}: std.mem.Allocator"),
                format!("{TRANSPORT_FIELD}: {RUNTIME}.Transport"),
            ],
            declared,
        )
    );

    for method in &entity.methods {
        source.push('\n');
        source.push_str(&operation(method, declared, types, language, limits)?);
    }

    source.push_str("};\n");
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
    group: &str,
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

    let answered = match method.success.as_ref() {
        Some((_, Some(shape))) => Some(annotation(shape, true, types)?),
        _ => None,
    };

    let mut source = folded("    ", &summary(method));
    for argument in required.iter().chain(optional.iter()) {
        source.push_str(&folded(
            "    ",
            &format!(
                "`{}` carries `{}`{}",
                argument.name,
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }

    // Every argument on a line of its own, whatever the count: an operation the document grows a
    // parameter for is one line of diff rather than a rewritten signature.
    let mut declared = vec![format!("self: *{group}")];
    declared.push(format!("{ALLOCATOR_ARGUMENT}: std.mem.Allocator"));
    for argument in &required {
        declared.push(format!("{}: {}", argument.name, argument.annotated));
    }
    if let (Some(body), Some(shape)) = (body_argument.as_deref(), method.request.as_ref()) {
        declared.push(format!("{body}: {}", annotation(shape, true, types)?));
    }
    for argument in &optional {
        declared.push(format!("{}: ?{}", argument.name, argument.annotated));
    }

    let returns = match answered.as_deref() {
        Some(answered) => format!("!{RUNTIME}.Owned({answered})"),
        None => "!void".to_owned(),
    };
    source.push_str(&signature("    ", &name, &declared, &returns));

    let issued = format!(
        "        const answered = try self.{TRANSPORT_FIELD}.request(arena, .{{\n            \
         .method = {},\n{}{}{}        }});\n",
        literal(operation.method.as_str()),
        path(operation.path.as_str(), &required),
        query(&required, &optional),
        match method.request.as_ref() {
            None => String::new(),
            Some(_) => format!(
                "            .body = try {RUNTIME}.written(arena, {}),\n",
                body_argument.as_deref().unwrap_or(BODY_ARGUMENT)
            ),
        }
    );
    let raised = call(
        "        ",
        &format!("try {ERRORS}.{RAISE_FUNCTION}"),
        &[
            format!("self.{ALLOCATOR_FIELD}"),
            "answered.status".to_owned(),
            "answered.payload".to_owned(),
            format!("&self.{REPORTED_FIELD}"),
        ],
        ";",
    );

    match answered.as_deref() {
        Some(answered_type) => {
            source.push_str(&format!(
                "        var owned: {RUNTIME}.Owned({answered_type}) = \
                 try .init({ALLOCATOR_ARGUMENT});\n        \
                 errdefer owned.deinit();\n        \
                 const arena = owned.arena.allocator();\n\n\
                 {issued}{raised}\n        \
                 owned.value = try {};\n        \
                 return owned;\n    \
                 }}\n",
                read_back(
                    method
                        .success
                        .as_ref()
                        .and_then(|(_, shape)| shape.as_ref())
                        .expect("a method answering a type carries its shape"),
                    types
                )?
            ));
        }
        None => {
            source.push_str(&format!(
                "        var held: std.heap.ArenaAllocator = .init({ALLOCATOR_ARGUMENT});\n        \
                 defer held.deinit();\n        \
                 const arena = held.allocator();\n\n\
                 {issued}{raised}    \
                 }}\n"
            ));
        }
    }

    Ok(source)
}

/// One signature, with every argument on a line of its own.
fn signature(indent: &str, name: &str, declared: &[String], returns: &str) -> String {
    let mut source = format!("{indent}pub fn {name}(\n");
    for argument in declared {
        source.push_str(&format!("{indent}    {argument},\n"));
    }
    source.push_str(&format!("{indent}) {returns} {{\n"));
    source
}

/// What reads the value an operation answered back out of the body that carried it.
fn read_back(shape: &Shape, types: &Types) -> Result<String, Error> {
    Ok(format!(
        "{}(arena, try {RUNTIME}.decodePayload(arena, answered.payload))",
        reader(shape, types, 0)?
    ))
}

/// Refuses two arguments of one method that would be spelled the same way.
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

/// Where the request lands, as one member of the request the transport is handed.
fn path(template: &str, required: &[Argument<'_>]) -> String {
    let filled: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Path)
        .collect();

    if filled.is_empty() {
        return format!("            .path = {},\n", literal(template));
    }

    let mut source = format!(
        "            .path = try {RUNTIME}.path(arena, {}, &.{{\n",
        literal(template)
    );
    for argument in &filled {
        source.push_str(&format!(
            "                .{{ .name = {}, .value = {RUNTIME}.value({}) }},\n",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str("            }),\n");
    source
}

/// What travels in the query string, as one member of the request the transport is handed.
///
/// Every parameter the operation declares is listed; the transport is what leaves out the ones the
/// caller passed nothing for, so what the emitted method says does not depend on what it was called
/// with.
fn query(required: &[Argument<'_>], optional: &[Argument<'_>]) -> String {
    let asked: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
        .chain(optional.iter())
        .collect();

    if asked.is_empty() {
        return String::new();
    }

    let mut source = String::from("            .query = &.{\n");
    for argument in &asked {
        source.push_str(&format!(
            "                .{{ .name = {}, .value = {RUNTIME}.value({}) }},\n",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str("            },\n");
    source
}

/// The type a value of that shape carries.
fn annotation(shape: &Shape, required: bool, types: &Types) -> Result<String, Error> {
    let declared = annotated(shape, types, 0)?;
    if required {
        return Ok(declared);
    }
    Ok(format!("?{declared}"))
}

fn annotated(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => types.scalars.of(scalar).to_owned(),
        Shape::Array(inner) => format!("[]const {}", annotated(inner, types, depth + 1)?),
        Shape::Map(inner) => format!("{RUNTIME}.Map({})", annotated(inner, types, depth + 1)?),
        // A value of a closed list travels as the string the API answers, and that is what it is;
        // which strings it may be is said beside it, by naming the namespace that declares them.
        Shape::Enum { .. } => "[]const u8".to_owned(),
        Shape::Named(name) => format!("{MODELS}.{}", types.schema(name)?),
        Shape::Object(object) => format!("{MODELS}.{}", types.schema(&object.name)?),
        Shape::Json => "std.json.Value".to_owned(),
    })
}

/// The type a parameter travelling in a path or a query carries.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "[]const u8",
        "integer" => "i64",
        "number" => "f64",
        "boolean" => "bool",
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
        Shape::Scalar(scalar) => format!("{RUNTIME}.{}", scalar_reader(scalar)),
        Shape::Array(inner) => format!("{RUNTIME}.list({}).read", reader(inner, types, depth + 1)?),
        Shape::Map(inner) => format!("{RUNTIME}.map({}).read", reader(inner, types, depth + 1)?),
        Shape::Enum { name, .. } => format!(
            "{RUNTIME}.memberOf({MODELS}.{}).read",
            types.enumeration(name)?
        ),
        Shape::Named(name) => format!("{MODELS}.{}.fromJson", types.schema(name)?),
        Shape::Object(object) => format!("{MODELS}.{}.fromJson", types.schema(&object.name)?),
        Shape::Json => format!("{RUNTIME}.jsonValue"),
    })
}

fn scalar_reader(scalar: &Scalar) -> &'static str {
    match scalar {
        // Every one of these travels as the text the API answered: Zig carries no type for an
        // identifier, a moment or a day, and text is what has to go back out unchanged.
        Scalar::String | Scalar::Url | Scalar::Uuid | Scalar::DateTime | Scalar::Date => "text",
        Scalar::Integer32 => "integer32",
        Scalar::Integer64 => "integer64",
        Scalar::Number => "number",
        Scalar::Boolean => "boolean",
    }
}

/// What the package imports to reach everything the generator writes.
fn index() -> String {
    format!(
        "pub const api = @import(\"{API_FILE}.zig\");\n\
         pub const errors = @import(\"{ERRORS_FILE}.zig\");\n\
         pub const models = @import(\"{MODELS_FILE}.zig\");\n"
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
/// Zig reads no interpolation inside a quoted string, so only the quote, the escape itself and what
/// is not text have to be spelled out. A control character is written as the hexadecimal escape the
/// language reads for one byte, applied to the bytes the character is encoded as, since `\x` names
/// a byte rather than a code point.
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
                let mut encoded = [0u8; 4];
                for byte in control.encode_utf8(&mut encoded).as_bytes() {
                    rendered.push_str(&format!("\\x{byte:02x}"));
                }
            }
            plain => rendered.push(plain),
        }
    }

    rendered.push('"');
    rendered
}
