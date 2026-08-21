//! Every version an example project pins is one the repository already resolves.
//!
//! The Rust example project reaches the client by path, so that the pages agree with the client at
//! this commit rather than with whatever was last published. Everything else it depends on is
//! pinned exactly, and the manifest says why. Those pins are the versions the root `Cargo.lock`
//! already holds, so proving the documentation offline fetches nothing the checkout did not need
//! anyway.
//!
//! That is a claim about two files at once, and nothing was comparing them. The project was written
//! pinning `uuid` at `=1.24.0` while the lock had moved to `1.24.1`, which the client requires. The
//! two are one patch apart and read as the same version at a glance, so the pin looked like what it
//! claimed to be. Cargo disagreed and could not resolve the project at all, and the failure landed
//! in the documentation gate rather than anywhere near the dependency bump that caused it.
//!
//! A drift that does not conflict is worse, because nothing fails. `futures` was pinned a patch
//! behind the lock in the same way and resolved fine, quietly fetching a second copy of a crate the
//! repository had already built, which is the thing the pin exists to avoid.
//!
//! Nothing here names a project or a crate. The example projects are found under the SDK reference,
//! and what counts as resolved is read out of the lock, so a bump reports here rather than waiting
//! for the one pin that happens to conflict.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{fs, io};

use hook0_documentation_examples::SDK_REFERENCE;

/// The most files walked under the SDK reference.
const MAX_FILES: usize = 4096;

/// How far below the SDK reference the walk goes.
const MAX_DEPTH: usize = 8;

/// The most bytes read out of one manifest.
const MAX_MANIFEST_BYTES: u64 = 1 << 20;

/// The most bytes read out of the lock, which is the largest file this guard opens.
const MAX_LOCK_BYTES: u64 = 32 << 20;

/// What a build leaves behind. A project assembled into one of these carries pins nobody wrote.
const DERIVED: [&str; 6] = [".git", "node_modules", "obj", "target", "vendor", "zig-out"];

/// One pin that names a version the repository does not resolve.
struct Drift {
    manifest: String,
    crate_name: String,
    pinned: String,
    resolved: Vec<String>,
}

/// The repository this crate sits in, which is the directory above it.
fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("this crate is a directory inside the repository")
        .to_path_buf()
}

/// Reads one file, refusing one past the bound rather than pulling it all in.
fn read_bounded(path: &Path, ceiling: u64) -> String {
    let size = fs::metadata(path)
        .unwrap_or_else(|failure| panic!("could not stat {}: {failure}", path.display()))
        .len();
    assert!(
        size <= ceiling,
        "{} is {size} bytes, past the {ceiling} this guard reads",
        path.display()
    );
    fs::read_to_string(path)
        .unwrap_or_else(|failure| panic!("could not read {}: {failure}", path.display()))
}

/// Every cargo manifest under the SDK reference, which is where an example project lives.
///
/// Walked rather than listed, so a project added for a language nobody has written yet is checked
/// the day it arrives.
fn example_manifests(sdk: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut queue = vec![(sdk.to_path_buf(), 0usize)];
    let mut seen = 0usize;

    while let Some((directory, depth)) = queue.pop() {
        if depth > MAX_DEPTH {
            continue;
        }
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(failure) if failure.kind() == io::ErrorKind::NotFound => continue,
            Err(failure) => panic!("could not read {}: {failure}", directory.display()),
        };

        for entry in entries {
            let entry = entry.unwrap_or_else(|failure| {
                panic!("could not walk {}: {failure}", directory.display())
            });
            seen += 1;
            assert!(
                seen <= MAX_FILES,
                "more than {MAX_FILES} files under {} — this guard is walking somewhere it should not",
                sdk.display()
            );

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            // `file_type` rather than `metadata`, so a symlink is what it is rather than what it
            // points at, and the walk cannot be sent outside the tree.
            let kind = entry
                .file_type()
                .unwrap_or_else(|failure| panic!("could not stat {}: {failure}", path.display()));

            if kind.is_dir() {
                if !DERIVED.contains(&name.as_str()) {
                    queue.push((path, depth + 1));
                }
            } else if kind.is_file() && name == "Cargo.toml" {
                found.push(path);
            }
        }
    }

    found.sort();
    found
}

/// Every version the repository resolves, by crate. A crate can appear more than once when two
/// incompatible majors are both in the tree, and either is a version this repository already has.
fn resolved(root: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let lock = root.join("Cargo.lock");
    assert!(
        lock.is_file(),
        "{} does not exist, so there is nothing to check the pins against",
        lock.display()
    );

    let parsed = read_bounded(&lock, MAX_LOCK_BYTES)
        .parse::<toml::Table>()
        .unwrap_or_else(|failure| panic!("could not parse {}: {failure}", lock.display()));

    let packages = parsed
        .get("package")
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| panic!("{} holds no packages", lock.display()));

    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in packages {
        let (Some(name), Some(version)) = (
            package.get("name").and_then(toml::Value::as_str),
            package.get("version").and_then(toml::Value::as_str),
        ) else {
            continue;
        };
        versions
            .entry(name.to_string())
            .or_default()
            .insert(version.to_string());
    }

    assert!(
        !versions.is_empty(),
        "{} named no package at all",
        lock.display()
    );
    versions
}

#[test]
fn no_example_project_pins_a_version_the_repository_does_not_resolve() {
    let root = repository();
    let sdk = root.join(SDK_REFERENCE);
    assert!(
        sdk.is_dir(),
        "{} does not exist, so this guard is looking in the wrong place",
        sdk.display()
    );

    let manifests = example_manifests(&sdk);
    assert!(
        !manifests.is_empty(),
        "no cargo manifest under {}, so this guard just examined nothing",
        sdk.display()
    );

    let versions = resolved(&root);
    let mut drifted = Vec::new();
    let mut checked = 0usize;

    for manifest in &manifests {
        let parsed = read_bounded(manifest, MAX_MANIFEST_BYTES)
            .parse::<toml::Table>()
            .unwrap_or_else(|failure| panic!("could not parse {}: {failure}", manifest.display()));

        let Some(dependencies) = parsed.get("dependencies").and_then(toml::Value::as_table) else {
            continue;
        };
        let relative = manifest
            .strip_prefix(&root)
            .unwrap_or(manifest)
            .to_string_lossy()
            .into_owned();

        for (crate_name, spec) in dependencies {
            // A dependency states its version either as a bare string or under `version`, and one
            // reached by path states none at all.
            let requirement = match spec {
                toml::Value::String(version) => Some(version.as_str()),
                toml::Value::Table(table) => table.get("version").and_then(toml::Value::as_str),
                _ => None,
            };
            // Only an exact pin makes a claim about what the repository resolves. A range is a
            // range, and cargo is free to answer it with whatever it likes.
            let Some(pinned) = requirement.and_then(|version| version.strip_prefix('=')) else {
                continue;
            };

            checked += 1;
            let known = versions.get(crate_name.as_str());
            if !known.is_some_and(|found| found.contains(pinned)) {
                drifted.push(Drift {
                    manifest: relative.clone(),
                    crate_name: crate_name.clone(),
                    pinned: pinned.to_string(),
                    resolved: known
                        .map(|found| found.iter().cloned().collect())
                        .unwrap_or_default(),
                });
            }
        }
    }

    // A walk that found manifests but no pin at all has stopped checking anything, which is the
    // silence this guard was written against rather than a pass.
    assert!(
        checked > 0,
        "{} manifests under {} and not one exact pin among them, so this guard examined nothing",
        manifests.len(),
        sdk.display()
    );

    if drifted.is_empty() {
        return;
    }

    let report = drifted
        .iter()
        .map(|one| {
            let resolved = match one.resolved.as_slice() {
                [] => "the repository does not depend on it at all".to_string(),
                found => format!("the repository resolves {}", found.join(", ")),
            };
            format!(
                "  {} pins {} at {}\n      {resolved}",
                one.manifest, one.crate_name, one.pinned
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let counted = match drifted.len() {
        1 => "One pin names".to_string(),
        many => format!("{many} pins name"),
    };

    panic!(
        "{counted} a version this repository does not resolve:\n\n{report}\n\nAn example project \
         pins what the root `Cargo.lock` already holds, so that proving the documentation fetches \
         nothing new and the pages are built against the versions the client itself is. Set each \
         pin above to the version beside it. A pin left behind either conflicts, and the \
         documentation gate fails for a reason that has nothing to do with documentation, or it \
         resolves and quietly builds a second copy of a crate the repository already has."
    );
}
