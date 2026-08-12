//! Reads a Hook0 OpenAPI snapshot and derives the entity model the targets are built from.
//!
//! Entities and their methods come out of the `entity.verb` convention the operation ids already
//! follow, narrowed to the operations carrying the tag a target selects — [`PUBLIC_TAG`] for the
//! SDKs, [`MCP_TAG`] for the MCP server. Nothing about the API surface is written down here:
//! entities, methods and parameters are only ever what the snapshot declares.
//!
//! Targets that emit source read the model rather than the document, so the snapshot is parsed
//! once, here, and nowhere else: [`mcp::tool_definitions`] writes the tool table the Hook0 MCP
//! server compiles.
//!
//! Every input is bounded by [`Limits`], and a snapshot crossing a ceiling is rejected with the
//! ceiling it crossed rather than trimmed down to fit.

mod error;
mod limits;
pub mod mcp;
mod model;
mod snapshot;

pub use error::Error;
pub use limits::Limits;
pub use mcp::MCP_TAG;
pub use model::{Entity, EntityModel, Method, Nonconformity, UnconventionalOperation, Verb};
pub use snapshot::{
    HttpMethod, Operation, PUBLIC_TAG, Parameter, ParameterLocation, RequestBody, Snapshot,
};
