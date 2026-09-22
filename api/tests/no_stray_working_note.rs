//! Repo-hygiene guard: the repository root must carry no stray working note.
//!
//! Design: a working note — an agent's `plan-<date>.md`, a scratch file — belongs
//! in the task tracker or a local volume, never committed, and least of all at the
//! root of an open-source tree. This test reads the working tree only (no server,
//! no network, no database) so it runs anywhere `cargo test` does and turns the
//! invariant red the moment such a file is tracked.
//!
//! The match is deliberately narrow — `plan*.md` at the root, case-insensitive —
//! because that is the one name that is *always* an error there. Editorial SEO
//! material lives under `seo/<YYYY-MM-DD>-...` (and `seo/` is gitignored on this
//! public repo), never a loose file at the root.

use std::path::{Path, PathBuf};

/// Repository root, resolved from this crate's manifest dir (`api/`) one level up.
/// `CARGO_MANIFEST_DIR` is set by cargo at compile time, so the path does not
/// depend on the current working directory the test happens to run from.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the api crate always sits one level below the repository root")
        .to_path_buf()
}

/// Names of files directly at the repo root that look like a working note
/// (`plan*.md`, any case). Sorted, so a failure reads deterministically.
fn stray_working_notes_at_root() -> Vec<String> {
    let mut stray: Vec<String> = std::fs::read_dir(repo_root())
        .expect("the repository root is readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            let lower = name.to_ascii_lowercase();
            lower.starts_with("plan") && lower.ends_with(".md")
        })
        .collect();
    stray.sort();
    stray
}

#[test]
fn repository_root_carries_no_stray_working_note() {
    let stray = stray_working_notes_at_root();
    assert!(
        stray.is_empty(),
        "forbidden working note(s) at the repository root: {}. \
         A plan/scratch note belongs in the task tracker or a local volume, never committed here.",
        stray.join(", ")
    );
}
