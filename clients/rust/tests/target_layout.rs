//! Keeps hand-written tests out of the directories the SDK generator owns.
//!
//! `clients/<target>/src` is generator output: it is rewritten wholesale every time the target is
//! regenerated. `clients/<target>/tests` is hand-written and never regenerated. A test file that
//! drifts back into `src` is therefore deleted without a word at the next regeneration, and the
//! coverage it carried disappears with it — which is exactly what this guard refuses.
//!
//! Targets are discovered by looking at what is actually in `clients/`, so a language added later
//! is covered without this file being touched.

use std::fs;
use std::path::{Path, PathBuf};

/// The words that mark a file or directory as holding tests, across naming conventions:
/// `foo.test.ts`, `foo_test.go`, `test_foo.py`, `FooTests.cs`, `foo_spec.rb`, `__tests__/`.
/// These describe the rule itself; no target and no language is named anywhere in this file.
const TEST_WORDS: [&str; 4] = ["test", "tests", "spec", "specs"];

/// No target nests its sources anywhere near this deep. The bound turns a pathological tree into a
/// failure instead of a walk that never ends.
const MAX_DEPTH: usize = 16;

/// Likewise for breadth: a `src` holding more entries than this is not a source tree.
const MAX_ENTRIES: usize = 20_000;

/// Splits a file or directory name into the words it is built from, across the separators the
/// ecosystem uses (`.`, `_`, `-`, and camel-case humps).
fn words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_was_lower = false;

    for character in name.chars() {
        if character == '.' || character == '_' || character == '-' || character == ' ' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            previous_was_lower = false;
            continue;
        }

        if character.is_uppercase() && previous_was_lower && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }

        previous_was_lower = character.is_lowercase() || character.is_numeric();
        current.extend(character.to_lowercase());
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn marks_tests(name: &str) -> bool {
    words(name)
        .iter()
        .any(|word| TEST_WORDS.contains(&word.as_str()))
}

/// Every file or directory under `root` whose name marks it as holding tests. A directory that is
/// itself reported is not descended into: naming what to move is enough, listing its contents on
/// top of it is noise.
fn test_paths_under(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut visited = 0usize;

    while let Some((directory, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Err(format!(
                "{} is nested more than {MAX_DEPTH} directories deep",
                directory.display()
            ));
        }

        let entries = fs::read_dir(&directory)
            .map_err(|e| format!("could not read {}: {e}", directory.display()))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| format!("could not read {}: {e}", directory.display()))?;

            visited += 1;
            if visited > MAX_ENTRIES {
                return Err(format!(
                    "{} holds more than {MAX_ENTRIES} entries",
                    root.display()
                ));
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            if marks_tests(&name) {
                found.push(entry.path());
                continue;
            }

            // `file_type` does not follow symlinks, so a link back up the tree is never descended.
            let file_type = entry
                .file_type()
                .map_err(|e| format!("could not read {}: {e}", entry.path().display()))?;
            if file_type.is_dir() {
                stack.push((entry.path(), depth + 1));
            }
        }
    }

    found.sort();
    Ok(found)
}

/// `clients/`, reached from this crate rather than from the working directory, which `cargo test`
/// makes no promise about.
fn clients_directory() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate manifest directory has a parent")
}

fn display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[test]
fn no_target_keeps_its_tests_under_src() {
    let clients = clients_directory();
    let repository = clients
        .parent()
        .expect("the clients directory has a parent");

    let mut targets = fs::read_dir(clients)
        .expect("the clients directory is readable")
        .map(|entry| entry.expect("a clients directory entry is readable").path())
        .filter(|path| path.join("src").is_dir())
        .collect::<Vec<_>>();
    targets.sort();

    assert!(
        !targets.is_empty(),
        "no target with a `src` directory found under {} — this guard is looking in the wrong place",
        clients.display()
    );

    let offenders = targets
        .iter()
        .flat_map(|target| {
            let source = target.join("src");
            let paths = test_paths_under(&source)
                .unwrap_or_else(|e| panic!("could not walk {}: {e}", source.display()));
            paths
                .into_iter()
                .map(|path| {
                    format!(
                        "  {}  ->  {}/",
                        display(&path, repository),
                        display(&target.join("tests"), repository)
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "These tests live under a target's `src`, which the SDK generator rewrites wholesale on \
         every regeneration — they would be deleted without a word, and the coverage they carry \
         with them:\n\n{}\n\nMove each of them to the `tests` directory of its target, which is \
         hand-written and never regenerated, and point that target's test runner at it.",
        offenders.join("\n")
    );
}
