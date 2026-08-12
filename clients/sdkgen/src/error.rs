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
