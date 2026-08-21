//! Proving that the examples in the SDK reference are examples of something real.
//!
//! A documentation snippet is the part of the documentation that rots first and most visibly: a
//! reader copies one, it does not compile, and the page loses its authority. This crate takes
//! every fenced example of `documentation/reference/sdk`, assembles it against the client it
//! claims to use, and hands it to that language's own toolchain.
//!
//! Nothing here is a list. The pages come from the directory, the languages come from the
//! generator's registry, and how a language is proven comes from a manifest beside its harness.
//! What that buys is the case this exists for: somebody adds a target, writes its page, and
//! forgets to make its examples runnable — and the build says so, instead of passing because the
//! new language was in nobody's table.

pub mod discovery;
pub mod error;
pub mod harness;
pub mod limits;
pub mod manifest;
pub mod page;
pub mod project;

pub use discovery::{Documentation, Language, discover};
pub use error::Error;
pub use manifest::{Manifest, Proof};
pub use page::{Example, Page};
pub use project::{Proven, prove};

/// Where the SDK reference lives, relative to the repository.
pub const SDK_REFERENCE: &str = "documentation/reference/sdk";

/// One generated client, as this checker needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRoot {
    /// The name the target answers to, which is also the language a fence is opened with and the
    /// name of its harness directory.
    pub name: String,
    /// The client's own directory, relative to the repository.
    pub client: String,
}

/// Everything the generator writes, as this checker needs it.
///
/// This is the only place the set of languages comes from. A target added to the registry is a
/// language this checker starts asking about on the same commit.
pub fn registry() -> Vec<TargetRoot> {
    hook0_sdkgen::targets::targets()
        .iter()
        .map(|target| TargetRoot {
            name: target.name.to_owned(),
            client: client_of(target.root),
        })
        .collect()
}

/// The client a target's emission root belongs to.
///
/// A target lands inside its client — `clients/go/generated`, `clients/mcp/src/server` — so the
/// client is the two components above whatever the target owns. Reading it off the registry rather
/// than spelling `clients/<name>` here keeps the two from drifting the day a client moves.
fn client_of(root: &str) -> String {
    root.split('/').take(2).collect::<Vec<&str>>().join("/")
}
