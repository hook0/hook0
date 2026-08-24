//! Every lockfile this repository commits still resolves against the manifests beside it.
//!
//! Most of the Rust here is one workspace, whose lockfile every build maintains. A few crates sit
//! outside it deliberately — this harness, the projects that prove the documentation, and the
//! smokes that drive a generated client — so that the root workspace stays the list of what the
//! repository builds and releases. Each of those carries a committed lockfile of its own, and
//! nothing at the root reaches it: `cargo update` there leaves it exactly where it was.
//!
//! That costs nothing until one of them reaches back into this repository by path, because then
//! the manifest it points at moves underneath it. A dependency bump took the Rust client's `uuid`
//! to `1.25.0` while the smoke beside it still pinned `1.24.1`, and nothing satisfies `^1.25.0`
//! and `1.24.1` at once, so the two could not be resolved together at all. The bump itself was
//! green. What went red was every later merge request that happened to touch `api/`, since that is
//! what triggers the job the smoke runs in, and the error named `uuid` rather than the change that
//! had moved it.
//!
//! Asking cargo is the whole check. Reading the lockfiles and comparing versions here would be a
//! second resolver and a worse one: the root lockfile legitimately holds several
//! semver-incompatible copies of the same crate, so "these two files disagree about a version" is
//! not the question and answering it produces a page of differences that are all fine. `--locked`
//! asks the only question there is — whether the lockfile as committed still says what the
//! manifests need — and it answers for a pin that conflicts and a lockfile that has merely gone
//! stale alike. `--depth 0` is there because the tree is not wanted, only the resolution that has
//! to happen before one can be printed.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use hook0_smoke::process::capture;

/// The most lockfiles walked to. The repository commits a handful; past this is a walk that has
/// escaped rather than a repository that grew.
const MAX_FILES: usize = 64;

/// How far below the repository root a lockfile is looked for.
const MAX_DEPTH: usize = 6;

/// Directories that hold no lockfile of this repository's, only copies of one.
const NOT_SOURCE: [&str; 4] = [".git", "node_modules", "target", "vendor"];

/// How long one resolution is given before it is killed and the deadline reported as the reason.
const WITHIN: Duration = Duration::from_secs(180);

/// The most of cargo's own report carried into the failure, so that a resolver with a great deal
/// to say cannot turn one stale lockfile into a panic nobody can read.
const MAX_REPORT_BYTES: usize = 4 * 1024;

/// The repository this crate sits in, which is the directory above it.
fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("a directory above this crate")
        .to_path_buf()
}

/// Every lockfile the repository commits, wherever it sits.
fn lockfiles(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![(root.to_path_buf(), 0usize)];

    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH || found.len() >= MAX_FILES {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            // `file_type` rather than `metadata`, so a symlink is what it is rather than what it
            // points at and the walk cannot be sent somewhere else.
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name();
            let name = name.to_string_lossy().into_owned();
            if kind.is_dir() {
                if !NOT_SOURCE.contains(&name.as_str()) {
                    pending.push((entry.path(), depth + 1));
                }
            } else if kind.is_file() && name == "Cargo.lock" && found.len() < MAX_FILES {
                found.push(entry.path());
            }
        }
    }

    found.sort();
    found
}

/// Cargo's report, cut on a character boundary to something a panic can carry.
fn readable(output: &str) -> String {
    if output.len() <= MAX_REPORT_BYTES {
        return output.trim_end().to_owned();
    }
    let mut end = MAX_REPORT_BYTES;
    while end > 0 && !output.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", output[..end].trim_end())
}

#[test]
fn every_committed_lockfile_still_resolves_against_the_manifests_beside_it() {
    let root = repository();
    let found = lockfiles(&root);

    // A walk that reached nothing would pass this having resolved nothing at all. The root's own
    // lockfile and at least one outside it are what makes the question worth asking.
    assert!(
        found.len() > 1,
        "expected a lockfile at the repository root and at least one outside it, found {found:?}"
    );

    let stale: Vec<String> = found
        .iter()
        .filter_map(|lock| {
            let manifest = lock.with_file_name("Cargo.toml");
            let manifest = manifest.to_string_lossy().into_owned();
            let ended = capture(
                "cargo",
                &[
                    "tree",
                    "--locked",
                    "--depth",
                    "0",
                    "--manifest-path",
                    &manifest,
                ],
                WITHIN,
            )
            .unwrap_or_else(|cause| panic!("cargo could not be run for {manifest}: {cause}"));

            (!ended.ok).then(|| {
                format!(
                    "{} ({}):\n{}",
                    lock.strip_prefix(&root).unwrap_or(lock).display(),
                    ended.status,
                    readable(&ended.output)
                )
            })
        })
        .collect();

    assert!(
        stale.is_empty(),
        "these lockfiles no longer resolve against the manifests beside them, which is what a \
         dependency bump made outside their workspace does to them; re-resolve each one where it \
         sits:\n\n{}",
        stale.join("\n\n")
    );
}
