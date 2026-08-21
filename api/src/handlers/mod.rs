//! HTTP handlers of the Hook0 API.
//!
//! An operation reaches the generated clients only when its
//! `#[api_v2_operation(...)]` carries the `sdk` tag. Writing that tag
//! publishes the operation into every SDK we generate and turns its name, its
//! parameters and its responses into a contract nobody gets to break
//! afterwards, so it belongs on the product our users integrate against and not
//! on the control plane the dashboard drives. Tagged operations spell their
//! `operation_id` as `entity.verb` — `list`, `get`, `create`, `update`,
//! `delete` — and answer errors with `Problem`; `sdk_surface.rs` checks both
//! against the document the application serves.
//!
//! Changing any served operation also makes the committed OpenAPI snapshot fail
//! until somebody adopts the change. `api/README.md` covers when the tag is
//! warranted and how to adopt the snapshot.

pub mod applications;
pub mod auth;
pub mod email_preferences;
pub mod environment_variables;
pub mod errors;
pub mod event_types;
pub mod events;
pub mod events_per_day;
pub mod instance;
pub mod organizations;
pub mod registrations;
pub mod request_attempts;
pub mod responses;
pub mod service_token;
pub mod subscriptions;

#[cfg(feature = "application-secret-compatibility")]
pub mod application_secrets;

#[cfg(test)]
mod sdk_surface;
