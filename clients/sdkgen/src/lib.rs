//! Reads a Hook0 OpenAPI snapshot and derives the entity model the targets are built from.
//!
//! Entities and their methods come out of the `entity.verb` convention the operation ids already
//! follow, narrowed to the operations carrying the tag a target selects — [`PUBLIC_TAG`] for the
//! SDKs, [`MCP_TAG`] for the MCP server. Nothing about the API surface is written down here:
//! entities, methods and parameters are only ever what the snapshot declares.
//!
//! Targets that emit source read the model rather than the document, so the snapshot is parsed
//! once, here, and nowhere else. What there is to emit is [`targets`], a registry of plain values:
//! one driver walks it, writing or checking every target, and nothing names a target twice.
//!
//! Every input is bounded by [`Limits`], and a snapshot crossing a ceiling is rejected with the
//! ceiling it crossed rather than trimmed down to fit.

pub mod conformance;
pub mod emit;
mod error;
pub mod identifier;
mod limits;
pub mod model;
mod snapshot;
pub mod targets;

pub use conformance::{ConformanceError, Corpus, CorpusLimits};
pub use emit::{
    CommentStyle, EmittedFile, FileTree, Ownership, RelativePath, WriteReport, banner, write_target,
};
pub use error::Error;
pub use identifier::{Case, Casing, Escape, ReservedWords};
pub use limits::Limits;
pub use model::{
    ApiModel, Entity, EntityModel, ErrorModel, Field, IGNORED_KEYWORDS, MODELLED_KEYWORDS, Method,
    Nonconformity, ObjectShape, ProblemCatalogue, Scalar, Scheme, SecurityModel, Shape,
    UnconventionalOperation, Verb,
};
pub use snapshot::{
    HttpMethod, Operation, PUBLIC_TAG, Parameter, ParameterLocation, RequestBody, Snapshot,
};
/// The MCP target, under the path it answered to before it joined the registry.
pub use targets::mcp;
pub use targets::mcp::MCP_TAG;
