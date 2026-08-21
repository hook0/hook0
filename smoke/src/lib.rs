//! Holding every client this repository generates against a Hook0 that is really running.
//!
//! The suites each client ships are exhaustive about behaviour — retry counts, timing, truncated
//! answers, hostile headers — and they get that by talking to a server the suite starts itself.
//! What none of them can do is prove the client can talk to Hook0 at all: that the token is
//! accepted, that a problem document is shaped the way the client reads it, that a duplicated
//! ingestion is refused the way the client expects, and that a signature the *server* computed
//! verifies. A client can pass everything it has and still fail on first contact.
//!
//! What each smoke drives against that instance is its whole generated surface, one line of report
//! per operation, and [`surface`] is what holds the set of lines to the set of operations the API
//! document declares. That bijection is what stops the flows rotting: an operation the API grows
//! fails every language until somebody drives it.
//!
//! The parts live here rather than in the binary so that the questions worth asking without a
//! Docker daemon — is every client the generator declares paired with a smoke, and does the
//! bijection hold what it says it holds — are asked by a plain `cargo test`.

pub mod api;
pub mod compose;
pub mod discovery;
pub mod error;
pub mod process;
pub mod receiver;
pub mod stack;
pub mod surface;
pub mod worker;
