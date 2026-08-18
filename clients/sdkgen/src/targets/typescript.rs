//! Emits the generated half of the TypeScript SDK.
//!
//! The package is published to npm with no copy of the OpenAPI snapshot beside it, so the types,
//! the problems and the request layer travel as committed source rather than as a build artefact.
//! Everything the API declares — one interface per named schema, one closed set of values per
//! enumeration it names, one kind per problem the error contract can report, one method per
//! operation — is written here; everything the API does not declare — how a request reaches the
//! network, how a send is retried, how a webhook signature is verified — is hand-written beside
//! this directory and never regenerated.
//!
//! The two halves meet at one seam and nowhere else, and that seam is an interface declared *here*:
//! the hand-written half answers to it, so nothing here knows what a socket is and nothing beside
//! it knows what the API declares.
//!
//! A type declared here *is* the JSON the API sends. Members carry the names the document spells
//! them with rather than the names the language would prefer, and every value the document states
//! as text stays text: what comes out of a parser is already the declared type, so there is no
//! layer between reading a body and holding one, and nothing to keep in step with the document.
//!
//! What is written is already formatted. A signature is emitted on one line when it fits inside the
//! width the repository's formatter prints at and one argument per line when it does not, objects
//! are written open so a formatter leaves them open, and quotes are picked the way it picks them —
//! so the bytes emitted are the bytes `prettier` would leave.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, a scalar no type covers — stops the emission
//! rather than yielding a smaller SDK.

use std::collections::{BTreeMap, BTreeSet};

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, checked_words, escape, render, spell};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{Contract, Decoding, LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "typescript";

/// Where the generated half of the package lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written.
const ROOT: &str = "clients/typescript/src/generated";

/// The modules this target writes, each one holding one layer of the surface.
const ROOT_MODULE: &str = "index";
const MODELS_MODULE: &str = "models";
const ERRORS_MODULE: &str = "errors";
const API_MODULE: &str = "api";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix the class carrying a failure the API described carries, so a problem and a type spelling
/// the same word stay apart.
const FAILURE_SUFFIX: &str = "Error";

/// Longest fragment of a snapshot description a doc comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Widest line the repository's formatter prints at, which is what decides whether a signature or a
/// collection is written on one line or one entry per line.
const MAX_LINE_CHARS: usize = 100;

/// How far one level of a block is indented.
const INDENT: &str = "  ";

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// The names the scaffolding below declares, which no type of the document may answer to as well.
///
/// They are claimed alongside everything the model names, so a schema called `Transport` is
/// reported as the collision it is rather than emitted as a package that does not compile. Sorted,
/// so that reading the list says what is taken.
const SCAFFOLDING: [&str; 7] = [
    "Transport",
    "TransportRequest",
    "TransportResponse",
    "pathSegment",
    "queryValue",
    "raiseForStatus",
    "readPayload",
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
        language: super::typescript(),
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
            &errors(model, &types)?,
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
        contents: format!("{banner}\n{}\n", body.trim_end()),
    })
}

/// Every name the generated modules declare, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a package that will not compile. They share one namespace because the module
/// below hands every one of them out from a single place.
struct Types {
    /// Type each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Type each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Class carrying a failure the API described.
    failure: String,
    /// Type the discriminant of the error contract carries.
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
        requesting.insert("TransportRequest");
        requesting.insert("TransportResponse");

        let failures = BTreeSet::from([self.failure.as_str(), "raiseForStatus", "readPayload"]);

        let mut declared: BTreeSet<&str> = self.schemas.values().map(String::as_str).collect();
        declared.extend(self.enums.values().map(String::as_str));

        vec![
            (API_MODULE, requesting),
            (ERRORS_MODULE, failures),
            (MODELS_MODULE, declared),
        ]
    }
}

/// The type the discriminant of the error contract is read through.
///
/// The catalogue is the values of one closed list of strings the error schema declares, so the type
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
#[derive(Debug, Default)]
struct Needs {
    below: BTreeSet<String>,
}

impl Needs {
    fn declared(&mut self, name: &str) {
        self.below.insert(name.to_owned());
    }
}

/// One import, laid out the way the formatter lays it out: on one line when it fits, one name per
/// line when it does not.
fn import(names: &BTreeSet<String>, module: &str) -> String {
    if names.is_empty() {
        return String::new();
    }

    let joined: Vec<&str> = names.iter().map(String::as_str).collect();
    let inline = format!("import {{ {} }} from './{module}';", joined.join(", "));
    if inline.chars().count() <= MAX_LINE_CHARS {
        return format!("{inline}\n");
    }

    let mut broken = String::from("import {\n");
    for name in &joined {
        broken.push_str(&format!("{INDENT}{name},\n"));
    }
    broken.push_str(&format!("}} from './{module}';\n"));
    broken
}

/// The module handing out everything the layers below declare.
///
/// The layers are re-exported one name at a time: a star would let a schema the API starts
/// declaring reach the package's surface without anybody deciding it should, and would hide the day
/// two of them answer to one name.
fn root(types: &Types) -> String {
    let mut source = String::from(
        "/**\n\
         \x20* Everything the API document describes: one type per schema it declares, one closed set of\n\
         \x20* values per enumeration it names, one kind per problem it can report, and one method per\n\
         \x20* operation, grouped by the entity its operation id names.\n\
         \x20*\n\
         \x20* Everything the document does not describe — reaching the network, retrying a send,\n\
         \x20* verifying a webhook signature — is hand-written beside this directory and never\n\
         \x20* regenerated. The two meet at the `Transport` interface declared here, which the\n\
         \x20* hand-written half answers to.\n\
         \x20*/\n\n",
    );

    for (module, names) in types.exports() {
        for name in names {
            source.push_str(&format!("export {{ {name} }} from './{module}';\n"));
        }
        source.push('\n');
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
    let mut body = String::from(
        "/** The types the API carries, one declaration per schema and per enumeration it names. */\n\n",
    );

    for (name, values) in enums {
        body.push_str(&enumeration(
            types.enumeration(name)?,
            values,
            language,
            limits,
        )?);
    }
    for (name, object) in &model.schemas {
        body.push_str(&structure(types.schema(name)?, object, types)?);
    }

    Ok(body)
}

/// One closed list of strings, as the values it admits and the type of one of them.
///
/// It is an object of constants rather than an enumeration of the language's own: an enumeration
/// declares a type that exists at run time, which nothing that reads JSON can produce, whereas this
/// is exactly the text the API sends, usable both as a value to compare against and as a type.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut members: BTreeMap<String, &String> = BTreeMap::new();
    let mut source =
        format!("/** One of the values the API answers with. */\nexport const {declared} = {{\n");

    for value in values {
        let member = ident(value, language.casing.constant, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        source.push_str(&format!("{INDENT}{member}: {},\n", literal(value)));
    }
    source.push_str("} as const;\n\n");

    source.push_str("/** One of the values the API answers with. */\n");
    source.push_str(&binding(
        "",
        &format!("export type {declared} ="),
        &format!("(typeof {declared})[keyof typeof {declared}]"),
    ));
    source.push('\n');

    Ok(source)
}

/// One named schema, as the type a caller reads and writes.
///
/// A member carries the name the document spells it with, so what a parser hands back is already
/// one of these and nothing has to be renamed on the way in or out.
fn structure(declared: &str, object: &ObjectShape, types: &Types) -> Result<String, Error> {
    let mut source = format!(
        "/** The `{}` the API declares. */\nexport interface {declared} {{\n",
        comment(&object.name)
    );

    for field in &object.fields {
        let held = annotation(&field.shape, types, 0)?;
        let optional = if field.required { "" } else { "?" };
        source.push_str(&format!(
            "{INDENT}/** `{}`{} */\n{INDENT}readonly {}{optional}: {held};\n",
            comment(&field.name),
            described(field.description.as_deref()),
            member(&field.name)
        ));
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

/// The failures the API reports, and what every generated method reads its answer through.
fn errors(model: &ApiModel, types: &Types) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    // The discriminant is read off a document a parser handed back, so it is named the way the
    // document names it rather than the way the language would.
    let read = member(&model.errors.discriminant);
    let failure = &types.failure;
    let catalogue = &types.problem_enum;

    let mut imported = BTreeSet::new();
    imported.insert(catalogue.clone());
    imported.insert(schema.to_owned());

    let mut source = String::from(
        "/**\n\
         \x20* The failures the API reports: the document it describes one with, and what every\n\
         \x20* generated method reads an answer through.\n\
         \x20*/\n\n",
    );
    source.push_str(&import(&imported, MODELS_MODULE));

    source.push_str(&format!(
        "\n/** Lowest status the API answers a success under. */\n\
         const LOWEST_SUCCESS = {LOWEST_SUCCESS};\n\n\
         /** Lowest status that is no longer a success. */\n\
         const LOWEST_REDIRECTION = {LOWEST_REDIRECTION};\n\n\
         /**\n\
         \x20* Longest fragment of an answer a message carries. Bodies are written by a server this\n\
         \x20* package does not control, so they are cut at a fixed budget rather than echoed whole\n\
         \x20* into whatever the caller logs.\n\
         \x20*/\n\
         const MAX_PREVIEW_CHARS = 256;\n\n"
    ));

    source.push_str(&format!(
        "/**\n\
         \x20* What the API answered when it did not answer a success.\n\
         \x20*\n\
         \x20* Every problem the document names is one of these, told apart by the kind it carries;\n\
         \x20* the document the API sent, when it sent one this package can read, is beside it. A body\n\
         \x20* naming no problem still reaches a caller as one of these, carrying no kind.\n\
         \x20*/\n\
         export class {failure} extends Error {{\n\
         {INDENT}/** Status the API answered under. */\n\
         {INDENT}readonly status: number;\n\n\
         {INDENT}/** Problem the API named, absent when the body named none. */\n\
         {INDENT}readonly kind?: {catalogue};\n\n\
         {INDENT}/** Document the API answered, absent when it answered none this package can read. */\n\
         {INDENT}readonly problem?: {schema};\n\n\
         {INDENT}/** Reports what the API answered, whether or not it could be read as a problem. */\n\
         {INDENT}constructor(status: number, problem: {schema} | undefined, detail: string) {{\n\
         {INDENT}{INDENT}super(detail);\n\
         {INDENT}{INDENT}this.name = {};\n\
         {INDENT}{INDENT}this.status = status;\n\
         {INDENT}{INDENT}if (problem !== undefined) {{\n\
         {INDENT}{INDENT}{INDENT}this.kind = problem.{read};\n\
         {INDENT}{INDENT}{INDENT}this.problem = problem;\n\
         {INDENT}{INDENT}}}\n\
         {INDENT}}}\n\
         }}\n\n",
        literal(failure)
    ));

    source.push_str(&format!(
        "/** Throws what the API reported, when what it answered was not a success. */\n\
         export function raiseForStatus(status: number, payload: string): void {{\n\
         {INDENT}if (status >= LOWEST_SUCCESS && status < LOWEST_REDIRECTION) {{\n\
         {INDENT}{INDENT}return;\n\
         {INDENT}}}\n\n\
         {INDENT}const seen = preview(payload);\n\
         {INDENT}const problem = parse(payload) as {schema} | undefined;\n\
         {INDENT}throw new {failure}(status, problem, `the API answered ${{status}}: ${{seen}}`);\n\
         }}\n\n"
    ));

    source.push_str(&format!(
        "/** Reads what the API answered, or throws when it answered something this package cannot read. */\n\
         export function readPayload<T>(status: number, payload: string): T {{\n\
         {INDENT}const read = parse(payload);\n\
         {INDENT}if (read === undefined) {{\n\
         {INDENT}{INDENT}const seen = preview(payload);\n\
         {INDENT}{INDENT}const detail = `the API answered ${{status}} with an unreadable body: ${{seen}}`;\n\
         {INDENT}{INDENT}throw new {failure}(status, undefined, detail);\n\
         {INDENT}}}\n\n\
         {INDENT}return read as T;\n\
         }}\n\n"
    ));

    source.push_str(&format!(
        "/** What the text carries, or nothing at all when it carries no document. */\n\
         function parse(payload: string): unknown {{\n\
         {INDENT}try {{\n\
         {INDENT}{INDENT}return JSON.parse(payload) as unknown;\n\
         {INDENT}}} catch {{\n\
         {INDENT}{INDENT}return undefined;\n\
         {INDENT}}}\n\
         }}\n\n\
         /** As much of an answer as a message may carry. */\n\
         function preview(payload: string): string {{\n\
         {INDENT}if (payload.length <= MAX_PREVIEW_CHARS) {{\n\
         {INDENT}{INDENT}return payload;\n\
         {INDENT}}}\n\n\
         {INDENT}return `${{payload.slice(0, MAX_PREVIEW_CHARS)}}…`;\n\
         }}\n"
    ));

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
        "/**\n\
         \x20* One method per operation the API declares, grouped by the entity its operation id\n\
         \x20* names, and the seam every one of them is issued through.\n\
         \x20*/\n\n",
    );
    header.push_str(&import(
        &BTreeSet::from(["raiseForStatus".to_owned(), "readPayload".to_owned()]),
        ERRORS_MODULE,
    ));
    header.push_str(&import(&needs.below, MODELS_MODULE));

    header.push_str(&format!(
        "\n/** One request a generated method issues. */\n\
         export interface TransportRequest {{\n\
         {INDENT}/** Method the request is issued with. */\n\
         {INDENT}readonly method: string;\n\n\
         {INDENT}/** Path the request is issued against, every parameter already written into it. */\n\
         {INDENT}readonly path: string;\n\n\
         {INDENT}/** Query the request carries, in the order the document declares it. */\n\
         {INDENT}readonly query: readonly [string, string][];\n\n\
         {INDENT}/** Body the request carries, absent when it carries none. */\n\
         {INDENT}readonly body?: string;\n\
         }}\n\n\
         /** What the API answered. */\n\
         export interface TransportResponse {{\n\
         {INDENT}/** Status the API answered under. */\n\
         {INDENT}readonly status: number;\n\n\
         {INDENT}/** Body the API answered, as it travelled. */\n\
         {INDENT}readonly payload: string;\n\
         }}\n\n\
         /**\n\
         \x20* What a generated method issues its request through.\n\
         \x20*\n\
         \x20* It is declared here rather than imported: the half of this package that reaches the\n\
         \x20* network is hand-written and answers to it, so neither half has to name what the other\n\
         \x20* declares, and nothing here carries a socket.\n\
         \x20*/\n\
         export interface Transport {{\n\
         {INDENT}/** Issues one request, answering the status the API replied with and the body it sent. */\n\
         {INDENT}request(request: TransportRequest): Promise<TransportResponse>;\n\
         }}\n\n"
    ));
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
            block.push_str(&format!(
                "/** Writes a value the way a request line carries it. */\n\
                 function queryValue(value: string | number | boolean): string {{\n\
                 {INDENT}return String(value);\n\
                 }}\n\n"
            ));
        }
        if self.path {
            block.push_str(&format!(
                "/** Writes a value as one segment of a path, with nothing left in it that could name another. */\n\
                 function pathSegment(value: string | number | boolean): string {{\n\
                 {INDENT}return encodeURIComponent(queryValue(value));\n\
                 }}\n\n"
            ));
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
        "/**\n\
         \x20* What the API declares under `{}`.\n\
         \x20*\n\
         \x20* Every method of it is issued through the transport it is handed.\n\
         \x20*/\n\
         export class {declared} {{\n\
         {INDENT}private readonly transport: Transport;\n\n\
         {INDENT}/** Reaches what the API declares under `{}`. */\n\
         {INDENT}constructor(transport: Transport) {{\n\
         {INDENT}{INDENT}this.transport = transport;\n\
         {INDENT}}}\n",
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
        Some((_, Some(shape))) => {
            let held = annotation(shape, types, 0)?;
            declare(shape, types, needs)?;
            Some(held)
        }
        _ => None,
    };
    let answered = match returned.as_deref() {
        Some(held) => format!("Promise<{held}>"),
        None => "Promise<void>".to_owned(),
    };

    let mut arguments = Vec::new();
    for argument in &required {
        arguments.push(format!("{}: {}", argument.name, argument.annotated));
    }
    if let Some(shape) = method.request.as_ref() {
        arguments.push(format!("body: {}", annotation(shape, types, 0)?));
        declare(shape, types, needs)?;
    }
    for argument in &optional {
        arguments.push(format!("{}?: {}", argument.name, argument.annotated));
    }

    let mut source = format!(
        "\n{INDENT}/**\n{INDENT} * `{}`, `{} {}`.{}\n{INDENT} */\n",
        comment(&method.operation_id),
        operation.method,
        comment(&operation.path),
        summary(method)
    );
    source.push_str(&signature(&name, &arguments, &answered));

    let body = format!("{INDENT}{INDENT}");
    let held = if path_parameters.is_empty() {
        "const"
    } else {
        "let"
    };
    source.push_str(&format!(
        "{body}{held} path = {};\n",
        literal(&operation.path)
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
                format!("pathSegment({})", argument.name),
            ],
            ");",
        ));
    }

    // What the API requires is written into the collection as it is built, and only what may be
    // left out is pushed afterwards.
    let mut entries = Vec::new();
    for argument in required
        .iter()
        .filter(|argument| argument.parameter.location == ParameterLocation::Query)
    {
        helpers.query = true;
        entries.push(format!(
            "[{}, queryValue({})]",
            literal(&argument.parameter.name),
            argument.name
        ));
    }
    source.push_str(&collection(&body, &entries));

    for argument in &optional {
        helpers.query = true;
        let name = &argument.name;
        source.push_str(&format!("{body}if ({name} !== undefined) {{\n"));
        source.push_str(&call(
            &format!("{body}{INDENT}"),
            "query.push([",
            &[
                literal(&argument.parameter.name),
                format!("queryValue({name})"),
            ],
            "]);",
        ));
        source.push_str(&format!("{body}}}\n"));
    }

    source.push_str(&format!(
        "{body}const issued = await this.transport.request({{\n"
    ));
    source.push_str(&format!(
        "{body}{INDENT}method: {},\n{body}{INDENT}path,\n{body}{INDENT}query,\n",
        literal(operation.method.as_str())
    ));
    if method.request.is_some() {
        source.push_str(&format!("{body}{INDENT}body: JSON.stringify(body),\n"));
    }
    source.push_str(&format!("{body}}});\n"));
    source.push_str(&format!(
        "{body}raiseForStatus(issued.status, issued.payload);\n"
    ));

    if let Some(held) = returned {
        source.push_str(&format!(
            "{body}return readPayload<{held}>(issued.status, issued.payload);\n"
        ));
    }
    source.push_str(&format!("{INDENT}}}\n"));

    Ok(source)
}

/// The declaration line of a method, laid out the way the formatter lays it out: on one line when
/// it fits inside the width lines are printed at, one argument per line when it does not.
///
/// No comma follows the last argument of a broken list: the repository's formatter writes trailing
/// commas only where a collection admits one.
fn signature(name: &str, arguments: &[String], answered: &str) -> String {
    let inline = format!(
        "{INDENT}async {name}({}): {answered} {{",
        arguments.join(", ")
    );
    if inline.chars().count() <= MAX_LINE_CHARS {
        return format!("{inline}\n");
    }

    let mut broken = format!("{INDENT}async {name}(\n");
    for (index, argument) in arguments.iter().enumerate() {
        let comma = if index + 1 == arguments.len() {
            ""
        } else {
            ","
        };
        broken.push_str(&format!("{INDENT}{INDENT}{argument}{comma}\n"));
    }
    broken.push_str(&format!("{INDENT}): {answered} {{\n"));
    broken
}

/// A binding, laid out the way the formatter lays it out: on one line when it fits inside the width
/// lines are printed at, and with what is bound on the line under it when it does not.
fn binding(indent: &str, opened: &str, value: &str) -> String {
    let inline = format!("{indent}{opened} {value};");
    if inline.chars().count() <= MAX_LINE_CHARS {
        return format!("{inline}\n");
    }

    format!("{indent}{opened}\n{indent}{INDENT}{value};\n")
}

/// A call, laid out the way the formatter lays it out: on one line when it fits, one argument per
/// line when it does not.
fn call(indent: &str, opened: &str, arguments: &[String], closed: &str) -> String {
    let inline = format!("{indent}{opened}{}{closed}", arguments.join(", "));
    if inline.chars().count() <= MAX_LINE_CHARS {
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
fn collection(indent: &str, entries: &[String]) -> String {
    let opened = format!("{indent}const query: [string, string][] =");

    if entries.is_empty() {
        return format!("{opened} [];\n");
    }

    let inline = format!("{opened} [{}];", entries.join(", "));
    if inline.chars().count() <= MAX_LINE_CHARS {
        return format!("{inline}\n");
    }

    let mut broken = format!("{opened} [\n");
    for entry in entries {
        broken.push_str(&format!("{indent}{INDENT}{entry},\n"));
    }
    broken.push_str(&format!("{indent}];\n"));
    broken
}

/// The type a value of that shape carries.
///
/// Optionality is membership in `required` and nothing else, and it is written on the member rather
/// than in the type: a member the document does not require is one an answer may not carry, which
/// is not the same as one carrying nothing.
fn annotation(shape: &Shape, types: &Types, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => types.scalars.of(scalar).to_owned(),
        Shape::Array(inner) => format!("{}[]", annotation(inner, types, depth + 1)?),
        Shape::Map(inner) => format!("Record<string, {}>", annotation(inner, types, depth + 1)?),
        Shape::Enum { name, .. } => types.enumeration(name)?.to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => "unknown".to_owned(),
    })
}

/// Records every name of the layer holding the types that a value of that shape reaches for.
fn declare(shape: &Shape, types: &Types, needs: &mut Needs) -> Result<(), Error> {
    match shape {
        Shape::Array(inner) | Shape::Map(inner) => declare(inner, types, needs),
        Shape::Enum { name, .. } => {
            needs.declared(types.enumeration(name)?);
            Ok(())
        }
        Shape::Named(name) => {
            needs.declared(types.schema(name)?);
            Ok(())
        }
        Shape::Object(object) => {
            needs.declared(types.schema(&object.name)?);
            Ok(())
        }
        Shape::Scalar(_) | Shape::Json => Ok(()),
    }
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "string",
        "integer" | "number" => "number",
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
/// A method sits where a member sits, and every word the language keeps for itself is a name a
/// member may carry, so the keyword vocabulary does not apply here: an operation the document calls
/// `delete` is a method called `delete`. What does apply is the far smaller set of names every
/// object already answers to, and a name that rendered to no identifier at all.
fn method_name(text: &str, language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    let words = checked_words(text, limits)?;
    let rendered = render(&words, language.casing.method);

    Ok(escape(&rendered, super::typescript_method_reserved()))
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

/// A member of a type, spelled exactly as the document spells it, quoted when that is not a name a
/// member may carry unquoted.
///
/// Nothing is renamed: what the parser hands back is the type, so a member spelled otherwise would
/// need a conversion on the way in and another on the way out.
fn member(name: &str) -> String {
    let mut characters = name.chars();
    let opens = characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '$');
    let bare =
        opens && characters.all(|held| held.is_ascii_alphanumeric() || held == '_' || held == '$');

    if bare {
        return name.to_owned();
    }
    literal(name)
}

/// What a method says about itself, from what the document says about the operation.
fn summary(method: &Method) -> String {
    let described = method
        .operation
        .summary
        .as_ref()
        .or(method.operation.description.as_ref());

    match described {
        Some(text) => format!("\n{INDENT} *\n{INDENT} * {}", comment(text)),
        None => String::new(),
    }
}

/// Snapshot text, as a doc comment may carry it.
///
/// The snapshot is untrusted input travelling into source: a run of whitespace becomes one space so
/// nothing leaves the line the comment sits on, what would close the comment early is spelled
/// otherwise, and what is left is cut at a fixed budget.
fn comment(text: &str) -> String {
    let spaced: String = text
        .chars()
        .map(|character| {
            if character.is_whitespace() || character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    // A comment carrying what closes a comment would end it early, leaving the rest of the
    // description outside it and inside the source.
    let collapsed = spaced.replace("*/", "* /");

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
/// The quote is picked the way the repository's formatter picks it: the one that has to be escaped
/// the fewest times, and the one it prefers when neither wins.
fn literal(text: &str) -> String {
    let single = text.matches('\'').count();
    let double = text.matches('"').count();
    let quote = if single > double { '"' } else { '\'' };

    let mut rendered = String::from(quote);
    for character in text.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            held if held == quote => {
                rendered.push('\\');
                rendered.push(held);
            }
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
    rendered.push(quote);
    rendered
}
