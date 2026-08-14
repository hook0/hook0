//! Emits the generated half of the C# SDK.
//!
//! The package is published to NuGet with no copy of the OpenAPI snapshot beside it, so the types,
//! the problems and the request layer travel as committed source rather than as a build artefact.
//! Everything the API declares — one record per named schema, one closed list of constants per
//! closed list of strings, one exception per problem the error contract can report, one method per
//! operation — is written here; everything the API does not declare — how a request reaches the
//! network, how a send is retried, how a webhook signature is verified — is hand-written beside
//! this directory and never regenerated.
//!
//! The two halves meet at two seams and nowhere else. The generated code reads its decoding
//! helpers from the hand-written runtime beside it, and it calls whatever it is handed as a
//! transport: nothing here knows what a socket is, and nothing beside it knows what the API
//! declares.
//!
//! Both idioms are written from one description of an operation. C# callers use a blocking surface
//! and a `Task`-returning one, and a client carrying only one of them is a client half its callers
//! cannot use; writing them from one [`Flavour`] rather than from two emitters is what keeps a fix
//! to one of them from being a fix to only one of them.
//!
//! What is written is already formatted. An emitted method holds no local of its own — the path,
//! the query and the answer are all expressions — so an operation whose parameter is spelled like
//! one of the emitter's own names cannot quietly be assigned over, and C#'s rule that a local may
//! not share a name with a parameter in scope never applies.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, two arguments of one method spelled alike, a
//! scalar no annotation covers — stops the emission rather than yielding a smaller SDK.

use std::collections::{BTreeMap, BTreeSet};

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, spell};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Scalar, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "csharp";

/// Where the generated half of the package lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written.
const ROOT: &str = "clients/csharp/src/Hook0/Generated";

/// The segment of [`ROOT`] the namespace starts after.
///
/// C# is the one language here that writes its namespaces into the source rather than reading them
/// off the tree, so the two would otherwise be spelled in two places and could disagree. They are
/// spelled once: the namespace is what [`ROOT`] says below this segment, which is also why no name
/// of any particular API appears in this file.
const NAMESPACE_ROOT: &str = "src";

/// The files this target writes, each one holding one layer of the surface.
const MODELS_FILE: &str = "Models";
const ERRORS_FILE: &str = "Errors";
const SYNC_FILE: &str = "Api";
const ASYNC_FILE: &str = "AsyncApi";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix the `Task`-returning flavour of an operation group carries.
const ASYNC_GROUP_SUFFIX: &str = "AsyncApi";

/// Suffix a `Task`-returning method carries, which is what every C# caller expects to read.
const ASYNC_METHOD_SUFFIX: &str = "Async";

/// Suffix an exception carries, so a problem and a type spelling the same word stay apart.
const EXCEPTION_SUFFIX: &str = "Exception";

/// What the hand-written half declares, which the generated half reaches through the namespace it
/// sits under. Sorted, as they are claimed in order.
const RUNTIME_TYPE: &str = "Runtime";
const SYNC_TRANSPORT_TYPE: &str = "ITransport";
const ASYNC_TRANSPORT_TYPE: &str = "IAsyncTransport";
const ANSWER_TYPE: &str = "TransportAnswer";

/// What the generated half declares beside the types the document names.
const PROBLEMS_TYPE: &str = "Problems";

/// What an emitted group calls to raise what the API reported and hand back nothing.
const CHECK_HELPER: &str = "CheckAnswer";

/// What an emitted group calls to raise what the API reported and read what it answered.
const READ_HELPER: &str = "ReadAnswer";

/// The members a closed list of strings declares beside its values.
const DECLARED_MEMBER: &str = "Declared";
const VALUES_MEMBER: &str = "Values";
const CONTAINS_MEMBER: &str = "Contains";

/// What the field carrying the transport of an operation group is called.
///
/// It opens with the one character no rendered identifier can, so a parameter of an operation
/// spelled `transport` shadows the constructor's argument and never the field every method reads.
const TRANSPORT_FIELD: &str = "_transport";

/// What the argument carrying the body an operation reads is called.
const BODY_ARGUMENT: &str = "body";

/// What the argument abandoning a `Task`-returning request is called, and what it carries.
const CANCELLATION_ARGUMENT: &str = "cancellationToken";
const CANCELLATION_TYPE: &str = "CancellationToken";

/// Longest fragment of a snapshot description a documentation comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Longest line the emitted source carries.
///
/// An argument list that would not fit is written one argument per line. Which of the two a call is
/// written as depends only on how long it is, so the same model always yields the same bytes.
const MAX_LINE_CHARS: usize = 120;

/// How far one level of a block is indented, which is what the ecosystem's formatter prints.
const INDENT: &str = "    ";

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// The names the scaffolding around the emitted code answers to, which no type of the document may
/// answer to as well.
///
/// They are claimed alongside everything the model names, so a schema called `Runtime` is reported
/// as the collision it is rather than emitted as a type that shadows the hand-written one every
/// decoder reaches through. Sorted, as they are claimed in order.
const SCAFFOLDING: [&str; 5] = [
    ANSWER_TYPE,
    ASYNC_TRANSPORT_TYPE,
    PROBLEMS_TYPE,
    RUNTIME_TYPE,
    SYNC_TRANSPORT_TYPE,
];

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan the compiler still reads.
        ownership: Ownership::Directory,
        language: super::csharp(),
        emit,
    }
}

/// Everything the generated half of the package is made of.
fn emit(language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;
    let banner = banner(language.comment, &update_command(NAME), &limits)?;
    let namespace = namespace()?;

    let enums = model.enumerations(&limits)?;
    let types = Types::read(model, &enums, &namespace, language, &limits)?;

    let files = vec![
        file(
            MODELS_FILE,
            &banner,
            &namespace,
            &models(model, &enums, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            ERRORS_FILE,
            &banner,
            &namespace,
            &errors(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            SYNC_FILE,
            &banner,
            &namespace,
            &requests(model, &types, language, &limits, Flavour::Blocking)?,
            language,
            &limits,
        )?,
        file(
            ASYNC_FILE,
            &banner,
            &namespace,
            &requests(model, &types, language, &limits, Flavour::Awaiting)?,
            language,
            &limits,
        )?,
    ];

    FileTree::build(files, &limits)
}

/// The namespace the emitted types are declared in, read off [`ROOT`].
fn namespace() -> Result<String, Error> {
    let mut segments = ROOT
        .split('/')
        .skip_while(|segment| *segment != NAMESPACE_ROOT);
    if segments.next().is_none() {
        return Err(Error::UnsafePath {
            path: preview(ROOT),
            reason: format!(
                "it carries no `{NAMESPACE_ROOT}` segment for a namespace to start after"
            ),
        });
    }

    let namespace: Vec<&str> = segments.collect();
    if namespace.is_empty() {
        return Err(Error::UnsafePath {
            path: preview(ROOT),
            reason: format!(
                "nothing sits below its `{NAMESPACE_ROOT}` segment to name a namespace"
            ),
        });
    }

    Ok(namespace.join("."))
}

/// One file: the banner, the namespace everything in it is declared under, and the body.
fn file(
    stem: &str,
    banner: &str,
    namespace: &str,
    body: &Written,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<EmittedFile, Error> {
    let mut source = format!("{banner}\n");
    for used in &body.usings {
        source.push_str(&format!("using {used};\n"));
    }
    if !body.usings.is_empty() {
        source.push('\n');
    }
    source.push_str(&format!("namespace {namespace};\n{}", body.source));

    Ok(EmittedFile {
        path: RelativePath::build(&format!("{stem}.{}", language.extension), limits)?,
        contents: source,
    })
}

/// One emitted file's body, and the namespaces it turned out to need.
///
/// What a file imports is gathered while it is written rather than guessed beforehand: a `using`
/// nothing under it names is a line the ecosystem's own analysers report, and one that is missing
/// does not compile.
#[derive(Debug, Default)]
struct Written {
    usings: BTreeSet<&'static str>,
    source: String,
}

impl Written {
    fn namespace(&mut self, name: &'static str) {
        self.usings.insert(name);
    }
}

/// Whether a request layer waits for its transport or awaits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flavour {
    Blocking,
    Awaiting,
}

impl Flavour {
    /// What the group of that flavour is suffixed with.
    fn group_suffix(self) -> &'static str {
        match self {
            Self::Blocking => GROUP_SUFFIX,
            Self::Awaiting => ASYNC_GROUP_SUFFIX,
        }
    }

    /// What a method of that flavour is suffixed with.
    fn method_suffix(self) -> &'static str {
        match self {
            Self::Blocking => "",
            Self::Awaiting => ASYNC_METHOD_SUFFIX,
        }
    }

    /// What the group of that flavour issues its requests through.
    fn transport(self) -> &'static str {
        match self {
            Self::Blocking => SYNC_TRANSPORT_TYPE,
            Self::Awaiting => ASYNC_TRANSPORT_TYPE,
        }
    }

    /// What a method of that flavour is declared as.
    fn keyword(self) -> &'static str {
        match self {
            Self::Blocking => "public",
            Self::Awaiting => "public async",
        }
    }

    /// What a method of that flavour hands back, around the value the operation answers.
    fn returns(self, value: Option<&str>) -> String {
        match (self, value) {
            (Self::Blocking, None) => "void".to_owned(),
            (Self::Blocking, Some(value)) => value.to_owned(),
            (Self::Awaiting, None) => "Task".to_owned(),
            (Self::Awaiting, Some(value)) => format!("Task<{value}>"),
        }
    }

    /// How the call reaching the transport opens.
    fn opens(self) -> String {
        match self {
            Self::Blocking => format!("{TRANSPORT_FIELD}.Request("),
            Self::Awaiting => format!("await {TRANSPORT_FIELD}.Request{ASYNC_METHOD_SUFFIX}("),
        }
    }

    /// How the call reaching the transport closes.
    ///
    /// A library never resumes on whichever context its caller happened to await from: a caller
    /// blocking on one of these from a context that runs continuations on a single thread would
    /// otherwise wait for a continuation that cannot start.
    fn closes(self) -> &'static str {
        match self {
            Self::Blocking => ")",
            Self::Awaiting => ").ConfigureAwait(false)",
        }
    }

    /// What the namespaces a group of that flavour names are.
    fn needs(self, written: &mut Written) {
        if self == Self::Awaiting {
            written.namespace("System.Threading");
            written.namespace("System.Threading.Tasks");
        }
    }
}

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a type that does not compile once the file is read.
struct Types {
    /// Record each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Type each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Exception each problem is raised as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Exception every problem is a kind of.
    problem_base: String,
    /// Type the discriminant of the error contract is read through.
    problem_enum: String,
    /// Operation groups, by entity name, for each flavour.
    groups: BTreeMap<String, (String, String)>,
    /// What the language calls each scalar the model carries.
    scalars: ScalarNames,
}

impl Types {
    fn read(
        model: &ApiModel,
        enums: &BTreeMap<String, Vec<String>>,
        namespace: &str,
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

        // The namespace is claimed a segment at a time: a type answering to one of them would be
        // read as the namespace in half the places it is named and as the type in the other half.
        for segment in namespace.split('.') {
            claim(segment.to_owned(), "the namespace this target declares")?;
        }
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
            let blocking = claim(
                format!("{stem}{}", Flavour::Blocking.group_suffix()),
                &entity.name,
            )?;
            let awaiting = claim(
                format!("{stem}{}", Flavour::Awaiting.group_suffix()),
                &entity.name,
            )?;
            groups.insert(entity.name.clone(), (blocking, awaiting));
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

    fn group(&self, entity: &str, flavour: Flavour) -> Result<&str, Error> {
        self.groups
            .get(entity)
            .map(|(blocking, awaiting)| match flavour {
                Flavour::Blocking => blocking.as_str(),
                Flavour::Awaiting => awaiting.as_str(),
            })
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(entity),
            })
    }
}

/// The closed list the discriminant of the error contract is read through.
///
/// The catalogue is the values of one closed list of strings the error schema declares, so the type
/// already exists among the types: it is found rather than declared twice.
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
) -> Result<Written, Error> {
    let mut written = Written::default();

    for (name, values) in enums {
        let declared = types.enumeration(name)?.to_owned();
        let source = enumeration(&declared, values, &mut written, language, limits)?;
        written.source.push_str(&source);
    }
    for (name, object) in &model.schemas {
        let declared = types.schema(name)?.to_owned();
        let source = structure(&declared, object, types, &mut written, language, limits)?;
        written.source.push_str(&source);
    }

    Ok(written)
}

/// One closed list of strings, as the constants naming the values it admits.
///
/// The values travel as the strings the API answers, and each constant *is* that string, so nothing
/// is mapped on the way in or on the way out. An enumeration would put a mapping there: `in_progress`
/// is a value the API answers and `InProgress` is the only thing a C# member may be called, and the
/// day those two drift apart is the day the client writes a value the API never declared.
fn enumeration(
    declared: &str,
    values: &[String],
    written: &mut Written,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    written.namespace("System");
    written.namespace("System.Collections.Generic");

    // What the type declares beside the values is claimed first, so a value spelling one of them is
    // reported rather than declared twice.
    let mut members: BTreeMap<String, &str> = BTreeMap::new();
    for reserved in [DECLARED_MEMBER, VALUES_MEMBER, CONTAINS_MEMBER] {
        members.insert(
            reserved.to_owned(),
            "what this type declares beside its values",
        );
    }

    let mut constants = String::new();
    let mut listed = Vec::with_capacity(values.len());

    for value in values {
        let member = member_name(value, language.casing.constant, declared, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        constants.push_str(&documented(
            INDENT,
            &format!("The API answers <c>{}</c>.", comment(value)),
        ));
        constants.push_str(&format!(
            "{INDENT}public const string {member} = {};\n\n",
            literal(value)
        ));
        listed.push(literal(value));
    }

    Ok(format!(
        "\n{}public static class {declared}\n{{\n{constants}{}{}{}}}\n",
        documented("", "One of the values the API answers with."),
        format_args!(
            "{}\n\n",
            call(
                INDENT,
                &format!("private static readonly string[] {DECLARED_MEMBER} = "),
                "[",
                &listed,
                "]",
                ";",
            )
        ),
        format_args!(
            "{}{INDENT}public static IReadOnlyList<string> {VALUES_MEMBER} => {DECLARED_MEMBER};\n\n",
            documented(INDENT, "Every value the API declares for this list.")
        ),
        format_args!(
            "{}{INDENT}public static bool {CONTAINS_MEMBER}(string value) => \
             Array.IndexOf({DECLARED_MEMBER}, value) >= 0;\n",
            documented_call(
                INDENT,
                "Whether the API declares that value.",
                &[("value", "The text to look for.")],
                Some("Whether this list carries it."),
            )
        ),
    ))
}

/// One named schema, as the value a caller reads and writes.
///
/// The wire name of every member travels in the attribute beside it rather than in how the member
/// happens to be spelled, so a name moved out of the way of the language — or out of the way of the
/// type it sits in — never reaches the wire.
fn structure(
    declared: &str,
    object: &ObjectShape,
    types: &Types,
    written: &mut Written,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    written.namespace("System.Text.Json.Serialization");

    let ordered = ordered_fields(object);
    let mut source = format!(
        "\n{}",
        documented(
            "",
            &format!("The <c>{}</c> the API declares.", comment(&object.name))
        )
    );

    if ordered.is_empty() {
        source.push_str(&format!("public sealed record {declared};\n"));
        return Ok(source);
    }

    source.push_str(&format!("public sealed record {declared}\n{{\n"));

    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    for (index, field) in ordered.iter().enumerate() {
        let member = member_name(&field.name, language.casing.field, declared, limits)?;
        if let Some(first) = claimed.insert(member.clone(), &field.name) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(&field.name),
            });
        }

        if index > 0 {
            source.push('\n');
        }
        source.push_str(&documented(
            INDENT,
            &format!(
                "Carries <c>{}</c>{}",
                comment(&field.name),
                described(field.description.as_deref())
            ),
        ));
        source.push_str(&format!(
            "{INDENT}[JsonPropertyName({})]\n",
            literal(&field.name)
        ));
        if !field.required {
            // Said one member at a time rather than once for the whole document: a member the
            // document does require and a caller left unset still travels, so what was read comes
            // back out as what was read.
            source.push_str(&format!(
                "{INDENT}[JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]\n"
            ));
        }
        source.push_str(&format!(
            "{INDENT}public {}{} {member} {{ get; init; }}\n",
            if field.required { "required " } else { "" },
            annotation(&field.shape, field.required, types, written, 0)?
        ));
    }

    source.push_str("}\n");
    Ok(source)
}

/// Fields in the one order a record declares them: what the document requires, then what it does
/// not, so that reading a declaration says what has to be passed before what may be left out.
fn ordered_fields(object: &ObjectShape) -> Vec<&Field> {
    let mut ordered: Vec<&Field> = object
        .fields
        .iter()
        .filter(|field| field.required)
        .collect();
    ordered.extend(object.fields.iter().filter(|field| !field.required));
    ordered
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
) -> Result<Written, Error> {
    let mut written = Written::default();
    written.namespace("System");

    let schema = types.schema(&model.errors.schema)?;
    let discriminant = member_name(
        &model.errors.discriminant,
        language.casing.field,
        schema,
        limits,
    )?;
    let base = &types.problem_base;
    let catalogue = &types.problem_enum;

    let arguments = [
        ("status", "What the API answered under."),
        (
            "problem",
            "The document it answered, when this client could read one.",
        ),
        ("detail", "What to say about the failure."),
    ];
    let signature = format!("int status, {schema}? problem, string detail");

    written.source.push_str(&format!(
        "\n{}public class {base}({signature}) : Exception(detail)\n{{\n{}{INDENT}public int Status \
         {{ get; }} = status;\n\n{}{INDENT}public {schema}? {schema} {{ get; }} = problem;\n}}\n",
        documented_call(
            "",
            "A failure the API answered with, whether or not it could be read as a problem.",
            &arguments,
            None,
        ),
        documented(INDENT, "What the API answered under."),
        documented(
            INDENT,
            "The document the API answered, when this client could read one.",
        ),
    ));

    for (value, declared) in &types.problems {
        written.source.push_str(&format!(
            "\n{}public sealed class {declared}({signature})\n{INDENT}: {base}(status, problem, \
             detail);\n",
            documented_call(
                "",
                &format!("The API reported <c>{}</c>.", comment(value)),
                &arguments,
                None,
            ),
        ));
    }

    // The discriminant is what says whether a body named a problem at all, and a body naming one
    // this client has never heard of falls back to the base failure rather than to nothing.
    let mut arms = String::new();
    for (value, declared) in &types.problems {
        let member = member_name(value, language.casing.constant, catalogue, limits)?;
        arms.push_str(&format!(
            "{INDENT}{INDENT}{INDENT}{catalogue}.{member} =>\n{INDENT}{INDENT}{INDENT}{INDENT}new \
             {declared}(status, problem, {RUNTIME_TYPE}.Reported(status, problem)),\n"
        ));
    }

    written.source.push_str(&format!(
        "\n{}public static class {PROBLEMS_TYPE}\n{{\n\
         {}{INDENT}public static void {CHECK_HELPER}({ANSWER_TYPE} answered)\n\
         {INDENT}{{\n\
         {INDENT}{INDENT}RaiseForStatus(answered.Status, answered.Payload);\n\
         {INDENT}}}\n\n\
         {}{INDENT}public static TValue {READ_HELPER}<TValue>({ANSWER_TYPE} answered)\n\
         {INDENT}{{\n\
         {INDENT}{INDENT}RaiseForStatus(answered.Status, answered.Payload);\n\
         {INDENT}{INDENT}return {RUNTIME_TYPE}.Read<TValue>(answered.Payload);\n\
         {INDENT}}}\n\n\
         {}{INDENT}public static void RaiseForStatus(int status, byte[] payload)\n\
         {INDENT}{{\n\
         {INDENT}{INDENT}if (status is >= {LOWEST_SUCCESS} and < {LOWEST_REDIRECTION})\n\
         {INDENT}{INDENT}{{\n\
         {INDENT}{INDENT}{INDENT}return;\n\
         {INDENT}{INDENT}}}\n\n\
         {INDENT}{INDENT}{schema}? problem = {RUNTIME_TYPE}.ReadOrNothing<{schema}>(payload);\n\
         {INDENT}{INDENT}if (problem is null)\n\
         {INDENT}{INDENT}{{\n\
         {INDENT}{INDENT}{INDENT}throw new {base}(status, null, {RUNTIME_TYPE}.Unreadable(status, \
         payload));\n\
         {INDENT}{INDENT}}}\n\n\
         {INDENT}{INDENT}throw problem.{discriminant} switch\n\
         {INDENT}{INDENT}{{\n\
         {arms}\
         {INDENT}{INDENT}{INDENT}_ =>\n\
         {INDENT}{INDENT}{INDENT}{INDENT}new {base}(status, problem, {RUNTIME_TYPE}.Reported(status, \
         problem)),\n\
         {INDENT}{INDENT}}};\n\
         {INDENT}}}\n}}\n",
        documented("", "The failures the API reports, read out of what it answered."),
        documented_call(
            INDENT,
            "Raise what the API reported, and answer nothing when it reported nothing.",
            &[("answered", "The status and the body the transport answered.")],
            None,
        ),
        documented_call(
            INDENT,
            "Raise what the API reported, or read back the value it answered.",
            &[("answered", "The status and the body the transport answered.")],
            Some("The value the API answered."),
        ),
        documented_call(
            INDENT,
            "Raise what the API reported, when what it answered was not a success.",
            &[
                ("status", "What the API answered under."),
                ("payload", "The body it answered."),
            ],
            None,
        ),
    ));

    Ok(written)
}

/// One method per operation, grouped by the entity its operation id names.
fn requests(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<Written, Error> {
    let mut written = Written::default();
    flavour.needs(&mut written);

    for entity in model.entities.entities() {
        let source = group(entity, types, &mut written, language, limits, flavour)?;
        written.source.push_str(&source);
    }

    Ok(written)
}

fn group(
    entity: &Entity,
    types: &Types,
    written: &mut Written,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
    let declared = types.group(&entity.name, flavour)?.to_owned();
    let transport = flavour.transport();

    let mut source = format!(
        "\n{}public sealed class {declared}({transport} transport)\n{{\n\
         {INDENT}private readonly {transport} {TRANSPORT_FIELD} = transport;\n",
        documented_call(
            "",
            &format!(
                "What the API declares under <c>{}</c>, issued through the transport it is handed.",
                comment(&entity.name)
            ),
            &[("transport", "What one request is issued through.")],
            None,
        )
    );

    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    for method in &entity.methods {
        let name = method_name(&method.verb_text, &declared, flavour, limits)?;
        if let Some(first) = claimed.insert(name.clone(), &method.operation_id) {
            return Err(Error::SchemaNameCollision {
                name: preview(&name),
                first: preview(first),
                second: preview(&method.operation_id),
            });
        }

        source.push('\n');
        source.push_str(&operation(
            &name, method, types, written, language, limits, flavour,
        )?);
    }

    source.push_str("}\n");
    Ok(source)
}

/// One argument of an emitted method: what it is called, what it carries, and where it travels.
struct Argument<'a> {
    name: String,
    annotated: String,
    parameter: &'a Parameter,
}

fn operation(
    name: &str,
    method: &Method,
    types: &Types,
    written: &mut Written,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
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
        Some(_) => Some(ident(BODY_ARGUMENT, Case::LowerCamel, language, limits)?),
        None => None,
    };
    refuse_arguments_spelled_alike(
        method,
        &required,
        &optional,
        body_argument.as_deref(),
        flavour,
    )?;

    let returned = match method.success.as_ref() {
        Some((_, Some(shape))) => Some(annotation(shape, true, types, written, 0)?),
        _ => None,
    };

    let mut documented_arguments: Vec<(&str, String)> = Vec::new();
    for argument in &required {
        documented_arguments.push((
            argument.name.as_str(),
            format!(
                "Carries <c>{}</c>{}",
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    if let Some(body) = body_argument.as_deref() {
        documented_arguments.push((body, "What the operation reads.".to_owned()));
    }
    for argument in &optional {
        documented_arguments.push((
            argument.name.as_str(),
            format!(
                "Carries <c>{}</c>, when the caller passes one{}",
                comment(&argument.parameter.name),
                described(argument.parameter.description.as_deref())
            ),
        ));
    }
    if flavour == Flavour::Awaiting {
        documented_arguments.push((
            CANCELLATION_ARGUMENT,
            "What abandons the request before it is answered.".to_owned(),
        ));
    }

    let described_arguments: Vec<(&str, &str)> = documented_arguments
        .iter()
        .map(|(name, text)| (*name, text.as_str()))
        .collect();

    let mut source = documented_call(
        INDENT,
        &summary(method),
        &described_arguments,
        answers(flavour, returned.is_some()),
    );

    let mut declared: Vec<String> = Vec::new();
    for argument in &required {
        declared.push(format!("{} {}", argument.annotated, argument.name));
    }
    if let (Some(body), Some(shape)) = (body_argument.as_deref(), method.request.as_ref()) {
        declared.push(format!(
            "{} {body}",
            annotation(shape, true, types, written, 0)?
        ));
    }
    for argument in &optional {
        declared.push(format!("{}? {} = null", argument.annotated, argument.name));
    }
    if flavour == Flavour::Awaiting {
        declared.push(format!(
            "{CANCELLATION_TYPE} {CANCELLATION_ARGUMENT} = default"
        ));
    }

    source.push_str(&signature(
        INDENT,
        &format!(
            "{} {} {name}",
            flavour.keyword(),
            flavour.returns(returned.as_deref())
        ),
        &declared,
    ));
    source.push_str(&format!("{INDENT}{{\n"));

    // Where an argument of the transport call sits once that call is written open, which is what
    // the expressions handed to it are folded against.
    let handed = format!("{INDENT}{INDENT}{INDENT}");
    let mut issued = vec![
        literal(operation.method.as_str()),
        path(operation.path.as_str(), &required, written, &handed),
        query(&required, &optional, &handed),
    ];
    issued.push(match method.request.as_ref() {
        None => "null".to_owned(),
        Some(_) => body_argument
            .clone()
            .unwrap_or_else(|| BODY_ARGUMENT.to_owned()),
    });
    if flavour == Flavour::Awaiting {
        issued.push(CANCELLATION_ARGUMENT.to_owned());
    }

    let head = match returned.as_deref() {
        Some(value) => format!("return {PROBLEMS_TYPE}.{READ_HELPER}<{value}>("),
        None => format!("{PROBLEMS_TYPE}.{CHECK_HELPER}("),
    };
    source.push_str(&call(
        &format!("{INDENT}{INDENT}"),
        &head,
        &flavour.opens(),
        &issued,
        flavour.closes(),
        ");",
    ));
    source.push('\n');
    source.push_str(&format!("{INDENT}}}\n"));

    Ok(source)
}

/// What the `<returns>` of a method says, when a method of that flavour has one to say anything
/// about. A `void` method has nothing to return and a documented return on one is a warning.
fn answers(flavour: Flavour, carries: bool) -> Option<&'static str> {
    match (flavour, carries) {
        (Flavour::Blocking, false) => None,
        (Flavour::Blocking, true) => Some("What the API answered."),
        (Flavour::Awaiting, false) => Some("The request, once the API has answered it."),
        (Flavour::Awaiting, true) => Some("What the API answered, once it has."),
    }
}

/// Refuses two arguments of one method that would be spelled the same way.
///
/// C# has one namespace for the arguments of a method, so a path parameter and a query parameter
/// the document spells `event-id` and `event_id` would be one argument, and whichever one lost
/// would travel carrying the other one's value.
fn refuse_arguments_spelled_alike(
    method: &Method,
    required: &[Argument<'_>],
    optional: &[Argument<'_>],
    body: Option<&str>,
    flavour: Flavour,
) -> Result<(), Error> {
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();
    if let Some(body) = body {
        claimed.insert(body, "the body the operation reads");
    }
    if flavour == Flavour::Awaiting {
        claimed.insert(
            CANCELLATION_ARGUMENT,
            "what abandons a request this flavour is given",
        );
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
fn path(template: &str, required: &[Argument<'_>], written: &mut Written, indent: &str) -> String {
    let filled: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Path)
        .collect();

    if filled.is_empty() {
        return literal(template);
    }

    written.namespace("System.Collections.Generic");

    // Written from the indent it will sit at, so what decides whether it is one line is the line it
    // will actually occupy rather than the expression on its own; the indent is then trimmed back
    // off, since whatever writes it in puts the first line where it goes.
    let entries = format!("{indent}{INDENT}");
    call(
        indent,
        "",
        &format!("{RUNTIME_TYPE}.Path("),
        &[literal(template), list(&entries, &pairs(&filled))],
        ")",
        "",
    )
    .trim_start()
    .to_owned()
}

/// What travels in the query string, as one expression.
///
/// What the document requires is always sent; what it does not is sent only when the caller passed
/// it, which the runtime decides rather than the emitted method.
fn query(required: &[Argument<'_>], optional: &[Argument<'_>], indent: &str) -> String {
    let asked: Vec<&Argument<'_>> = required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
        .collect();

    let entries = format!("{indent}{INDENT}");
    call(
        indent,
        "",
        &format!("{RUNTIME_TYPE}.Query("),
        &[
            list(&entries, &pairs(&asked)),
            list(
                &entries,
                &pairs(&optional.iter().collect::<Vec<&Argument<'_>>>()),
            ),
        ],
        ")",
        "",
    )
    .trim_start()
    .to_owned()
}

/// The name a parameter travels under and the argument carrying its value, as a list of pairs.
fn pairs(arguments: &[&Argument<'_>]) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| format!("({}, {})", literal(&argument.parameter.name), argument.name))
        .collect()
}

/// A collection, on the line it sits on when it fits there and one entry per line when it does not.
///
/// Folded against the indent it will be written at rather than against nothing, for the same reason
/// a call is: what has to fit is the line, not the expression.
fn list(indent: &str, entries: &[String]) -> String {
    call(indent, "", "[", entries, "]", "")
        .trim_start()
        .to_owned()
}

/// One call, on the line it sits on when the whole line fits and one argument per line when it does
/// not.
///
/// What is measured is the line as it will actually be written — everything before the call and
/// everything after it included — rather than the call alone, since a margin the surrounding text is
/// not counted against is not a margin. An argument that is itself written across lines forces the
/// call open too: there is no arrangement in which one of them sits inside a single line.
///
/// Which of the two a call is written as depends only on how long it is, so the same model always
/// yields the same bytes; and a call that would have run past the margin is broken here rather than
/// left for something downstream to reformat.
fn call(
    indent: &str,
    head: &str,
    opens: &str,
    arguments: &[String],
    closes: &str,
    tail: &str,
) -> String {
    let single = format!(
        "{indent}{head}{opens}{}{closes}{tail}",
        arguments.join(", ")
    );
    let folded = arguments.iter().any(|argument| argument.contains('\n'));
    if !folded && single.chars().count() <= MAX_LINE_CHARS {
        return single;
    }

    let mut source = format!("{indent}{head}{opens}\n");
    for (index, argument) in arguments.iter().enumerate() {
        source.push_str(&format!(
            "{indent}{INDENT}{argument}{}\n",
            if index + 1 == arguments.len() {
                ""
            } else {
                ","
            }
        ));
    }
    source.push_str(&format!("{indent}{closes}{tail}"));
    source
}

/// One signature, on one line when it fits and one argument per line when it does not.
fn signature(indent: &str, declaration: &str, declared: &[String]) -> String {
    let single = format!("{indent}{declaration}({})\n", declared.join(", "));
    if single.chars().count() - 1 <= MAX_LINE_CHARS {
        return single;
    }

    let mut source = format!("{indent}{declaration}(\n");
    for (index, argument) in declared.iter().enumerate() {
        source.push_str(&format!(
            "{indent}{INDENT}{argument}{}\n",
            if index + 1 == declared.len() {
                ")"
            } else {
                ","
            }
        ));
    }
    source
}

/// The type a value of that shape carries.
fn annotation(
    shape: &Shape,
    required: bool,
    types: &Types,
    written: &mut Written,
    depth: usize,
) -> Result<String, Error> {
    let declared = annotated(shape, types, written, depth)?;
    if required {
        return Ok(declared);
    }
    Ok(format!("{declared}?"))
}

fn annotated(
    shape: &Shape,
    types: &Types,
    written: &mut Written,
    depth: usize,
) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => {
            if let Some(namespace) = scalar_namespace(scalar) {
                written.namespace(namespace);
            }
            types.scalars.of(scalar).to_owned()
        }
        Shape::Array(inner) => {
            written.namespace("System.Collections.Generic");
            format!(
                "IReadOnlyList<{}>",
                annotated(inner, types, written, depth + 1)?
            )
        }
        Shape::Map(inner) => {
            written.namespace("System.Collections.Generic");
            format!(
                "IReadOnlyDictionary<string, {}>",
                annotated(inner, types, written, depth + 1)?
            )
        }
        // A value of a closed list travels as the string the API answers, and that is what it is;
        // which strings it may be is what the type declaring them says.
        Shape::Enum { .. } => "string".to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => {
            written.namespace("System.Text.Json.Nodes");
            "JsonNode".to_owned()
        }
    })
}

/// The namespace an annotation of that scalar names, when it names one.
fn scalar_namespace(scalar: &Scalar) -> Option<&'static str> {
    match scalar {
        Scalar::Uuid | Scalar::DateTime | Scalar::Date => Some("System"),
        Scalar::String
        | Scalar::Url
        | Scalar::Integer32
        | Scalar::Integer64
        | Scalar::Number
        | Scalar::Boolean => None,
    }
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "string",
        "integer" => "long",
        "number" => "double",
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

fn deep_enough(depth: usize) -> Result<(), Error> {
    if depth > Limits::DEFAULT.max_shape_depth {
        return Err(Error::SchemaTooDeep {
            subject: preview("a value of the model"),
            limit: Limits::DEFAULT.max_shape_depth,
        });
    }
    Ok(())
}

/// The name a method of an operation group is spelled under.
///
/// A method stays clear of what every object already answers to, and of the name of the group it
/// sits in — which the compiler refuses outright and which no vocabulary can hold, since it is a
/// different word for every group.
fn method_name(
    text: &str,
    enclosing: &str,
    flavour: Flavour,
    limits: &Limits,
) -> Result<String, Error> {
    let spelled = spell(
        text,
        Case::UpperCamel,
        super::csharp_member_reserved(),
        limits,
    )?;
    Ok(apart_from(
        &format!("{spelled}{}", flavour.method_suffix()),
        enclosing,
    ))
}

/// The name a member of an emitted declaration is spelled under.
fn member_name(text: &str, case: Case, enclosing: &str, limits: &Limits) -> Result<String, Error> {
    let spelled = spell(text, case, super::csharp_member_reserved(), limits)?;
    Ok(apart_from(&spelled, enclosing))
}

/// A member spelled out of the way of the type it sits in.
///
/// C# refuses a member carrying the name of its own type, and that is not a vocabulary: it is one
/// word per declaration. The escape is the same one the shadowing vocabulary uses, so a member
/// moved out of the way reads the same whichever of the two moved it.
fn apart_from(member: &str, enclosing: &str) -> String {
    if member == enclosing {
        return format!("{member}_");
    }
    member.to_owned()
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
            "<c>{}</c> on <c>{}</c>.",
            comment(method.operation.method.as_str()),
            comment(&method.operation.path)
        ),
    }
}

/// A documentation comment carrying nothing but what the declaration is.
fn documented(indent: &str, text: &str) -> String {
    documented_call(indent, text, &[], None)
}

/// A documentation comment: what the declaration is, what each of its arguments carries, and what
/// it hands back.
///
/// Every public declaration carries one. A build that asks for the documentation file reports every
/// one that does not, and an argument left undescribed beside ones that are described is reported
/// too, so the comment is written whole here rather than being completed by hand afterwards.
fn documented_call(
    indent: &str,
    text: &str,
    arguments: &[(&str, &str)],
    returns: Option<&str>,
) -> String {
    let mut source = folded(indent, "summary", text);
    for (name, described) in arguments {
        source.push_str(&folded(
            indent,
            &format!("param name=\"{name}\""),
            described,
        ));
    }
    if let Some(returns) = returns {
        source.push_str(&folded(indent, "returns", returns));
    }
    source
}

/// One documentation element, on one line when it fits and folded between words when it does not.
///
/// A description the document writes as one paragraph is one line here until it no longer fits;
/// what is left runs on under the same indent. Folding happens between words, and a single word
/// longer than a line is written whole rather than cut in half — a name is worth more than a margin.
fn folded(indent: &str, element: &str, text: &str) -> String {
    let closing: String = element.split(' ').next().unwrap_or(element).to_owned();
    let opening = format!("{indent}/// <{element}>");
    let single = format!("{opening}{text}</{closing}>");
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }

    let continuation = format!("{indent}/// ");
    let mut lines = format!("{opening}\n");
    let mut line = continuation.clone();
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

    if !empty {
        lines.push_str(&line);
        lines.push('\n');
    }
    lines.push_str(&format!("{indent}/// </{closing}>\n"));
    lines
}

/// Snapshot text, as a documentation comment may carry it.
///
/// The snapshot is untrusted input travelling into source that a compiler reads as XML: a run of
/// whitespace becomes one space so nothing leaves the line the comment sits on, the three
/// characters that would open a tag or an entity are written as the entities that stand for them,
/// and what is left is cut at a fixed budget.
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

    // Escaped after the budget rather than before it, so an entity is never cut in half.
    rendered
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Snapshot text, as a string literal may carry it.
fn literal(text: &str) -> String {
    let mut rendered = String::from("\"");
    for character in text.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
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
