//! Where the repository is.

use std::path::PathBuf;

pub const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn tree() -> PathBuf {
    PathBuf::from(REPOSITORY)
}
