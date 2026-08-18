//! Emits the generated half of the Rust SDK.
//!
//! The crate is published to crates.io with no copy of the OpenAPI snapshot beside it, so the
//! types, the problems and the request layer travel as committed source rather than as a build
//! artefact. Everything the API declares — one type per named schema, one closed enum per
//! enumeration it names, one variant per problem the error contract can report, one method per
//! operation — is written here; everything the API does not declare — how a request reaches the
//! network, how a send is retried, how a webhook signature is verified — is hand-written beside
//! this directory and never regenerated.
//!
//! The two halves meet at one seam and nowhere else, and that seam is a trait declared *here*: the
//! hand-written half implements it, so this module reaches nothing that carries a socket, and
//! nothing beside it knows what the API declares.
//!
//! What is written is already formatted. A signature is emitted on one line when it fits inside the
//! width code is laid out at and one argument per line when it does not, every statement is short
//! enough to stand on its own line, and imports are written one name per line, sorted, in groups a
//! formatter never moves a name across — so the bytes emitted are the bytes `rustfmt` would leave.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, a scalar no type covers — stops the emission
//! rather than yielding a smaller SDK.

use std::collections::{BTreeMap, BTreeSet};

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, spell};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Scalar, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{Contract, Decoding, LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "rust";

/// Where the generated half of the crate lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written.
const ROOT: &str = "clients/rust/src/generated";

/// The modules this target writes, each one holding one layer of the surface.
const ROOT_MODULE: &str = "mod";
const MODELS_MODULE: &str = "models";
const ERRORS_MODULE: &str = "errors";
const API_MODULE: &str = "api";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix the type carrying a failure the API described carries, so a problem and a type spelling
/// the same word stay apart.
const FAILURE_SUFFIX: &str = "Error";

/// Longest fragment of a snapshot description a doc comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Widest line code is laid out at, which is what decides whether a signature is written on one
/// line or one argument per line.
const MAX_LINE_CHARS: usize = 100;

/// Widest the arguments of a call are written at before they are laid out one per line, which is
/// narrower than a whole line.
const MAX_ARGUMENT_CHARS: usize = 60;

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// How far one level of a block is indented.
const INDENT: &str = "    ";

/// The names the scaffolding below declares, which no type of the document may answer to as well.
///
/// They are claimed alongside everything the model names, so a schema called `Transport` is
/// reported as the collision it is rather than emitted as a crate that does not compile. Sorted, so
/// that reading the list says what is taken.
const SCAFFOLDING: [&str; 6] = [
    "RequestError",
    "Transport",
    "path_segment",
    "preview",
    "problem_for",
    "query_value",
];

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its
        // module with it instead of lingering as an orphan nothing imports.
        ownership: Ownership::Directory,
        contract: Contract::Whole,
        decoding: Decoding::Modelled,
        language: super::rust(),
        emit,
    }
}

/// Everything the generated half of the crate is made of.
fn emit(language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;
    let banner = banner(language.comment, &update_command(NAME), &limits)?;

    let enums = model.enumerations(&limits)?;
    let types = Types::read(model, &enums, language, &limits)?;

    let files = vec![
        file(ROOT_MODULE, &banner, &root(&types), language, &limits)?,
        file(
            MODELS_MODULE,
            &banner,
            &models(model, &enums, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            ERRORS_MODULE,
            &banner,
            &errors(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            API_MODULE,
            &banner,
            &requests(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
    ];

    FileTree::build(files, &limits)
}

/// One file: the banner, then the body.
fn file(
    stem: &str,
    banner: &str,
    body: &str,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<EmittedFile, Error> {
    let name = format!("{stem}.{}", language.extension);

    // Every layer is written as a run of declarations each ending in a blank line, which would
    // leave the file ending in one; a file ends where its last declaration does.
    Ok(EmittedFile {
        path: RelativePath::build(&name, limits)?,
        contents: format!("{banner}{}\n", body.trim_end()),
    })
}

/// Every name the generated modules declare, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a crate that will not compile. They share one namespace because the module below
/// hands every one of them out from a single place.
struct Types {
    /// Type each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Type each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Type carrying a failure the API described.
    failure: String,
    /// Enum the discriminant of the error contract carries.
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
            let spelled = ident(name, language.casing.type_name, language, limits)?;
            schemas.insert(name.clone(), claim(spelled, name)?);
        }

        let mut declared_enums = BTreeMap::new();
        for name in enums.keys() {
            let spelled = ident(name, language.casing.type_name, language, limits)?;
            declared_enums.insert(name.clone(), claim(spelled, name)?);
        }

        let problem_enum = enum_of(&model.errors, &declared_enums)?;

        let named = ident(
            &model.errors.schema,
            language.casing.type_name,
            language,
            limits,
        )?;
        let failure = claim(format!("{named}{FAILURE_SUFFIX}"), &model.errors.schema)?;

        let mut groups = BTreeMap::new();
        for entity in model.entities.entities() {
            let stem = ident(&entity.name, language.casing.type_name, language, limits)?;
            let declared = claim(format!("{stem}{GROUP_SUFFIX}"), &entity.name)?;
            groups.insert(entity.name.clone(), declared);
        }

        Ok(Self {
            schemas,
            enums: declared_enums,
            failure,
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

    /// Everything the module above hands out, by the module that declares it.
    fn exports(&self) -> Vec<(&'static str, BTreeSet<&str>)> {
        let mut requesting: BTreeSet<&str> = self.groups.values().map(String::as_str).collect();
        requesting.insert("Transport");

        let failures = BTreeSet::from([self.failure.as_str(), "RequestError", "problem_for"]);

        let mut declared: BTreeSet<&str> = self.schemas.values().map(String::as_str).collect();
        declared.extend(self.enums.values().map(String::as_str));

        vec![
            (API_MODULE, requesting),
            (ERRORS_MODULE, failures),
            (MODELS_MODULE, declared),
        ]
    }
}

/// The variant each value of a closed list is read as.
///
/// Two values spelling one variant would be one variant, and whichever one lost would be a silent
/// mis-decoding, so the collision stops the emission.
fn variants<'a>(
    values: impl IntoIterator<Item = &'a String>,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<BTreeMap<String, String>, Error> {
    let mut spelled: BTreeMap<String, &str> = BTreeMap::new();
    let mut read = BTreeMap::new();

    for value in values {
        let variant = ident(value, language.casing.type_name, language, limits)?;
        if let Some(first) = spelled.insert(variant.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&variant),
                first: preview(first),
                second: preview(value),
            });
        }
        read.insert(value.clone(), variant);
    }

    Ok(read)
}

/// The enum the discriminant of the error contract is read through.
///
/// The catalogue is the values of one closed list of strings the error schema declares, so the enum
/// already exists among the types: it is found rather than declared twice.
fn enum_of(errors: &ErrorModel, enums: &BTreeMap<String, String>) -> Result<String, Error> {
    let discriminant = discriminant_field(errors)?;

    let Shape::Enum { name, .. } = &discriminant.shape else {
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

/// What a module reaches for, gathered while its body is written rather than guessed afterwards.
///
/// An import a Rust module does not use is a warning, and a warning is what this repository's build
/// denies, so this is the difference between a module and one that does not compile.
#[derive(Debug, Default)]
struct Needs {
    /// Paths outside this target, which the crates the types are spelled in carry.
    paths: BTreeSet<&'static str>,
    /// Names the layer holding the types declares, which only a layer above it reaches for.
    below: BTreeSet<String>,
}

impl Needs {
    fn path(&mut self, name: &'static str) {
        self.paths.insert(name);
    }

    fn declared(&mut self, name: &str) {
        self.below.insert(name.to_owned());
    }

    /// What a module of the layer holding the types reaches for, one name per line and sorted, or
    /// nothing at all when it reaches for nothing.
    fn block(&self) -> String {
        if self.paths.is_empty() {
            return String::new();
        }

        let mut block = String::new();
        for path in &self.paths {
            block.push_str(&format!("use {path};\n"));
        }
        block.push('\n');
        block
    }

    /// The same, for the names the layer below declares, written in the group the imports of this
    /// target sit in so that nothing is ever sorted across the blank line under it.
    fn below_block(&self) -> String {
        let mut block = String::new();
        for name in &self.below {
            block.push_str(&format!("use super::{MODELS_MODULE}::{name};\n"));
        }
        block
    }
}

/// The module handing out everything the layers below declare.
///
/// The layers themselves stay private and every name is re-exported one at a time: a glob would let
/// a schema the API starts declaring reach the crate's surface without anybody deciding it should,
/// and would hide the day two of them answer to one name.
fn root(types: &Types) -> String {
    let mut source = String::from(
        "//! Everything the API document describes: one type per schema it declares, one closed\n\
         //! enum per enumeration it names, one variant per problem it can report, and one method\n\
         //! per operation, grouped by the entity its operation id names.\n\
         //!\n\
         //! Everything the document does not describe — reaching the network, retrying a send,\n\
         //! verifying a webhook signature — is hand-written beside this directory and never\n\
         //! regenerated. The two meet at the [`Transport`] trait declared here, which the\n\
         //! hand-written half implements.\n\n",
    );

    for (module, _) in types.exports() {
        source.push_str(&format!("mod {module};\n"));
    }

    for (module, names) in types.exports() {
        source.push('\n');
        for name in names {
            source.push_str(&format!("pub use {module}::{name};\n"));
        }
    }

    source
}

/// The types every body the API reads and writes is made of.
fn models(
    model: &ApiModel,
    enums: &BTreeMap<String, Vec<String>>,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    let mut body = String::new();

    for (name, values) in enums {
        body.push_str(&enumeration(
            types.enumeration(name)?,
            values,
            language,
            limits,
        )?);
    }
    for (name, object) in &model.schemas {
        body.push_str(&structure(
            types.schema(name)?,
            object,
            types,
            &mut needs,
            language,
            limits,
        )?);
    }

    let header = "//! The types the API carries, one declaration per schema and per enumeration it \
                  names.\n\n";

    Ok(format!("{header}{}{body}", needs.block()))
}

/// One closed list of strings, as an enum of its own and one variant per value it admits.
///
/// The wire spelling travels twice: once as the rename that decides how a value is read and written
/// back, and once as an exhaustive match, so a caller has the text without going through a
/// serializer.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let spelled = variants(values, language, limits)?;
    let ordered: Vec<(&String, &String)> = values
        .iter()
        .filter_map(|value| spelled.get(value).map(|variant| (value, variant)))
        .collect();

    let mut source = format!(
        "/// One of the values the API answers with.\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]\n\
         pub enum {declared} {{\n"
    );
    for (value, variant) in &ordered {
        source.push_str(&format!(
            "    /// The `{}` the API answers with.\n    #[serde(rename = {})]\n    {variant},\n",
            comment(value),
            literal(value)
        ));
    }
    source.push_str("}\n\n");

    source.push_str(&format!(
        "impl {declared} {{\n    \
         /// The text this value travels as.\n    \
         pub const fn as_str(&self) -> &'static str {{\n        \
         match self {{\n"
    ));
    for (value, variant) in &ordered {
        source.push_str(&format!(
            "            Self::{variant} => {},\n",
            literal(value)
        ));
    }
    source.push_str("        }\n    }\n}\n\n");

    source.push_str(&format!(
        "impl std::fmt::Display for {declared} {{\n    \
         fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        \
         formatter.write_str(self.as_str())\n    \
         }}\n\
         }}\n\n"
    ));

    Ok(source)
}

/// One named schema, as the type a caller reads and writes.
///
/// The name a member travels under is written verbatim as a rename, so whatever the emitter had to
/// do to spell that name as an identifier never reaches the wire.
fn structure(
    declared: &str,
    object: &ObjectShape,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut source = format!(
        "/// The `{}` the API declares.\n\
         #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\n",
        comment(&object.name)
    );

    if object.fields.is_empty() {
        source.push_str(&format!("pub struct {declared} {{}}\n\n"));
        return Ok(source);
    }
    source.push_str(&format!("pub struct {declared} {{\n"));

    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    for field in &object.fields {
        let name = ident(&field.name, language.casing.field, language, limits)?;
        if let Some(first) = claimed.insert(name.clone(), &field.name) {
            return Err(Error::SchemaNameCollision {
                name: preview(&name),
                first: preview(first),
                second: preview(&field.name),
            });
        }

        let declared_type = annotation(&field.shape, field.required, types, needs, 0)?;
        source.push_str(&format!(
            "    /// `{}`{}\n    #[serde(rename = {})]\n",
            comment(&field.name),
            described(field.description.as_deref()),
            literal(&field.name)
        ));
        if !field.required {
            source.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
        }
        source.push_str(&format!("    pub {name}: {declared_type},\n"));
    }
    source.push_str("}\n\n");

    Ok(source)
}

/// What a member says about itself beyond the name it carries, when the document says anything.
fn described(description: Option<&str>) -> String {
    match description {
        Some(text) => format!(": {}", comment(text)),
        None => ".".to_owned(),
    }
}

/// The failures the API reports, and the one error every generated method answers with.
fn errors(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    let discriminant = discriminant_field(&model.errors)?;
    let read = ident(
        &model.errors.discriminant,
        language.casing.field,
        language,
        limits,
    )?;
    let failure = &types.failure;
    let catalogue = &types.problem_enum;

    // The discriminant is what says whether a body named a problem at all, and a document may
    // declare it as a member it does not require.
    let named = if discriminant.required {
        format!("problem.as_ref().map(|problem| problem.{read})")
    } else {
        format!("problem.as_ref().and_then(|problem| problem.{read})")
    };

    let mut imported = BTreeSet::from([catalogue.as_str(), schema]);
    imported.remove("");

    let mut source = String::from(
        "//! The failures the API reports: the document it describes one with, and the one error\n\
         //! every generated method answers instead of what it was asked for.\n\n",
    );
    for name in &imported {
        source.push_str(&format!("use super::{MODELS_MODULE}::{name};\n"));
    }

    source.push_str(&format!(
        "\n/// Lowest status the API answers a success under.\n\
         const LOWEST_SUCCESS: u16 = {LOWEST_SUCCESS};\n\n\
         /// Lowest status that is no longer a success.\n\
         const LOWEST_REDIRECTION: u16 = {LOWEST_REDIRECTION};\n\n\
         /// Longest fragment of an answer a message carries.\n\
         ///\n\
         /// Bodies are written by a server this crate does not control, so they are cut at a fixed\n\
         /// budget rather than echoed whole into whatever the caller logs.\n\
         const MAX_PREVIEW_BYTES: usize = 256;\n\n"
    ));

    source.push_str(&format!(
        "/// What the API answered when it did not answer a success.\n\
         ///\n\
         /// Every problem the document names is one of these, told apart by the kind it carries;\n\
         /// the document the API sent, when it sent one this crate can read, is beside it. A body\n\
         /// naming no problem still reaches a caller as one of these, carrying no kind.\n\
         #[derive(Debug, Clone)]\n\
         pub struct {failure} {{\n    \
         /// Status the API answered under.\n    \
         pub status: u16,\n    \
         /// Problem the API named, absent when the body named none.\n    \
         pub kind: Option<{catalogue}>,\n    \
         /// Document the API answered, absent when it answered none this crate can read.\n    \
         pub problem: Option<{schema}>,\n    \
         /// What to say about the failure, as much of the API's answer as fits included.\n    \
         pub detail: String,\n\
         }}\n\n\
         impl std::fmt::Display for {failure} {{\n    \
         fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        \
         formatter.write_str(&self.detail)\n    \
         }}\n\
         }}\n\n\
         impl std::error::Error for {failure} {{}}\n\n"
    ));

    source.push_str(&format!(
        "/// Everything a generated method answers instead of what it was asked for.\n\
         #[derive(Debug)]\n\
         pub enum RequestError {{\n    \
         /// The request never reached the API, or its answer never came back.\n    \
         Transport(Box<dyn std::error::Error + Send + Sync>),\n    \
         /// The API answered a failure, and described it.\n    \
         ///\n    \
         /// Held behind a pointer, so that every method answers a small error whatever the\n    \
         /// document declares a failure body to carry.\n    \
         Api(Box<{failure}>),\n    \
         /// The API answered something this crate cannot read.\n    \
         Unreadable {{\n        \
         /// Status the API answered under.\n        \
         status: u16,\n        \
         /// What was answered, as much of it as fits included.\n        \
         detail: String,\n    \
         }},\n    \
         /// The body of the request could not be written.\n    \
         Unwritable {{\n        \
         /// Why it could not be written.\n        \
         detail: String,\n    \
         }},\n\
         }}\n\n"
    ));

    source.push_str(
        "impl RequestError {\n    \
         /// Reports that the request never reached the API.\n    \
         pub fn transport<E>(cause: E) -> Self\n    \
         where\n        \
         E: std::error::Error + Send + Sync + 'static,\n    \
         {\n        \
         Self::Transport(Box::new(cause))\n    \
         }\n\n    \
         /// Reports a request body this crate could not write.\n    \
         pub fn unwritable(cause: serde_json::Error) -> Self {\n        \
         Self::Unwritable {\n            \
         detail: cause.to_string(),\n        \
         }\n    \
         }\n\n    \
         /// Reports an answer this crate could not read.\n    \
         pub fn unreadable(status: u16, payload: &[u8], cause: &serde_json::Error) -> Self {\n        \
         let seen = preview(payload);\n        \
         let unread = format!(\"the API answered {status} with an unreadable body\");\n        \
         let detail = format!(\"{unread}: {cause} ({seen})\");\n        \
         Self::Unreadable { status, detail }\n    \
         }\n\
         }\n\n\
         impl std::fmt::Display for RequestError {\n    \
         fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        \
         match self {\n            \
         Self::Transport(cause) => write!(formatter, \"the API was not reached: {cause}\"),\n            \
         Self::Api(failure) => failure.fmt(formatter),\n            \
         Self::Unreadable { detail, .. } => formatter.write_str(detail),\n            \
         Self::Unwritable { detail } => formatter.write_str(detail),\n        \
         }\n    \
         }\n\
         }\n\n\
         impl std::error::Error for RequestError {}\n\n",
    );

    source.push_str(&format!(
        "/// The failure the API reported, and nothing at all when what it answered was a success.\n\
         pub fn problem_for(status: u16, payload: &[u8]) -> Option<{failure}> {{\n    \
         if (LOWEST_SUCCESS..LOWEST_REDIRECTION).contains(&status) {{\n        \
         return None;\n    \
         }}\n\n    \
         let problem: Option<{schema}> = serde_json::from_slice(payload).ok();\n    \
         let seen = preview(payload);\n    \
         let kind = {named};\n\n    \
         Some({failure} {{\n        \
         status,\n        \
         kind,\n        \
         problem,\n        \
         detail: format!(\"the API answered {{status}}: {{seen}}\"),\n    \
         }})\n\
         }}\n\n"
    ));

    source.push_str(
        "/// As much of an answer as a message may carry, with whatever was not text left out.\n\
         fn preview(payload: &[u8]) -> String {\n    \
         let text = String::from_utf8_lossy(payload);\n    \
         let mut kept = String::with_capacity(MAX_PREVIEW_BYTES);\n\n    \
         for character in text.chars() {\n        \
         if kept.len() + character.len_utf8() > MAX_PREVIEW_BYTES {\n            \
         kept.push('…');\n            \
         break;\n        \
         }\n        \
         kept.push(character);\n    \
         }\n\n    \
         kept\n\
         }\n",
    );

    Ok(source)
}

/// One method per operation, grouped by the entity its operation id names.
fn requests(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    let mut helpers = Helpers::default();
    let mut body = String::new();

    for entity in model.entities.entities() {
        body.push_str(&group(
            entity,
            types,
            &mut needs,
            &mut helpers,
            language,
            limits,
        )?);
    }

    let mut header = String::from(
        "//! One method per operation the API declares, grouped by the entity its operation id\n\
         //! names, and the seam every one of them is issued through.\n\
         //!\n\
         //! How many arguments a method takes is the document's business rather than this crate's:\n\
         //! an operation declaring a dozen parameters is asked for with a dozen arguments.\n\
         #![allow(clippy::too_many_arguments)]\n\n",
    );
    header.push_str(&format!(
        "use super::{ERRORS_MODULE}::RequestError;\n\
         use super::{ERRORS_MODULE}::problem_for;\n"
    ));
    header.push_str(&needs.below_block());
    header.push('\n');
    header.push_str(&needs.block());

    header.push_str(
        "/// What a generated method issues its request through.\n\
         ///\n\
         /// It is declared here rather than imported: the half of this crate that reaches the\n\
         /// network is hand-written and implements it, so neither half has to name what the other\n\
         /// declares, and nothing here carries a socket.\n\
         pub trait Transport {\n    \
         /// What this transport reports when it could not reach the API at all.\n    \
         type Error: std::error::Error + Send + Sync + 'static;\n\n    \
         /// Issues one request, answering the status the API replied with and the body it sent.\n    \
         fn request(\n        \
         &self,\n        \
         method: &str,\n        \
         path: &str,\n        \
         query: &[(&str, String)],\n        \
         body: Option<Vec<u8>>,\n    \
         ) -> impl std::future::Future<Output = Result<(u16, Vec<u8>), Self::Error>> + Send;\n\
         }\n\n",
    );
    header.push_str(&helpers.block());

    Ok(format!("{header}{body}"))
}

/// The helpers the emitted methods call, kept out of a file that needs none of them.
#[derive(Debug, Default)]
struct Helpers {
    path: bool,
    query: bool,
}

impl Helpers {
    fn block(&self) -> String {
        let mut block = String::new();

        if self.path || self.query {
            block.push_str(
                "/// Writes a value the way a request line carries it, which is not always how it\n\
                 /// is printed.\n\
                 fn query_value(value: &dyn std::fmt::Display) -> String {\n    \
                 value.to_string()\n\
                 }\n\n",
            );
        }
        if self.path {
            block.push_str(
                "/// Writes a value as one segment of a path, with nothing left in it that could\n\
                 /// name another one.\n\
                 fn path_segment(value: &dyn std::fmt::Display) -> String {\n    \
                 let rendered = query_value(value);\n    \
                 let mut escaped = String::with_capacity(rendered.len());\n\n    \
                 for byte in rendered.bytes() {\n        \
                 let bare = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');\n        \
                 if bare {\n            \
                 escaped.push(char::from(byte));\n        \
                 } else {\n            \
                 escaped.push_str(&format!(\"%{byte:02X}\"));\n        \
                 }\n    \
                 }\n\n    \
                 escaped\n\
                 }\n\n",
            );
        }

        block
    }
}

fn group(
    entity: &Entity,
    types: &Types,
    needs: &mut Needs,
    helpers: &mut Helpers,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let declared = types.group(&entity.name)?;

    let mut source = format!(
        "/// What the API declares under `{}`.\n\
         ///\n\
         /// Every method of it is issued through the transport it is handed.\n\
         #[derive(Debug, Clone)]\n\
         pub struct {declared}<T> {{\n    \
         transport: T,\n\
         }}\n\n\
         impl<T: Transport> {declared}<T> {{\n    \
         /// Reaches what the API declares under `{}`.\n    \
         pub fn new(transport: T) -> Self {{\n        \
         Self {{ transport }}\n    \
         }}\n",
        comment(&entity.name),
        comment(&entity.name)
    );

    for method in &entity.methods {
        source.push_str(&operation(method, types, needs, helpers, language, limits)?);
    }
    source.push_str("}\n\n");

    Ok(source)
}

/// One argument of a method: how it is spelled, what it carries, and what the document says it is.
struct Argument<'a> {
    name: String,
    annotated: String,
    parameter: &'a Parameter,
}

fn operation(
    method: &Method,
    types: &Types,
    needs: &mut Needs,
    helpers: &mut Helpers,
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
    let mut required = Vec::new();
    let mut optional = Vec::new();
    for parameter in path_parameters
        .iter()
        .copied()
        .chain(query_parameters.iter().copied())
    {
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

    let returned = match method.success.as_ref() {
        Some((_, Some(shape))) => Some(annotation(shape, true, types, needs, 0)?),
        _ => None,
    };
    let answered = match returned.as_deref() {
        Some(declared) => format!("Result<{declared}, RequestError>"),
        None => "Result<(), RequestError>".to_owned(),
    };

    let mut arguments = vec!["&self".to_owned()];
    for argument in &required {
        arguments.push(format!("{}: {}", argument.name, argument.annotated));
    }
    if let Some(shape) = method.request.as_ref() {
        arguments.push(format!(
            "body: {}",
            annotation(shape, true, types, needs, 0)?
        ));
    }
    for argument in &optional {
        arguments.push(format!("{}: Option<{}>", argument.name, argument.annotated));
    }

    let mut source = format!(
        "\n{INDENT}/// `{}`, `{} {}`.{}\n",
        comment(&method.operation_id),
        operation.method,
        comment(&operation.path),
        summary(method)
    );
    source.push_str(&signature(&name, &arguments, &answered));

    let body = format!("{INDENT}{INDENT}");
    let held = if path_parameters.is_empty() {
        ""
    } else {
        "mut "
    };
    source.push_str(&binding(
        &body,
        &format!("let {held}path ="),
        &format!("{}.to_owned()", literal(&operation.path)),
    ));
    for argument in required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Path)
    {
        helpers.path = true;
        source.push_str(&call(
            &body,
            "path = path.replace(",
            &[
                literal(&format!("{{{}}}", argument.parameter.name)),
                format!("&path_segment(&{})", argument.name),
            ],
            ");",
        ));
    }

    // What the API requires is written into the collection as it is built, and only what may be
    // left out is pushed afterwards: a collection built empty and immediately filled is one a
    // linter asks to be written the other way round.
    let mut entries = Vec::new();
    for argument in required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
    {
        helpers.query = true;
        entries.push(format!(
            "({}, query_value(&{}))",
            literal(&argument.parameter.name),
            argument.name
        ));
    }

    let held = if optional.is_empty() { "" } else { "mut " };
    source.push_str(&collection(&body, held, &entries));

    for argument in &optional {
        helpers.query = true;
        let name = &argument.name;
        source.push_str(&format!("{body}if let Some({name}) = {name} {{\n"));
        source.push_str(&call(
            &format!("{body}{INDENT}"),
            "query.push((",
            &[
                literal(&argument.parameter.name),
                format!("query_value(&{name})"),
            ],
            "));",
        ));
        source.push_str(&format!("{body}}}\n"));
    }

    let sent = match method.request.as_ref() {
        Some(_) => {
            source.push_str(
                "        let body = serde_json::to_vec(&body).map_err(RequestError::unwritable)?;\n",
            );
            "Some(body)"
        }
        None => "None",
    };
    source.push_str(&format!(
        "        let issued = self.transport.request({}, &path, &query, {sent});\n        \
         let (status, payload) = issued.await.map_err(RequestError::transport)?;\n\n        \
         if let Some(failure) = problem_for(status, &payload) {{\n            \
         return Err(RequestError::Api(Box::new(failure)));\n        \
         }}\n\n",
        literal(operation.method.as_str())
    ));

    match returned {
        None => source.push_str("        Ok(())\n    }\n"),
        Some(_) => source.push_str(
            "        let read = serde_json::from_slice(&payload);\n        \
             read.map_err(|cause| RequestError::unreadable(status, &payload, &cause))\n    \
             }\n",
        ),
    }

    Ok(source)
}

/// A binding, laid out the way code is laid out: on one line when it fits inside the width lines
/// are written at, and with what is bound on the line under it when it does not.
fn binding(indent: &str, opened: &str, value: &str) -> String {
    let inline = format!("{indent}{opened} {value};");
    if inline.chars().count() <= MAX_LINE_CHARS {
        return format!("{inline}\n");
    }

    format!("{indent}{opened}\n{indent}{INDENT}{value};\n")
}

/// A call, laid out the way code is laid out: on one line when its arguments fit inside the width
/// arguments are written at and the line itself fits, one argument per line when they do not.
fn call(indent: &str, opened: &str, arguments: &[String], closed: &str) -> String {
    let joined = arguments.join(", ");
    let inline = format!("{indent}{opened}{joined}{closed}");
    let fits =
        joined.chars().count() <= MAX_ARGUMENT_CHARS && inline.chars().count() <= MAX_LINE_CHARS;
    if fits {
        return format!("{inline}\n");
    }

    let mut broken = format!("{indent}{opened}\n");
    for argument in arguments {
        broken.push_str(&format!("{indent}{INDENT}{argument},\n"));
    }
    broken.push_str(&format!("{indent}{closed}\n"));
    broken
}

/// The query a request carries, built with everything the API requires already in it.
fn collection(indent: &str, held: &str, entries: &[String]) -> String {
    let opened = format!("let {held}query: Vec<(&str, String)> =");

    if entries.is_empty() {
        return binding(indent, &opened, "Vec::new()");
    }

    let joined = entries.join(", ");
    if joined.chars().count() <= MAX_ARGUMENT_CHARS {
        return binding(indent, &opened, &format!("vec![{joined}]"));
    }

    let mut broken = format!("{indent}{opened} vec![\n");
    for entry in entries {
        broken.push_str(&format!("{indent}{INDENT}{entry},\n"));
    }
    broken.push_str(&format!("{indent}];\n"));
    broken
}

/// The declaration line of a method, laid out the way code is laid out: on one line when it fits
/// inside the width lines are written at, one argument per line when it does not.
fn signature(name: &str, arguments: &[String], answered: &str) -> String {
    let inline = format!(
        "{INDENT}pub async fn {name}({}) -> {answered} {{",
        arguments.join(", ")
    );
    if inline.chars().count() <= MAX_LINE_CHARS {
        return format!("{inline}\n");
    }

    let mut broken = format!("{INDENT}pub async fn {name}(\n");
    for argument in arguments {
        broken.push_str(&format!("{INDENT}{INDENT}{argument},\n"));
    }
    broken.push_str(&format!("{INDENT}) -> {answered} {{\n"));
    broken
}

/// The type a value of that shape carries.
///
/// Optionality is membership in `required` and nothing else: a member the document does not require
/// is carried as an absence the language spells out, never as a value standing for one.
fn annotation(
    shape: &Shape,
    required: bool,
    types: &Types,
    needs: &mut Needs,
    depth: usize,
) -> Result<String, Error> {
    deep_enough(depth)?;

    let declared = match shape {
        Shape::Scalar(scalar) => {
            for path in scalar_paths(scalar) {
                needs.path(path);
            }
            types.scalars.of(scalar).to_owned()
        }
        Shape::Array(inner) => {
            format!("Vec<{}>", annotation(inner, true, types, needs, depth + 1)?)
        }
        Shape::Map(inner) => {
            let values = annotation(inner, true, types, needs, depth + 1)?;
            needs.path("std::collections::HashMap");
            format!("HashMap<String, {values}>")
        }
        Shape::Enum { name, .. } => {
            let declared = types.enumeration(name)?;
            needs.declared(declared);
            declared.to_owned()
        }
        Shape::Named(name) => {
            let declared = types.schema(name)?;
            needs.declared(declared);
            declared.to_owned()
        }
        Shape::Object(object) => {
            let declared = types.schema(&object.name)?;
            needs.declared(declared);
            declared.to_owned()
        }
        Shape::Json => {
            needs.path("serde_json::Value");
            "Value".to_owned()
        }
    };

    if required {
        return Ok(declared);
    }
    Ok(format!("Option<{declared}>"))
}

/// The paths a type is reached through, when it is not one the language carries on its own.
fn scalar_paths(scalar: &Scalar) -> &'static [&'static str] {
    match scalar {
        Scalar::Uuid => &["uuid::Uuid"],
        Scalar::DateTime => &["chrono::DateTime", "chrono::Utc"],
        Scalar::Date => &["chrono::NaiveDate"],
        Scalar::Url => &["url::Url"],
        Scalar::String
        | Scalar::Integer32
        | Scalar::Integer64
        | Scalar::Number
        | Scalar::Boolean => &[],
    }
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains. Text is borrowed
/// rather than owned, since nothing keeps it past the line that writes it into the request.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "&str",
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
        Some(text) => format!("\n{INDENT}///\n{INDENT}/// {}", comment(text)),
        None => String::new(),
    }
}

/// Snapshot text, as a doc comment may carry it.
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
                rendered.push_str(&format!("\\u{{{:04x}}}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }
    rendered.push('"');
    rendered
}
