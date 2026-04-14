//! SQL layer for the subscription health monitor, split by domain.
//!
//! - [`buckets`] — bucket lifecycle: upsert into open buckets, close full
//!   buckets, drop buckets older than retention.
//! - [`cursor`] — evaluation cursor (read/advance) that bookmarks the last
//!   processed `request_attempt` timestamp.
//! - [`deltas`] — per-tick aggregation of recent `request_attempt` rows,
//!   grouped by subscription.
//! - [`events`] — health event writes, suspect detection, and resolved-event
//!   cleanup.
//! - [`subscription_state`] — subscription state writes (disable, cached
//!   failure percent reset) and the `SubscriptionHealth` aggregate that the
//!   state machine consumes.

mod buckets;
mod cursor;
mod deltas;
mod events;
mod subscription_state;

pub use buckets::*;
pub use cursor::*;
pub use deltas::*;
pub use events::*;
pub use subscription_state::*;
