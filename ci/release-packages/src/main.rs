//! Lists what the repository releases, and answers the questions a release asks of it.
//!
//! `list` prints the inventory, which is what a release pipeline does before it publishes anything
//! so that what went out is legible afterwards rather than inferred. `current`, `check-version` and
//! `set-version` are what the bump script and the tag pipeline ask instead of naming files,
//! `required-bump` is what they ask before writing anything so that the release is at least as big
//! as its own commits say it is, `directories` is what scopes a changelog to the package it belongs
//! to, and `mirrors` is what the mirror push iterates so that a client added to the generator's
//! registry is pushed to a repository of its own without a line of YAML being written.
//!
//! Everything but `list` is about the SDK release — the packages that go out together under one
//! tag. A package with a release flow of its own is listed and then left alone, since bumping it
//! here would set it to a version its own tag never claimed.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use release_packages::{
    Bump, Change, TargetRoot, check_bump, check_mirrors, check_publishers, check_version,
    current_version, discover, mirrors, naming, render, required_bump, sdk_train, set_version,
};

/// Where the repository is, which is the working directory unless something says otherwise.
const ROOT_VARIABLE: &str = "RELEASE_PACKAGES_ROOT";

const USAGE: &str = "usage: release-packages [list | current | directories | mirrors | \
                     check-version <X.Y.Z> | set-version <X.Y.Z> | \
                     required-bump <patch|minor|major> <tag-glob> <path>...]";

fn main() -> ExitCode {
    let tree = std::env::var_os(ROOT_VARIABLE)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] | ["list"] => list(&tree),
        ["current"] => current(&tree),
        ["directories"] => directories(&tree),
        ["mirrors"] => mirror_list(&tree),
        ["check-version", version] => check(&tree, version),
        ["set-version", version] => write(&tree, version),
        ["required-bump", requested, tag_glob, ref paths @ ..] if !paths.is_empty() => {
            demanded(&tree, requested, tag_glob, paths)
        }
        _ => Err(format!(
            "{USAGE}\n  {ROOT_VARIABLE} names the repository root; it defaults to the working \
             directory."
        )),
    };

    match outcome {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("release-packages: {reason}");
            ExitCode::FAILURE
        }
    }
}

/// Everything the repository can publish, however it is released.
fn list(tree: &Path) -> Result<String, String> {
    Ok(render(&packages(tree)?))
}

/// The one version the SDK release is at, for a bump to start from.
fn current(tree: &Path) -> Result<String, String> {
    let version = current_version(&sdk_train(&packages(tree)?)).map_err(|e| e.to_string())?;
    Ok(format!("{version}\n"))
}

/// The directory of every package one SDK tag covers, which is what scopes its changelog.
fn directories(tree: &Path) -> Result<String, String> {
    Ok(sdk_train(&packages(tree)?)
        .iter()
        .map(|package| format!("{}\n", package.directory))
        .collect())
}

/// Every mirror the release pushes, one per line: the directory to split, then the repository to
/// push it to. Two fields and no header, because a shell loop is what reads this.
fn mirror_list(tree: &Path) -> Result<String, String> {
    Ok(mirrors(&packages(tree)?)
        .map_err(|e| e.to_string())?
        .iter()
        .map(|mirror| format!("{} {}\n", mirror.directory, mirror.repository))
        .collect())
}

/// Refuse unless the tree says what the tag says.
fn check(tree: &Path, version: &str) -> Result<String, String> {
    let version = parse(version)?;
    let found = packages(tree)?;
    let mut report = render(&found);
    check_version(&sdk_train(&found), &version).map_err(|e| e.to_string())?;
    report.push_str(&format!(
        "\nevery SDK reads {version}, which is what the tag says\n"
    ));
    Ok(report)
}

/// Write one version across every SDK that declares one.
fn write(tree: &Path, version: &str) -> Result<String, String> {
    let version = parse(version)?;
    let found = packages(tree)?;
    let train = sdk_train(&found);

    let mut report = render(&found);
    report.push_str(&format!("\nsetting every SDK to {version}\n"));
    for (name, change) in set_version(tree, &train, &version).map_err(|e| e.to_string())? {
        report.push_str(&match change {
            Change::Written { file, from } => format!("  {name}: {from} -> {version} ({file})\n"),
            Change::Unchanged { file } => format!("  {name}: already {version} ({file})\n"),
            Change::VersionedByTag => {
                format!("  {name}: nothing to write, versioned by the tag it is reached under\n")
            }
        });
    }
    Ok(report)
}

/// Refuse a release smaller than the commits since the last one demand, and say what they were.
///
/// This reads the history rather than the tree, so it is the one command here that asks nothing of
/// the inventory: the packages a tag covers are what the caller passes as paths, and a package with
/// a release flow of its own passes its own.
fn demanded(
    tree: &Path,
    requested: &str,
    tag_glob: &str,
    paths: &[&str],
) -> Result<String, String> {
    let asked = Bump::named(requested)
        .ok_or_else(|| format!("`{requested}` is not a bump; use patch, minor or major"))?;
    let found = required_bump(tree, tag_glob, paths).map_err(|e| e.to_string())?;
    check_bump(&found, asked).map_err(|e| e.to_string())?;

    Ok(format!(
        "{} commits since {} touch {}\nthe smallest release they allow is {}, and {} was asked \
         for\n{}",
        found.read,
        found.since,
        paths.join(", "),
        found.bump,
        asked,
        naming(&found.reasons, Bump::Patch),
    ))
}

/// The inventory, and the three things that have to be true of it before any command reads it:
/// every target resolves to a package, every package reaches a registry or says why it does not,
/// and every package fetched by URL is named after the mirror serving it.
fn packages(tree: &Path) -> Result<Vec<release_packages::Package>, String> {
    let found = discover(&registry(), tree).map_err(|e| e.to_string())?;
    check_publishers(&found, tree).map_err(|e| e.to_string())?;
    check_mirrors(&found).map_err(|e| e.to_string())?;
    Ok(found)
}

fn parse(version: &str) -> Result<semver::Version, String> {
    semver::Version::parse(version).map_err(|e| format!("`{version}` is not a version: {e}"))
}

/// What the generator says it writes, which is the only thing here that knows a target exists.
fn registry() -> Vec<TargetRoot> {
    hook0_sdkgen::targets::targets()
        .iter()
        .map(|target| TargetRoot {
            name: target.name.to_string(),
            root: target.root.to_string(),
        })
        .collect()
}
