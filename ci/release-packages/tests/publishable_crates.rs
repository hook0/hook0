//! Refuses a crate that carries the repository's own licence and could still be published.
//!
//! Two licences live in this tree and they mean opposite things. The clients are MIT and are meant
//! to reach a registry; that is the whole point of them. Everything else is the SSPL, which is the
//! licence of the server this repository is, and none of it is a package anybody installs.
//!
//! Four of those server crates said so, but said it to the wrong tool. `publish = false` under
//! `[package.metadata.release]` is cargo-release's key, and it stops cargo-release. Cargo itself
//! never reads it, so `cargo publish` in any of those directories went through. The two crates that
//! did not even carry that line, `hook0-protobuf` and `hook0-sentry-integration`, went through in
//! the same silence. Six crates of server code were one command away from crates.io, which is a
//! place nothing comes back from: a published version cannot be withdrawn, only yanked, and the
//! source stays readable either way.
//!
//! The distance between the two keys is one word in a table header, and reading a manifest does not
//! show it. Both spell `publish = false`, and the eye that has just read the right one reads the
//! wrong one as the same line. So this asks cargo's question rather than a reader's: of the crates
//! this repository tracks, which ones does `cargo publish` still accept?
//!
//! Nothing here names a crate or a directory. The manifests come from `git ls-files`, and the
//! licence that marks server code is checked against the licence file at the root, so a repository
//! that relicenses reports it here rather than passing on a constant nobody revisited.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

/// The SPDX identifier of the repository's own licence, which is the mark of code that is the
/// server rather than a package. Held honest by [`licence_is_still_the_repository_s_own`] below:
/// a repository that relicenses fails there rather than quietly leaving this guard matching
/// nothing.
const SERVER_LICENCE: &str = "SSPL-1.0";

/// The wording that licence opens with, which is what ties [`SERVER_LICENCE`] to the file at the
/// root rather than to a memory of what it used to say.
const SERVER_LICENCE_WORDING: &str = "Server Side Public License";

/// The most manifests examined. A repository past this is one this guard reports rather than one it
/// half-checks, since a truncated answer here reads as a clean bill of health.
const MAX_MANIFESTS: usize = 512;

/// The most bytes read out of one manifest.
const MAX_MANIFEST_BYTES: u64 = 1 << 20;

/// The most bytes read out of the licence file at the root.
const MAX_LICENCE_BYTES: u64 = 1 << 20;

/// The most bytes read back from one git invocation.
const MAX_GIT_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

/// How many offenders the failure names before it stops and counts the rest.
const MAX_REPORTED: usize = 20;

/// One crate that carries the server's licence, and what its manifest says about publishing it.
struct Crate {
    manifest: String,
    name: String,
    /// What `[package] publish` holds, written the way a reader of the manifest would see it, or
    /// nothing when the key is absent. Absent is the dangerous case and the easy one to overlook,
    /// so it is kept distinct from a key that is present and wrong.
    publish: Option<String>,
}

/// The repository this crate sits in, or nothing when it sits in no repository at all.
///
/// Nothing is the answer for a source archive rather than a checkout. There the invariant does not
/// apply rather than holding unverified, and the skip is printed, because a guard nobody notices
/// has stopped running is the failure this file exists to avoid.
fn repository() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spawned = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(manifest)
        .output();

    let finished = match spawned {
        Ok(finished) => finished,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => {
            println!("skipped: there is no git to find the repository with");
            return None;
        }
        Err(failure) => panic!("could not run `git rev-parse`: {failure}"),
    };

    if !finished.status.success() {
        println!("skipped: {} is not in a git repository", manifest.display());
        return None;
    }

    let root = String::from_utf8_lossy(&finished.stdout).trim().to_string();
    assert!(
        !root.is_empty(),
        "git named no repository root for {}",
        manifest.display()
    );
    Some(PathBuf::from(root))
}

/// Every manifest the repository tracks.
///
/// Tracked rather than found on disk: a manifest git does not know about is not one a release could
/// ever reach, and a build directory holds a great many of them.
fn manifests(root: &Path) -> Vec<String> {
    let finished = Command::new("git")
        .args(["ls-files", "-z", "--", "*Cargo.toml", "Cargo.toml"])
        .current_dir(root)
        .output()
        .unwrap_or_else(|failure| panic!("could not run `git ls-files`: {failure}"));

    assert!(
        finished.status.success(),
        "`git ls-files` failed in {}: {}",
        root.display(),
        String::from_utf8_lossy(&finished.stderr).trim()
    );
    assert!(
        finished.stdout.len() <= MAX_GIT_OUTPUT_BYTES,
        "`git ls-files` answered more than the {MAX_GIT_OUTPUT_BYTES} bytes this guard reads"
    );

    let mut paths = finished
        .stdout
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect::<Vec<_>>();

    assert!(
        paths.len() <= MAX_MANIFESTS,
        "{} tracks {} manifests, more than the {MAX_MANIFESTS} this guard examines",
        root.display(),
        paths.len()
    );
    paths.sort();
    paths
}

/// Reads one file, refusing one larger than the bound rather than pulling it all in.
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

/// The crates carrying the server's licence, and what each says about publishing.
fn server_crates(root: &Path, paths: &[String]) -> Vec<Crate> {
    let mut found = Vec::new();

    for relative in paths {
        let text = read_bounded(&root.join(relative), MAX_MANIFEST_BYTES);
        let parsed = text
            .parse::<toml::Table>()
            .unwrap_or_else(|failure| panic!("could not parse {relative}: {failure}"));

        // A virtual manifest has no `[package]` at all, and describes no crate to publish.
        let Some(package) = parsed.get("package").and_then(toml::Value::as_table) else {
            continue;
        };
        let licence = package.get("license").and_then(toml::Value::as_str);
        if licence != Some(SERVER_LICENCE) {
            continue;
        }

        found.push(Crate {
            manifest: relative.clone(),
            name: package
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unnamed>")
                .to_string(),
            publish: package.get("publish").map(ToString::to_string),
        });
    }

    found
}

#[test]
fn the_licence_that_marks_server_code_is_still_the_repository_s_own() {
    let Some(root) = repository() else {
        return;
    };

    // The constant above is the whole reach of this guard: were it to name a licence this
    // repository no longer carries, the guard would match nothing and pass for good.
    let licence = root.join("LICENSE.txt");
    assert!(
        licence.is_file(),
        "{} does not exist, so nothing ties `{SERVER_LICENCE}` to what this repository is licensed \
         under. Point this at the licence file, or drop this guard along with the one below it.",
        licence.display()
    );

    let text = read_bounded(&licence, MAX_LICENCE_BYTES);
    assert!(
        text.contains(SERVER_LICENCE_WORDING),
        "{} does not read as the {SERVER_LICENCE_WORDING}, so `{SERVER_LICENCE}` no longer names \
         the licence this repository carries and the guard below has stopped matching the crates \
         it was written for. Update both to whatever this repository is licensed under now.",
        licence.display()
    );
}

#[test]
fn no_crate_under_the_server_licence_can_be_published() {
    let Some(root) = repository() else {
        return;
    };

    let paths = manifests(&root);
    assert!(
        !paths.is_empty(),
        "{} tracks no manifest at all, so this guard is looking in the wrong place",
        root.display()
    );

    let crates = server_crates(&root, &paths);
    // Were the licence to be spelled some other way, every crate would fall out of the filter above
    // and this would pass having checked nothing. It is the same silence the guard is written
    // against, so it fails here instead.
    assert!(
        !crates.is_empty(),
        "no manifest in {} declares `license = \"{SERVER_LICENCE}\"`, so this guard just examined \
         nothing. Either the licence is spelled differently now, in which case fix the constant, \
         or the server crates have gone, in which case this guard has no subject left.",
        root.display()
    );

    // Cargo publishes unless the manifest says otherwise, and it accepts a list of registries as
    // well as a bare `true`. Only an outright `false` closes the door.
    let open = crates
        .iter()
        .filter(|one| one.publish.as_deref() != Some("false"))
        .collect::<Vec<_>>();
    if open.is_empty() {
        return;
    }

    let mut report = open
        .iter()
        .take(MAX_REPORTED)
        .map(|one| match &one.publish {
            Some(value) => format!("  {} ({})\n      publish = {value}", one.name, one.manifest),
            None => format!(
                "  {} ({})\n      no `publish` key at all",
                one.name, one.manifest
            ),
        })
        .collect::<Vec<_>>()
        .join("\n");
    if open.len() > MAX_REPORTED {
        report.push_str(&format!("\n  ... and {} more", open.len() - MAX_REPORTED));
    }

    let counted = match open.len() {
        1 => "One crate carries".to_string(),
        many => format!("{many} crates carry"),
    };

    panic!(
        "{counted} the {SERVER_LICENCE} and `cargo publish` would still accept them. This licence \
         marks the server itself, which is not something anybody installs from a registry, and a \
         published version can only be yanked, never withdrawn:\n\n{report}\n\nAdd `publish = \
         false` to the `[package]` table of each manifest above. The same line under \
         `[package.metadata.release]` is cargo-release's key and does not reach cargo, so a crate \
         can carry that one and still be published by hand."
    );
}
