//! Lists what the repository releases, and answers the questions a release asks of it.
//!
//! `list` prints the inventory, which is what a release pipeline does before it publishes anything
//! so that what went out is legible afterwards rather than inferred. `current`, `check-version` and
//! `set-version` are what the bump script and the tag pipeline ask instead of naming files, and
//! `directories` is what scopes a changelog to the package it belongs to.
//!
//! Everything but `list` is about the SDK release — the packages that go out together under one
//! tag. A package with a release flow of its own is listed and then left alone, since bumping it
//! here would set it to a version its own tag never claimed.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use release_packages::{
    Change, TargetRoot, check_version, current_version, discover, render, sdk_train, set_version,
};

/// Where the repository is, which is the working directory unless something says otherwise.
const ROOT_VARIABLE: &str = "RELEASE_PACKAGES_ROOT";

const USAGE: &str = "usage: release-packages [list | current | directories | \
                     check-version <X.Y.Z> | set-version <X.Y.Z>]";

fn main() -> ExitCode {
    let tree = std::env::var_os(ROOT_VARIABLE)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] | ["list"] => list(&tree),
        ["current"] => current(&tree),
        ["directories"] => directories(&tree),
        ["check-version", version] => check(&tree, version),
        ["set-version", version] => write(&tree, version),
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

fn packages(tree: &Path) -> Result<Vec<release_packages::Package>, String> {
    discover(&registry(), tree).map_err(|e| e.to_string())
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
