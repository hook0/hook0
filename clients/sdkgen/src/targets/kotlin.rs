//! Emits the generated half of the Kotlin SDK.
//!
//! The artefact is published to Maven Central with no copy of the OpenAPI snapshot beside it, so
//! the types, the problems and the request layer travel as committed source rather than as a build
//! artefact. Everything the API declares — one `data class` per named schema, one `enum class` per
//! closed list of strings, one exception per problem the error contract can report, one method per
//! operation — is written here; everything the API does not declare — how a request reaches the
//! network, how a send is retried, how a webhook signature is verified, how a JSON document is read
//! — is hand-written beside this directory and never regenerated.
//!
//! The two halves meet at one seam: the generated code reads its decoding helpers and its transport
//! contract from the package above it, and calls whatever object it is handed as a transport.
//! Nothing here knows what a socket is, and nothing beside it knows what the API declares.
//!
//! Every operation is written twice, once blocking and once suspending, and the two are rendered by
//! the same function under a different [`Flavour`]. The two bodies differ in exactly two tokens —
//! the `suspend` modifier and which method of the transport is called — because everything else,
//! including the decision to raise and the decision to read, is one generated helper both of them
//! hand their answer to. There is no second place for the two surfaces to disagree in.
//!
//! Kotlin does not insist that a public type sit in a file named after it, but this target writes
//! one file per type anyway: the whole directory is owned by the generator, so a type the document
//! stops declaring has to take a file with it rather than leave a stale declaration inside a file
//! that is still written.
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
use crate::targets::{Contract, Decoding, LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "kotlin";

/// Where the generated half of the artefact lands.
///
/// Everything under it belongs to the generator; everything beside it, up the package tree, is
/// hand-written. The package sits beside the Java client's rather than under it: two artefacts
/// declaring `com.hook0.client.Options` would be one name on any classpath holding both, and
/// whichever one lost would be the one nobody meant to use.
const ROOT: &str = "clients/kotlin/src/main/kotlin/com/hook0/kotlin/generated";

/// The package everything written here is declared in.
const PACKAGE: &str = "com.hook0.kotlin.generated";

/// The hand-written types the generated half reaches, each one as it is imported and as it is
/// spelled once imported. They are the whole seam: nothing else outside this package is named.
const WIRE_IMPORT: &str = "com.hook0.kotlin.Wire";
const WIRE: &str = "Wire";
const ANSWER_IMPORT: &str = "com.hook0.kotlin.Answer";
const ANSWER: &str = "Answer";
const TRANSPORT_IMPORT: &str = "com.hook0.kotlin.Transport";
const TRANSPORT: &str = "Transport";
const QUERY_PARAMETER_IMPORT: &str = "com.hook0.kotlin.QueryParameter";
const QUERY_PARAMETER: &str = "QueryParameter";
const BASE_EXCEPTION_IMPORT: &str = "com.hook0.kotlin.Hook0Exception";
const BASE_EXCEPTION: &str = "Hook0Exception";
const DECODE_EXCEPTION_IMPORT: &str = "com.hook0.kotlin.DecodeException";
const DECODE_EXCEPTION: &str = "DecodeException";

/// Object holding what an answer means, which is what keeps the two surfaces from each deciding for
/// themselves.
const PROBLEMS_OBJECT: &str = "Problems";

/// Suffix telling an operation group from a type of the same name: the document names an entity
/// and a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix the flavour of an operation group whose calls suspend carries.
const SUSPENDING_GROUP_SUFFIX: &str = "SuspendingApi";

/// Suffix an exception carries, so a problem and a type spelling the same word stay apart.
const EXCEPTION_SUFFIX: &str = "Exception";

/// Prefix the one exception that stands for a failure no problem document could be read out of.
///
/// A sealed class is abstract in Kotlin, so — unlike the Java target, which raises its own base —
/// the failure the API answered without a problem document needs a member of the hierarchy of its
/// own. It is part of the sealed set, so a caller matching on the set exhaustively still covers it.
const UNREADABLE_PREFIX: &str = "Unreadable";

/// What an emitted group hands an answer it has nothing to read out of.
const CHECK_HELPER: &str = "checkAnswer";

/// What an emitted group hands an answer it has a value to read out of.
const READ_HELPER: &str = "readAnswer";

/// Member every value declares to write itself back the way the API reads it.
const WRITE_MEMBER: &str = "toJson";

/// Member every value declares to be read out of what the API answered.
const READ_MEMBER: &str = "fromJson";

/// Member a closed list of strings declares to answer the text it travels as.
const WIRE_MEMBER: &str = "wireValue";

/// What the argument carrying the body an operation reads is called.
const BODY_ARGUMENT: &str = "body";

/// Longest fragment of a snapshot description a documentation comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Longest line the emitted source carries, which is what the committed style check enforces.
///
/// A description the document writes as one paragraph is folded across as many comment lines as it
/// takes, and a declaration or a call that would not fit is written one argument per line. Both are
/// what the style check asks for, and neither is something a pass over the emitted source should
/// have to work out afterwards.
const MAX_LINE_CHARS: usize = 120;

/// How far a wrapped argument sits in from the line it continues, which is the indentation the
/// committed style settings name.
const CONTINUATION: &str = "  ";

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan nothing compiles against.
        ownership: Ownership::Directory,
        contract: Contract::Whole,
        decoding: Decoding::Modelled,
        language: super::kotlin(),
        emit,
    }
}

/// Everything the generated half of the artefact is made of.
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
            &enumeration(declared, values, language, &limits)?,
            &limits,
        )?);
    }

    for (name, object) in &model.schemas {
        let declared = types.schema(name)?;
        files.push(file(
            declared,
            &banner,
            &structure(declared, object, &types, language, &limits)?,
            &limits,
        )?);
    }

    files.push(file(
        &types.problem_base,
        &banner,
        &problem_base(&types),
        &limits,
    )?);
    files.push(file(
        &types.problem_unreadable,
        &banner,
        &problem_unreadable(&types),
        &limits,
    )?);
    for (value, declared) in &types.problems {
        files.push(file(
            declared,
            &banner,
            &problem(value, declared, &types),
            &limits,
        )?);
    }
    files.push(file(
        PROBLEMS_OBJECT,
        &banner,
        &problems(&types, language, &limits)?,
        &limits,
    )?);

    for entity in model.entities.entities() {
        for flavour in [Flavour::Blocking, Flavour::Suspending] {
            let declared = types.group(&entity.name, flavour)?;
            files.push(file(
                declared,
                &banner,
                &group(entity, &types, language, &limits, flavour)?,
                &limits,
            )?);
        }
    }

    FileTree::build(files, &limits)
}

/// One file: the banner, the package it declares, and the body that names its own imports.
fn file(declared: &str, banner: &str, body: &str, limits: &Limits) -> Result<EmittedFile, Error> {
    Ok(EmittedFile {
        path: RelativePath::build(&format!("{declared}.kt"), limits)?,
        contents: format!("{banner}\npackage {PACKAGE}\n\n{body}"),
    })
}

/// Whether a request layer waits for its transport or suspends until it has answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flavour {
    Blocking,
    Suspending,
}

impl Flavour {
    /// What the class says about itself, beyond the entity it covers.
    fn described(self) -> &'static str {
        match self {
            Self::Blocking => "Every call blocks until the API has answered.",
            Self::Suspending => {
                "Every call suspends until the API has answered, and holds no thread while it waits."
            }
        }
    }

    /// What a method of this flavour is declared as.
    fn modifier(self) -> &'static str {
        match self {
            Self::Blocking => "",
            Self::Suspending => "suspend ",
        }
    }

    /// Which method of the transport a method of this flavour issues its request through.
    fn issues_through(self) -> &'static str {
        match self {
            Self::Blocking => "request",
            Self::Suspending => "requestSuspending",
        }
    }
}

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as two files the compiler then refuses to read together.
struct Types {
    /// Type each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Named schemas declaring no member at all, which Kotlin has no `data class` for.
    memberless: BTreeSet<String>,
    /// Type each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Exception each problem is raised as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Exception every problem is a kind of.
    problem_base: String,
    /// Exception a failure carrying no readable problem document is raised as.
    problem_unreadable: String,
    /// Type the problem document itself is read as.
    problem_document: String,
    /// Enumeration the discriminant of the error contract is read through.
    catalogue: String,
    /// Member of the error schema that tells one problem from another.
    problem_discriminant: String,
    /// Operation groups, by entity name, for each flavour.
    groups: BTreeMap<String, (String, String)>,
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

        claim(
            PROBLEMS_OBJECT.to_owned(),
            "the scaffolding this target writes",
        )?;

        let mut schemas = BTreeMap::new();
        let mut memberless = BTreeSet::new();
        for (name, object) in &model.schemas {
            let declared = claim(
                ident(name, language.casing.type_name, language, limits)?,
                name,
            )?;
            if object.fields.is_empty() {
                memberless.insert(declared.clone());
            }
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

        // The catalogue is the values of one closed list of strings the error schema declares, so
        // the enumeration already exists among the types: it is found rather than declared twice.
        let catalogue = catalogue_of(&model.errors, &declared_enums)?;

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
        let problem_unreadable = claim(
            format!("{UNREADABLE_PREFIX}{problem_base}"),
            "the scaffolding this target writes",
        )?;

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
            let blocking = claim(format!("{stem}{GROUP_SUFFIX}"), &entity.name)?;
            let suspending = claim(format!("{stem}{SUSPENDING_GROUP_SUFFIX}"), &entity.name)?;
            groups.insert(entity.name.clone(), (blocking, suspending));
        }

        let problem_document = schemas.get(&model.errors.schema).cloned().ok_or_else(|| {
            Error::UnresolvableReference {
                reference: preview(&model.errors.schema),
            }
        })?;
        let problem_discriminant = ident(
            &model.errors.discriminant,
            language.casing.field,
            language,
            limits,
        )?;

        Ok(Self {
            schemas,
            memberless,
            enums: declared_enums,
            problems,
            problem_base,
            problem_unreadable,
            problem_document,
            catalogue,
            problem_discriminant,
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
            .map(|(blocking, suspending)| match flavour {
                Flavour::Blocking => blocking.as_str(),
                Flavour::Suspending => suspending.as_str(),
            })
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(entity),
            })
    }
}

/// The enumeration the discriminant of the error contract is read through.
fn catalogue_of(errors: &ErrorModel, enums: &BTreeMap<String, String>) -> Result<String, Error> {
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

/// What a file imports, gathered while its body is written rather than guessed afterwards.
///
/// Only what sits outside the generated package and outside what Kotlin imports by default is ever
/// named: everything this target declares shares one package, and `Map`, `List`, `ArrayList` and
/// `LinkedHashMap` are all reached without a line of import that would have to be kept in step with
/// the model.
#[derive(Debug, Default)]
struct Needs(BTreeSet<&'static str>);

impl Needs {
    fn on(&mut self, qualified: &'static str) {
        self.0.insert(qualified);
    }

    /// The import block, sorted the way the committed style settings read it.
    fn block(&self) -> String {
        if self.0.is_empty() {
            return String::new();
        }

        let mut block = String::new();
        for qualified in &self.0 {
            block.push_str(&format!("import {qualified}\n"));
        }
        block.push('\n');
        block
    }
}

/// One closed list of strings, as the enumeration a caller reads and writes.
///
/// Each constant carries the text the API spells it with, so nothing is spelled twice: what goes
/// back on the wire is what came off it, and a value the API answers that the list does not declare
/// stops the read rather than travelling on as a string nobody checked.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    needs.on(WIRE_IMPORT);
    needs.on(DECODE_EXCEPTION_IMPORT);

    let mut members: BTreeMap<String, &str> = BTreeMap::new();
    let mut constants = String::new();

    for (index, value) in values.iter().enumerate() {
        let member = ident(value, language.casing.constant, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        if index > 0 {
            constants.push('\n');
        }
        constants.push_str(&one_line_doc(
            "  ",
            &format!("The API spells this one `{}`.", comment(value)),
        ));
        constants.push_str(&format!(
            "  {member}({}){}\n",
            literal(value),
            if index + 1 == values.len() { ";" } else { "," }
        ));
    }
    if values.is_empty() {
        // A list declaring no value still has to read as an enumeration, and the semicolon is what
        // separates the constants it does not have from the members it does.
        constants.push_str("  ;\n");
    }

    let mut body = documented("", &["One of the values the API answers with.".to_owned()]);
    body.push_str(&format!(
        "enum class {declared}(val {WIRE_MEMBER}: String) {{\n{constants}\n"
    ));
    body.push_str("  companion object {\n");
    body.push_str(&documented(
        "    ",
        &[
            "Reads one out of what the API answered.".to_owned(),
            String::new(),
            "@param value the JSON document the API answered".to_owned(),
            "@return the value it names".to_owned(),
        ],
    ));
    body.push_str(&format!(
        "    fun {READ_MEMBER}(value: Any?): {declared} {{\n      \
         val named = {WIRE}.asText(value)\n      \
         for (candidate in {declared}.entries) {{\n        \
         if (candidate.{WIRE_MEMBER} == named) {{\n          \
         return candidate\n        \
         }}\n      \
         }}\n"
    ));
    body.push_str(&call(
        "      ",
        &format!("throw {DECODE_EXCEPTION}"),
        &[format!(
            "\"`${{{WIRE}.preview(named)}}` is not one of the values {declared} declares\""
        )],
        "",
    ));
    body.push_str("    }\n  }\n}\n");

    Ok(format!("{}{body}", needs.block()))
}

/// One named schema, as the value a caller reads and writes.
///
/// A schema is a `data class`: what the API answers carries no identity beyond what it holds, and a
/// data class says exactly that — the members, the equality, the printing and the copying all follow
/// from the one declaration. A member the document does not require is the only one declared
/// nullable, and it carries a default of nothing so a caller building a body writes down what the
/// API requires and no more.
fn structure(
    declared: &str,
    object: &ObjectShape,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    let ordered = ordered_fields(object);

    let mut named: Vec<(String, &Field)> = Vec::with_capacity(ordered.len());
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

    let mut documentation = vec![format!("The `{}` the API declares.", comment(&object.name))];
    if !named.is_empty() {
        documentation.push(String::new());
    }
    let mut components = Vec::with_capacity(named.len());
    for (name, field) in &named {
        documentation.push(format!(
            "@property {name} carries `{}`{}{}",
            comment(&field.name),
            if field.required {
                ""
            } else {
                ", or nothing when the API answers none"
            },
            described(field.description.as_deref())
        ));
        components.push(format!(
            "val {name}: {}{}",
            annotation(&field.shape, types, &mut needs, 0)?,
            if field.required { "" } else { "? = null" }
        ));
    }

    let mut body = documented("", &documentation);
    if named.is_empty() {
        // Kotlin declares no `data class` carrying nothing, so a schema with no member is written
        // as the value it stands for: one that equals every other one of its type and prints as it.
        body.push_str(&format!("class {declared} {{\n"));
    } else {
        body.push_str(&call(
            "",
            &format!("data class {declared}"),
            &components,
            " {",
        ));
    }
    body.push('\n');

    body.push_str(&documented(
        "  ",
        &[
            "Writes one back the way the API reads it.".to_owned(),
            String::new(),
            "@return the document the API reads".to_owned(),
        ],
    ));
    body.push_str(&format!(
        "  fun {WRITE_MEMBER}(): Map<String, Any?> {{\n    \
         val out = LinkedHashMap<String, Any?>()\n"
    ));
    for (name, field) in &named {
        let written = writer(&field.shape, name, types, &mut needs, 0)?;
        if field.required {
            body.push_str(&assignment(
                "    ",
                &format!("out[{}]", literal(&field.name)),
                &written,
            ));
            continue;
        }
        body.push_str(&format!("    if ({name} != null) {{\n"));
        body.push_str(&assignment(
            "      ",
            &format!("out[{}]", literal(&field.name)),
            &written,
        ));
        body.push_str("    }\n");
    }
    body.push_str("    return out\n  }\n");

    if named.is_empty() {
        body.push_str(&format!(
            "\n  override fun equals(other: Any?): Boolean = other is {declared}\n\n  \
             override fun hashCode(): Int = {declared}::class.hashCode()\n\n  \
             override fun toString(): String = \"{declared}()\"\n"
        ));
    }

    body.push_str("\n  companion object {\n");
    body.push_str(&documented(
        "    ",
        &[
            "Reads one out of what the API answered.".to_owned(),
            String::new(),
            "@param value the JSON document the API answered".to_owned(),
            format!("@return the {} the API declares", comment(&object.name)),
        ],
    ));
    needs.on(WIRE_IMPORT);
    body.push_str(&format!(
        "    fun {READ_MEMBER}(value: Any?): {declared} {{\n"
    ));
    if named.is_empty() {
        // A schema declaring no member is still a document rather than anything at all, so what
        // arrived is held against that before an empty value is answered.
        body.push_str(&format!(
            "      {WIRE}.asFields(value, {})\n      return {declared}()\n    }}\n",
            literal(&object.name)
        ));
    } else {
        body.push_str(&format!(
            "      val fields = {WIRE}.asFields(value, {})\n",
            literal(&object.name)
        ));
        let mut arguments = Vec::with_capacity(named.len());
        for (_, field) in &named {
            arguments.push(format!(
                "{WIRE}.{}(fields, {}, {})",
                if field.required { "read" } else { "maybe" },
                literal(&field.name),
                reader(&field.shape, types, &mut needs, 0)?
            ));
        }
        body.push_str(&call(
            "      ",
            &format!("return {declared}"),
            &arguments,
            "",
        ));
        body.push_str("    }\n");
    }
    body.push_str("  }\n}\n");

    Ok(format!("{}{body}", needs.block()))
}

/// Fields in the one order a declaration takes them: what the document requires, then what it does
/// not, so that reading a constructor says what has to be passed before what may be left out — and
/// so that everything carrying a default of nothing sits last, which is the only order Kotlin takes
/// them in.
fn ordered_fields(object: &ObjectShape) -> Vec<&Field> {
    let mut ordered: Vec<&Field> = object
        .fields
        .iter()
        .filter(|field| field.required)
        .collect();
    ordered.extend(object.fields.iter().filter(|field| !field.required));
    ordered
}

/// The exception every problem the API reports is a kind of.
///
/// It is `sealed`: the catalogue is closed, so the set of failures the API can report is closed
/// too, and saying so is what lets a caller match on them exhaustively and be told by the compiler
/// the day the API grows one more.
fn problem_base(types: &Types) -> String {
    let mut needs = Needs::default();
    needs.on(BASE_EXCEPTION_IMPORT);
    let base = &types.problem_base;
    let document = &types.problem_document;

    let mut body = documented(
        "",
        &[
            "A failure the API answered with, whether or not it could be read as a problem."
                .to_owned(),
            String::new(),
            "@property status what the API answered under".to_owned(),
            "@property problem the document it answered, or nothing when none could be read"
                .to_owned(),
            "@param detail what to say about the failure".to_owned(),
        ],
    );

    body.push_str(&declaration(
        &format!("sealed class {base}"),
        &[
            "val status: Int".to_owned(),
            format!("val problem: {document}?"),
            "detail: String".to_owned(),
        ],
        &format!("{BASE_EXCEPTION}(detail)"),
    ));

    format!("{}{body}", needs.block())
}

/// One problem the API reports, as the exception it is raised as.
fn problem(value: &str, declared: &str, types: &Types) -> String {
    let base = &types.problem_base;
    let document = &types.problem_document;

    let mut body = documented(
        "",
        &[
            format!("The API reported `{}`.", comment(value)),
            String::new(),
            "@param status what the API answered under".to_owned(),
            "@param problem the document it answered, or nothing when none could be read"
                .to_owned(),
            "@param detail what to say about the failure".to_owned(),
        ],
    );
    body.push_str(&declaration(
        &format!("class {declared}"),
        &[
            "status: Int".to_owned(),
            format!("problem: {document}?"),
            "detail: String".to_owned(),
        ],
        &format!("{base}(status, problem, detail)"),
    ));

    body
}

/// The one member of the hierarchy that stands for a failure carrying no problem document.
///
/// A gateway, a proxy or a load balancer answers what it likes, and the API's own contract says
/// nothing about it. A caller still has to be told what the status was, so the failure is a member
/// of the same sealed set rather than something else to catch.
fn problem_unreadable(types: &Types) -> String {
    let base = &types.problem_base;
    let declared = &types.problem_unreadable;

    let mut body = documented(
        "",
        &[
            "The API answered a failure this client could not read as a problem it declares."
                .to_owned(),
            String::new(),
            "@param status what the API answered under".to_owned(),
            "@param detail what to say about the failure".to_owned(),
        ],
    );
    body.push_str(&declaration(
        &format!("class {declared}"),
        &["status: Int".to_owned(), "detail: String".to_owned()],
        &format!("{base}(status, null, detail)"),
    ));

    body
}

/// What an answer from the API means, written once for both surfaces.
///
/// Both the blocking groups and the suspending ones hand their answer here, so the decision to
/// raise and the decision to read are made in one place: a defect fixed for one surface cannot be
/// left standing in the other, because there is no other.
fn problems(types: &Types, language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    let mut needs = Needs::default();
    needs.on(WIRE_IMPORT);
    needs.on(ANSWER_IMPORT);
    needs.on(DECODE_EXCEPTION_IMPORT);

    let base = &types.problem_base;
    let unreadable = &types.problem_unreadable;
    let document = &types.problem_document;
    let catalogue = &types.catalogue;
    let discriminant = &types.problem_discriminant;

    let mut body = documented(
        "",
        &[
            "What the API answered, read as the success or the failure it is.".to_owned(),
            String::new(),
            "Every generated operation of either surface passes through here, so the two cannot \
             come to disagree about what an answer means."
                .to_owned(),
        ],
    );
    body.push_str(&format!("object {PROBLEMS_OBJECT} {{\n"));

    body.push_str(&documented(
        "  ",
        &[
            "Raises what the API reported, when what it answered was not a success.".to_owned(),
            String::new(),
            "@param status what the API answered under".to_owned(),
            "@param payload the body it answered".to_owned(),
        ],
    ));
    body.push_str(&format!(
        "  fun raiseForStatus(status: Int, payload: String) {{\n    \
         if (status >= {LOWEST_SUCCESS} && status < {LOWEST_REDIRECTION}) {{\n      \
         return\n    \
         }}\n\n    \
         val problem: {document}\n    \
         try {{\n      \
         problem = {document}.{READ_MEMBER}({WIRE}.decodePayload(payload))\n    \
         }} catch (unreadable: {DECODE_EXCEPTION}) {{\n      \
         throw {unreadable}(status, {WIRE}.unreadable(status, payload))\n    \
         }}\n    \
         throw reported(status, problem)\n  \
         }}\n\n"
    ));

    body.push_str(&format!(
        "  // One branch per value the catalogue declares, over a closed enumeration: a problem the\n  \
         // API grows and this file does not is a compilation failure rather than a value that\n  \
         // nothing matches once it is in production.\n  \
         private fun reported(status: Int, problem: {document}): {base} {{\n    \
         val detail = {WIRE}.reported(status, problem.{WRITE_MEMBER}())\n    \
         return when (problem.{discriminant}) {{\n"
    ));
    let mut branches = Vec::with_capacity(types.problems.len());
    for (value, declared) in &types.problems {
        let member = ident(value, language.casing.constant, language, limits)?;
        branches.push(branch(
            &format!("{catalogue}.{member}"),
            &format!("{declared}(status, problem, detail)"),
        ));
    }
    // The style check asks for a blank line between every branch of a match as soon as one of them
    // is written across two lines, so whether the branches are spaced out is settled by the longest
    // name the catalogue carries rather than branch by branch.
    let spaced = branches
        .iter()
        .any(|written| written.matches('\n').count() > 1);
    for (index, written) in branches.iter().enumerate() {
        if spaced && index > 0 {
            body.push('\n');
        }
        body.push_str(written);
    }
    body.push_str("    }\n  }\n\n");

    body.push_str(&format!(
        "  /** Reads what the API answered as the nothing a call of this shape answers. */\n  \
         internal fun {CHECK_HELPER}(answered: {ANSWER}) {{\n    \
         raiseForStatus(answered.status, answered.body)\n  \
         }}\n\n  \
         /** Reads what the API answered as the value a call of this shape answers. */\n  \
         internal fun <T> {READ_HELPER}(answered: {ANSWER}, reader: (Any?) -> T): T {{\n    \
         raiseForStatus(answered.status, answered.body)\n    \
         return reader({WIRE}.decodePayload(answered.body))\n  \
         }}\n}}\n"
    ));

    Ok(format!("{}{body}", needs.block()))
}

/// One branch of the exhaustive match, wrapped when naming both sides would cross the margin.
fn branch(member: &str, built: &str) -> String {
    let single = format!("      {member} -> {built}");
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }
    format!("      {member} ->\n        {built}\n")
}

/// One method per operation, grouped by the entity its operation id names.
fn group(
    entity: &Entity,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    needs.on(TRANSPORT_IMPORT);
    let declared = types.group(&entity.name, flavour)?;

    let mut body = documented(
        "",
        &[
            format!(
                "What the API declares under `{}`, issued through the transport it is handed.",
                comment(&entity.name)
            ),
            String::new(),
            flavour.described().to_owned(),
            String::new(),
            "@param transport what one request is issued through".to_owned(),
        ],
    );
    body.push_str(&format!(
        "class {declared}(private val transport: {TRANSPORT}) {{\n"
    ));

    for method in &entity.methods {
        body.push('\n');
        body.push_str(&operation(
            method, types, &mut needs, language, limits, flavour,
        )?);
    }
    body.push_str("}\n");

    Ok(format!("{}{body}", needs.block()))
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
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
    let name = method_ident(&method.verb_text, language.casing.method, language, limits)?;
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
    refuse_arguments_spelled_alike(method, &required, &optional, method.request.is_some())?;

    let answered = match method.success.as_ref() {
        Some((_, Some(shape))) => Some(annotation(shape, types, needs, 0)?),
        _ => None,
    };

    let mut tags = Vec::new();
    let mut declared_arguments = Vec::new();
    for argument in &required {
        tags.push(format!(
            "@param {} carries `{}`{}",
            argument.name,
            comment(&argument.parameter.name),
            described(argument.parameter.description.as_deref())
        ));
        declared_arguments.push(format!("{}: {}", argument.name, argument.annotated));
    }
    if let Some(shape) = method.request.as_ref() {
        let annotated = annotation(shape, types, needs, 0)?;
        tags.push(format!(
            "@param {BODY_ARGUMENT} the {annotated} the operation reads"
        ));
        declared_arguments.push(format!("{BODY_ARGUMENT}: {annotated}"));
    }
    for argument in &optional {
        tags.push(format!(
            "@param {} carries `{}`, or nothing when the caller sends none{}",
            argument.name,
            comment(&argument.parameter.name),
            described(argument.parameter.description.as_deref())
        ));
        declared_arguments.push(format!("{}: {}? = null", argument.name, argument.annotated));
    }
    match (&answered, flavour) {
        (None, _) => {}
        (Some(_), Flavour::Blocking) => tags.push("@return what the API answered".to_owned()),
        (Some(_), Flavour::Suspending) => {
            tags.push("@return what the API answered, once it has".to_owned());
        }
    }

    let mut documentation = vec![summary(method)];
    if !tags.is_empty() {
        documentation.push(String::new());
        documentation.extend(tags);
    }

    let returned = match &answered {
        None => String::new(),
        Some(declared) => format!(": {declared}"),
    };

    let mut source = documented("  ", &documentation);
    source.push_str(&call(
        "  ",
        &format!("{}fun {name}", flavour.modifier()),
        &declared_arguments,
        &format!("{returned} {{"),
    ));

    let filled = !required
        .iter()
        .any(|argument| argument.parameter.location == ParameterLocation::Path);
    source.push_str(&format!(
        "    {} path = {}\n",
        if filled { "val" } else { "var" },
        literal(operation.path.as_str())
    ));
    for argument in required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Path)
    {
        needs.on(WIRE_IMPORT);
        source.push_str(&call(
            "    ",
            "path = path.replace",
            &[
                literal(&format!("{{{}}}", argument.parameter.name)),
                format!("{WIRE}.pathSegment({})", argument.name),
            ],
            "",
        ));
    }

    needs.on(QUERY_PARAMETER_IMPORT);
    source.push_str(&format!("    val query = ArrayList<{QUERY_PARAMETER}>()\n"));
    for argument in required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
    {
        needs.on(WIRE_IMPORT);
        source.push_str(&query_pair(
            "    ",
            &argument.parameter.name,
            &argument.name,
        ));
    }
    for argument in &optional {
        needs.on(WIRE_IMPORT);
        source.push_str(&format!("    if ({} != null) {{\n", argument.name));
        source.push_str(&query_pair(
            "      ",
            &argument.parameter.name,
            &argument.name,
        ));
        source.push_str("    }\n");
    }

    let sent = match method.request.as_ref() {
        None => "null".to_owned(),
        Some(shape) => writer(shape, BODY_ARGUMENT, types, needs, 0)?,
    };
    let issued = format!(
        "transport.{}({}, path, query, {sent})",
        flavour.issues_through(),
        literal(operation.method.as_str())
    );

    match method.success.as_ref() {
        Some((_, Some(shape))) => {
            let read = reader(shape, types, needs, 0)?;
            source.push_str(&call(
                "    ",
                &format!("return {PROBLEMS_OBJECT}.{READ_HELPER}"),
                &[issued, read],
                "",
            ));
        }
        _ => {
            source.push_str(&call(
                "    ",
                &format!("{PROBLEMS_OBJECT}.{CHECK_HELPER}"),
                &[issued],
                "",
            ));
        }
    }

    source.push_str("  }\n");
    Ok(source)
}

/// One name and value of the query string, as the transport reads them.
fn query_pair(indent: &str, travels_as: &str, argument: &str) -> String {
    call(
        indent,
        "query.add",
        &[format!(
            "{QUERY_PARAMETER}({}, {WIRE}.queryValue({argument}))",
            literal(travels_as)
        )],
        "",
    )
}

/// Refuses two arguments of one method that would be spelled the same way.
///
/// Kotlin has one namespace for the arguments of a method, so a path parameter and a query
/// parameter the document spells `event-id` and `event_id` would be one argument, and whichever one
/// lost would travel carrying the other one's value.
fn refuse_arguments_spelled_alike(
    method: &Method,
    required: &[Argument<'_>],
    optional: &[Argument<'_>],
    carries_a_body: bool,
) -> Result<(), Error> {
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();
    if carries_a_body {
        claimed.insert(BODY_ARGUMENT, "the body the operation reads");
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

/// The type a value of that shape is declared as.
fn annotation(
    shape: &Shape,
    types: &Types,
    needs: &mut Needs,
    depth: usize,
) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => {
            if let Some(qualified) = scalar_import(scalar) {
                needs.on(qualified);
            }
            types.scalars.of(scalar).to_owned()
        }
        Shape::Array(inner) => {
            format!("List<{}>", annotation(inner, types, needs, depth + 1)?)
        }
        Shape::Map(inner) => {
            format!(
                "Map<String, {}>",
                annotation(inner, types, needs, depth + 1)?
            )
        }
        Shape::Enum { name, .. } => types.enumeration(name)?.to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => "Any".to_owned(),
    })
}

/// The type of the standard library an annotation of that scalar names, when it names one.
fn scalar_import(scalar: &Scalar) -> Option<&'static str> {
    match scalar {
        Scalar::Uuid => Some("java.util.UUID"),
        Scalar::DateTime => Some("java.time.OffsetDateTime"),
        Scalar::Date => Some("java.time.LocalDate"),
        Scalar::String
        | Scalar::Url
        | Scalar::Integer32
        | Scalar::Integer64
        | Scalar::Number
        | Scalar::Boolean => None,
    }
}

/// The type a parameter travelling in a path or a query is declared as.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "String",
        "integer" => "Long",
        "number" => "Double",
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
fn reader(shape: &Shape, types: &Types, needs: &mut Needs, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}::{}", scalar_reader(scalar))
        }
        Shape::Array(inner) => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}.asList({})", reader(inner, types, needs, depth + 1)?)
        }
        Shape::Map(inner) => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}.asMap({})", reader(inner, types, needs, depth + 1)?)
        }
        Shape::Enum { name, .. } => declared_reader(types.enumeration(name)?, types),
        Shape::Named(name) => declared_reader(types.schema(name)?, types),
        Shape::Object(object) => declared_reader(types.schema(&object.name)?, types),
        Shape::Json => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}::asJson")
        }
    })
}

/// What reads a type this target declares, which is a reference to the member that reads it.
///
/// Everything but a schema declaring no member reads through a companion, since that is where a
/// declaration this target writes puts what a caller reaches without an instance.
fn declared_reader(declared: &str, types: &Types) -> String {
    if types.memberless.contains(declared) {
        return format!("{declared}::{READ_MEMBER}");
    }
    format!("{declared}.Companion::{READ_MEMBER}")
}

fn scalar_reader(scalar: &Scalar) -> &'static str {
    match scalar {
        Scalar::String | Scalar::Url => "asText",
        Scalar::Uuid => "asUuid",
        Scalar::DateTime => "asMoment",
        Scalar::Date => "asDay",
        Scalar::Integer32 => "asInteger",
        Scalar::Integer64 => "asLong",
        Scalar::Number => "asDouble",
        Scalar::Boolean => "asBoolean",
    }
}

/// What writes `subject` back the way the API reads it.
fn writer(
    shape: &Shape,
    subject: &str,
    types: &Types,
    needs: &mut Needs,
    depth: usize,
) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        // How much of a moment survives being written is a decision, not a spelling: it is made
        // once, in the hand-written half, rather than emitted into every type that carries one.
        Shape::Scalar(Scalar::Uuid) => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}.writeUuid({subject})")
        }
        Shape::Scalar(Scalar::DateTime) => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}.writeMoment({subject})")
        }
        Shape::Scalar(Scalar::Date) => {
            needs.on(WIRE_IMPORT);
            format!("{WIRE}.writeDay({subject})")
        }
        Shape::Scalar(_) | Shape::Json => subject.to_owned(),
        Shape::Enum { .. } => format!("{subject}.{WIRE_MEMBER}"),
        Shape::Named(_) | Shape::Object(_) => format!("{subject}.{WRITE_MEMBER}()"),
        Shape::Array(inner) => match element_writer(inner, types, needs, depth)? {
            Written::Itself => subject.to_owned(),
            Written::Function(called) => {
                needs.on(WIRE_IMPORT);
                format!("{WIRE}.writeList({subject}, {called})")
            }
        },
        Shape::Map(inner) => match element_writer(inner, types, needs, depth)? {
            Written::Itself => subject.to_owned(),
            Written::Function(called) => {
                needs.on(WIRE_IMPORT);
                format!("{WIRE}.writeMap({subject}, {called})")
            }
        },
    })
}

/// How the items of a list or the values of a map are written back.
///
/// A value that travels as it stands needs no function at all, which is what keeps a list of
/// strings from being copied through a lambda that does nothing; and a write that is one call on
/// the value is spelled as a reference to that call rather than as a lambda around it.
enum Written {
    Itself,
    Function(String),
}

fn element_writer(
    shape: &Shape,
    types: &Types,
    needs: &mut Needs,
    depth: usize,
) -> Result<Written, Error> {
    let item = format!("item{depth}");
    let written = writer(shape, &item, types, needs, depth + 1)?;

    if written == item {
        return Ok(Written::Itself);
    }

    let referenced = match shape {
        Shape::Enum { name, .. } => Some(format!("{}::{WIRE_MEMBER}", types.enumeration(name)?)),
        Shape::Named(name) => Some(format!("{}::{WRITE_MEMBER}", types.schema(name)?)),
        Shape::Object(object) => Some(format!("{}::{WRITE_MEMBER}", types.schema(&object.name)?)),
        Shape::Scalar(Scalar::Uuid) => Some(format!("{WIRE}::writeUuid")),
        Shape::Scalar(Scalar::DateTime) => Some(format!("{WIRE}::writeMoment")),
        Shape::Scalar(Scalar::Date) => Some(format!("{WIRE}::writeDay")),
        _ => None,
    };

    Ok(Written::Function(match referenced {
        Some(called) => called,
        None => format!("{{ {item} -> {written} }}"),
    }))
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

/// The name a *method* is spelled under, which is out of the way of a wider vocabulary.
///
/// A method is the one position where `hashCode` and `toString` are refused, since a declaration
/// of either would hide the one `Any` already carries. Everywhere else they are ordinary names.
fn method_ident(
    text: &str,
    case: Case,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let _ = language;
    spell(text, case, super::kotlin_method_reserved(), limits)
}

/// One call or declaration, on one line when it fits and one argument per line when it does not.
fn call(indent: &str, head: &str, arguments: &[String], trailing: &str) -> String {
    let single = format!("{indent}{head}({}){trailing}", arguments.join(", "));
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }

    let mut source = format!("{indent}{head}(\n");
    for (index, argument) in arguments.iter().enumerate() {
        source.push_str(&format!(
            "{indent}{CONTINUATION}{argument}{}\n",
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

/// One class declaration and what it extends, wrapped the way the committed style settings ask.
///
/// The parameters stay on the line the class is named on for as long as they fit, and it is the
/// supertype that moves to a line of its own. That is the opposite of what a call does, and it is
/// what a style check reads a class signature as: the parameters are what the class *is*, and the
/// supertype is one more thing said about it.
fn declaration(head: &str, parameters: &[String], supertype: &str) -> String {
    let single = format!("{head}({}) : {supertype}", parameters.join(", "));
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }

    let named = format!("{head}({}) :", parameters.join(", "));
    if named.chars().count() <= MAX_LINE_CHARS {
        return format!("{named}\n{CONTINUATION}{supertype}\n");
    }

    let mut source = format!("{head}(\n");
    for (index, parameter) in parameters.iter().enumerate() {
        source.push_str(&format!(
            "{CONTINUATION}{parameter}{}\n",
            if index + 1 == parameters.len() {
                ""
            } else {
                ","
            }
        ));
    }
    source.push_str(&format!(") : {supertype}\n"));
    source
}

/// One assignment, on one line when it fits and under a continuation indent when it does not.
///
/// Which of the two it is written as depends only on how long it is, so the same model always
/// yields the same bytes, and a line that would have run past the margin is broken here rather than
/// left for something downstream to report.
fn assignment(indent: &str, target: &str, written: &str) -> String {
    let single = format!("{indent}{target} = {written}");
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }
    format!("{indent}{target} =\n{indent}{CONTINUATION}{written}\n")
}

/// One documentation comment, folded so that no line of it crosses [`MAX_LINE_CHARS`].
///
/// An entry spelling nothing is the blank line every documentation tool reads as separating what a
/// declaration is from the tags describing its parts.
fn documented(indent: &str, lines: &[String]) -> String {
    let mut source = format!("{indent}/**\n");
    for line in lines {
        if line.is_empty() {
            source.push_str(&format!("{indent} *\n"));
            continue;
        }
        source.push_str(&folded(indent, line));
    }
    source.push_str(&format!("{indent} */\n"));
    source
}

/// One documentation comment short enough to sit on the line it documents.
fn one_line_doc(indent: &str, text: &str) -> String {
    let single = format!("{indent}/** {text} */");
    if single.chars().count() <= MAX_LINE_CHARS {
        return format!("{single}\n");
    }
    documented(indent, &[text.to_owned()])
}

/// One line of a documentation comment, folded between words.
///
/// A single word longer than a line is written whole rather than cut in half — a name is worth more
/// than a margin.
fn folded(indent: &str, text: &str) -> String {
    let opening = format!("{indent} * ");
    let continuation = format!("{indent} *     ");

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

/// What a field or a parameter says about itself beyond the name it carries.
fn described(description: Option<&str>) -> String {
    match description {
        Some(text) => format!(": {}", comment(text)),
        None => ".".to_owned(),
    }
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

/// Snapshot text, as a documentation comment may carry it.
///
/// The snapshot is untrusted input travelling into source. A run of whitespace becomes one space so
/// nothing leaves the line the comment sits on; the characters KDoc reads as markup, as a link, as
/// the start of a tag, or as the end of the comment itself are spelled as the entities that stand
/// for them; and what is left is cut at a fixed budget.
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
    let mut written = 0usize;
    for character in collapsed.chars() {
        if written >= MAX_COMMENT_CHARS {
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
            written += 1;
            spaced = false;
        }
        written += 1;
        match character {
            '&' => rendered.push_str("&amp;"),
            '<' => rendered.push_str("&lt;"),
            '>' => rendered.push_str("&gt;"),
            '@' => rendered.push_str("&#64;"),
            '*' => rendered.push_str("&#42;"),
            '[' => rendered.push_str("&#91;"),
            ']' => rendered.push_str("&#93;"),
            plain => rendered.push(plain),
        }
    }

    if rendered.is_empty() {
        return "undocumented by the API".to_owned();
    }
    rendered
}

/// Snapshot text, as a string literal may carry it.
///
/// The dollar sign is escaped along with the usual two: a literal carrying one would otherwise read
/// as the start of a template, and whatever followed it would be compiled rather than sent.
fn literal(text: &str) -> String {
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
                rendered.push_str(&format!("\\u{:04x}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }
    rendered.push('"');
    rendered
}
