//! Reads a Hook0 OpenAPI snapshot and derives the entity model SDK targets are built from.
//!
//! Entities and their methods come out of the `entity.verb` convention the operation ids already
//! follow, narrowed to the operations carrying the [`PUBLIC_TAG`] when the snapshot uses it.
//! Nothing about the API surface is written down here: entities, methods and parameters are only
//! ever what the snapshot declares.
//!
//! Every input is bounded by [`Limits`], and a snapshot crossing a ceiling is rejected with the
//! ceiling it crossed rather than trimmed down to fit.

mod error;
mod limits;
mod model;
mod snapshot;

pub use error::Error;
pub use limits::Limits;
pub use model::{Entity, EntityModel, Method, Nonconformity, UnconventionalOperation, Verb};
pub use snapshot::{HttpMethod, Operation, PUBLIC_TAG, Parameter, ParameterLocation, Snapshot};
