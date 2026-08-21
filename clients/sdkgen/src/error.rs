use thiserror::Error as ThisError;

/// Longest fragment of snapshot content an error message carries.
const PREVIEW_BYTES: usize = 64;

/// Everything that stops a snapshot from becoming an entity model.
#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
pub enum Error {
    #[error("snapshot is {size} bytes long, above the {limit} bytes accepted")]
    SnapshotTooLarge { size: u64, limit: usize },

    #[error("snapshot could not be read from {path}: {reason}")]
    SnapshotUnreadable { path: String, reason: String },

    #[error("snapshot is not a valid OpenAPI document: {0}")]
    MalformedSnapshot(String),

    #[error("snapshot declares {count} operations, above the {limit} accepted")]
    TooManyOperations { count: usize, limit: usize },

    #[error("snapshot yields {count} entities, above the {limit} accepted")]
    TooManyEntities { count: usize, limit: usize },

    #[error("entity `{entity}` carries {count} methods, above the {limit} accepted")]
    TooManyMethods {
        entity: String,
        count: usize,
        limit: usize,
    },

    #[error("operation `{method} {path}` carries {count} parameters, above the {limit} accepted")]
    TooManyParameters {
        method: String,
        path: String,
        count: usize,
        limit: usize,
    },

    #[error("identifier `{identifier}` is {size} bytes long, above the {limit} accepted")]
    IdentifierTooLong {
        identifier: String,
        size: usize,
        limit: usize,
    },

    #[error("operation id `{operation_id}` is declared on `{first}` and again on `{second}`")]
    DuplicateOperationId {
        operation_id: String,
        first: String,
        second: String,
    },

    #[error("reference `{reference}` nests deeper than the {limit} hops accepted")]
    ReferenceTooDeep { reference: String, limit: usize },

    #[error("reference `{reference}` points outside the snapshot")]
    UnresolvableReference { reference: String },

    #[error("the schema of `{subject}` could not be serialized: {reason}")]
    UnserializableSchema { subject: String, reason: String },

    #[error("the selection carries no operation, so the target would emit an empty surface")]
    EmptySelection,

    #[error("operation `{location}` was selected but declares no operation id to be named after")]
    UnnamedOperation { location: String },

    #[error(
        "parameter `{parameter}` of `{operation_id}` travels in a cookie, which a tool call has no way to carry"
    )]
    UnsupportedParameter {
        operation_id: String,
        parameter: String,
    },

    #[error(
        "the body of `{operation_id}` declares no JSON schema, so a caller would have nothing to fill in"
    )]
    BodyWithoutJsonSchema { operation_id: String },

    #[error(
        "the input schema of `{operation_id}` still points at `{reference}`, which no caller can resolve"
    )]
    UnresolvedSchema {
        operation_id: String,
        reference: String,
    },

    #[error("`{path}` is not a path a target may emit: {reason}")]
    UnsafePath { path: String, reason: String },

    #[error("path `{path}` is {size} bytes long, above the {limit} bytes accepted")]
    PathTooLong {
        path: String,
        size: usize,
        limit: usize,
    },

    #[error("path `{path}` nests {depth} segments deep, above the {limit} accepted")]
    PathTooDeep {
        path: String,
        depth: usize,
        limit: usize,
    },

    #[error("`{path}` is emitted twice, so one emission would overwrite the other")]
    DuplicateEmittedPath { path: String },

    #[error("the emission carries {count} files, above the {limit} accepted")]
    TooManyEmittedFiles { count: usize, limit: usize },

    #[error("the emission carries {size} bytes, above the {limit} bytes accepted")]
    EmissionTooLarge { size: usize, limit: usize },

    #[error("the target at {path} could not be read: {reason}")]
    TargetUnreadable { path: String, reason: String },

    #[error("the target at {path} could not be written: {reason}")]
    TargetUnwritable { path: String, reason: String },

    #[error("the target at {root} holds more than the {limit} entries a walk of it accepts")]
    TargetTooLarge { root: String, limit: usize },

    #[error(
        "`{path}` is named as a test file, and regeneration would delete it along with the coverage it carries"
    )]
    TestFileUnderTarget { path: String },

    #[error("`{identifier}` splits into {count} words, above the {limit} accepted")]
    TooManyWords {
        identifier: String,
        count: usize,
        limit: usize,
    },

    #[error("no identifier can be rendered from words that carry no character")]
    EmptyIdentifier,

    #[error("`{name}` spells `{identifier}`, which no identifier can be: {reason}")]
    UnspellableName {
        name: String,
        identifier: String,
        reason: String,
    },

    #[error("`{path}` is emitted as a file, yet `{nested}` needs it to be a directory")]
    EmittedPathCollision { path: String, nested: String },

    #[error("the keyword list a target escapes against is unusable: {reason}")]
    UnusableReservedWords { reason: String },

    #[error("the regeneration command a banner would name is unusable: {reason}")]
    UnusableCommand { reason: String },

    #[error("snapshot yields {count} object types, above the {limit} accepted")]
    TooManySchemas { count: usize, limit: usize },

    #[error("object type `{object}` carries {count} fields, above the {limit} accepted")]
    TooManyFields {
        object: String,
        count: usize,
        limit: usize,
    },

    #[error("the enum of `{subject}` carries {count} values, above the {limit} accepted")]
    TooManyEnumValues {
        subject: String,
        count: usize,
        limit: usize,
    },

    #[error("the schema of `{subject}` nests deeper than the {limit} levels accepted")]
    SchemaTooDeep { subject: String, limit: usize },

    #[error(
        "the schema of `{subject}` declares `{keyword}`, which the type language does not model"
    )]
    UnmodelledSchemaKeyword { subject: String, keyword: String },

    #[error("the schema of `{subject}` is not a JSON object")]
    UnreadableSchema { subject: String },

    #[error("the schema of `{subject}` declares neither a type, nor an enum, nor a reference")]
    UntypedSchema { subject: String },

    #[error("the schema of `{subject}` declares the type `{declared}`, which no target renders")]
    UnknownSchemaType { subject: String, declared: String },

    #[error("the schema of `{subject}` declares an enum that is not a closed list of strings")]
    UnmodelledEnum { subject: String },

    #[error(
        "the schema of `{subject}` declares both named fields and free-form ones, so what a target should declare for it is ambiguous"
    )]
    AmbiguousObjectSchema { subject: String },

    #[error(
        "component schema `{schema}` is not an object, so no target has a type to declare for it"
    )]
    NonObjectSchema { schema: String },

    #[error("the type name `{name}` is derived from `{first}` and again from `{second}`")]
    SchemaNameCollision {
        name: String,
        first: String,
        second: String,
    },

    #[error(
        "no selected operation answers a status of {threshold} or above with a named schema, so the error contract cannot be discovered"
    )]
    UndiscoverableErrorSchema { threshold: u16 },

    #[error("selected operations disagree on what they answer errors with: {candidates}")]
    DisagreeingErrorSchemas { candidates: String },

    #[error(
        "the error response of `{operation}` points at no named schema, so no target has a name to declare it under"
    )]
    UnnamedErrorSchema { operation: String },

    #[error(
        "error schema `{schema}` declares no closed string enum, so the problems it can carry cannot be listed"
    )]
    ErrorSchemaWithoutCatalogue { schema: String },

    #[error(
        "error schema `{schema}` declares several closed string enums, so which one lists its problems is ambiguous: {members}"
    )]
    AmbiguousErrorCatalogue { schema: String, members: String },

    #[error("operation `{operation}` answers under `{status}`, which names no single status code")]
    UnmodelledResponseStatus { operation: String, status: String },

    #[error("security scheme `{scheme}` is {declared}, which no target carries")]
    UnsupportedSecurityScheme { scheme: String, declared: String },

    #[error(
        "operation `{operation}` requires the security scheme `{scheme}`, which the snapshot does not declare"
    )]
    UnknownSecurityScheme { operation: String, scheme: String },
}

/// Shortens snapshot content down to what an error message may carry.
///
/// The snapshot is untrusted input: its identifiers and parser messages travel into logs, so they
/// are cut at a fixed budget rather than echoed whole.
pub(crate) fn preview(value: &str) -> String {
    if value.len() <= PREVIEW_BYTES {
        return value.to_owned();
    }

    let mut end = PREVIEW_BYTES;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }

    format!("{}…", &value[..end])
}
