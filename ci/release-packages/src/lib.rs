//! What the repository releases, derived from the repository rather than written down beside it.
//!
//! The failure this exists to remove is silent: a client that nobody remembered to add to the
//! release script ships forever at whatever version it was born with, and nothing goes red. So
//! there is no list of packages here. What there is, is a rule — a target lands somewhere under the
//! one directory its package occupies, and that package is whatever manifest sits at or above where
//! it lands — applied to the generator's own registry of targets. A target added there is released;
//! a target whose package cannot be established stops the release naming itself.
//!
//! The tree is read a second time in the other direction, so the mirror-image omission is caught
//! too: a package added under `clients/` that no target claims would be versioned by nothing, and
//! is refused rather than passed over.
//!
//! Being versioned is half of being released. The release flows are read the same way, off what
//! each publish job declares it publishes, and a package no job publishes is refused unless the
//! reason it has none is written where this can read it — so a registry left alone deliberately is
//! a decision rather than an absence indistinguishable from an oversight.
//!
//! Every walk is bounded — how many targets, how far up, how many entries in a directory, how many
//! bytes of a file — and a ceiling crossed is a refusal rather than a truncation.

mod error;
pub mod manifest;
mod xml;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub use error::Error;
pub use manifest::{Kind, Manifest, Registry, Version};

/// The directory every releasable package occupies one of.
pub const CLIENTS: &str = "clients";

/// The most targets walked in one run. Nine ship today; a repository past this is one where
/// somebody should be raising this deliberately.
pub const MAX_TARGETS: usize = 64;

/// The most directory levels climbed from where a target lands towards its package.
pub const MAX_ASCENT: usize = 8;

/// The most entries read out of one directory.
pub const MAX_DIR_ENTRIES: usize = 4096;

/// What the tag of a release covering every SDK at once starts with.
pub const SDK_TAG_PREFIX: &str = "sdk-v";

/// Where a package's own release flow would be, if it has one — `ci/release-mcp.gitlab-ci.yml` for
/// the package sitting in `clients/mcp`.
const FLOW: (&str, &str) = ("ci/release-", ".gitlab-ci.yml");

/// The one directory the release flows live in, and so the one a publish job is read out of.
const CI: &str = "ci";

/// What a publish job calls the package directory it publishes, and cds into to publish it.
///
/// Declaring it is what makes a publisher legible here, and using it is what stops the declaration
/// and the deed drifting apart: a job that published something else would have to go somewhere
/// other than where it says it goes.
pub const PUBLISHES: &str = "PUBLISHES";

/// Where a package no job publishes says why it has none.
pub const NO_PUBLISH_JOB: &str = "ci/release-no-publish-job.toml";

/// The most characters a directory, a registry or a published name stated in that file — or a
/// directory a job declares it publishes — may run to.
const MAX_NAMED_CHARS: usize = 256;

/// The most characters one recorded reason may run to. A reason is a paragraph about what a
/// registry is waiting for; anything past this is a document, and belongs where documents go.
const MAX_REASON_CHARS: usize = 4096;

/// Which release a package rides.
///
/// ADR 0004 gives a package that owns its release flow a tag of its own, and the SDKs that do not
/// own one go out together under a single tag. That is the whole distinction, and it is read off the
/// tree rather than written down: a package with a flow beside it is on its own; every other one is
/// on the SDK's. A bump that ignored this would set the MCP server to the SDK's version and quietly
/// break the release it actually has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Train {
    /// Versioned and published with the other SDKs, under one [`SDK_TAG_PREFIX`] tag.
    Sdk,
    /// Has a flow of its own, and a tag of its own with it.
    Own { flow: String, tag_prefix: String },
}

impl Train {
    /// What a release of this package is tagged with.
    pub fn tag_prefix(&self) -> &str {
        match self {
            Train::Sdk => SDK_TAG_PREFIX,
            Train::Own { tag_prefix, .. } => tag_prefix,
        }
    }
}

/// A target, reduced to the two things releasing it depends on.
///
/// Built from `hook0_sdkgen::targets::targets()` by the binary; built by hand in the suites, which
/// is what lets a target the tool has never seen be put in front of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRoot {
    /// How the generator names the target.
    pub name: String,
    /// Where it writes, relative to the repository root — its *generated* directory, which is
    /// somewhere inside the package rather than the package itself.
    pub root: String,
}

/// One thing the repository can publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// The target that generates into it.
    pub target: String,
    /// The one directory under [`CLIENTS`] the package occupies.
    pub directory: String,
    /// The manifest the name and version were read out of.
    pub manifest: String,
    pub kind: Kind,
    pub registry: Registry,
    /// What the package is published as, which is what a user installs.
    pub name: String,
    pub version: Version,
    pub train: Train,
}

impl Package {
    /// Whether this package goes out with the other SDKs rather than on a release of its own.
    pub fn on_sdk_train(&self) -> bool {
        self.train == Train::Sdk
    }
}

/// The packages one `sdk-vX.Y.Z` tag covers.
pub fn sdk_train(packages: &[Package]) -> Vec<Package> {
    packages
        .iter()
        .filter(|package| package.on_sdk_train())
        .cloned()
        .collect()
}

/// The one version every SDK is at, or a refusal naming the packages that disagree.
///
/// What a bump starts from has to be a fact rather than a file somebody picked, so it is read off
/// all of them at once: two SDKs at different versions is a release that already went wrong.
pub fn current_version(packages: &[Package]) -> Result<semver::Version, Error> {
    let mut agreed: Option<(&str, &semver::Version)> = None;
    for package in packages {
        let Version::Declared(site) = &package.version else {
            continue;
        };
        match agreed {
            None => agreed = Some((&package.name, &site.value)),
            Some((first, version)) if *version != site.value => {
                return Err(Error::TrainDisagrees {
                    first: first.to_string(),
                    first_version: version.to_string(),
                    second: package.name.clone(),
                    second_version: site.value.to_string(),
                });
            }
            Some(_) => {}
        }
    }
    agreed
        .map(|(_, version)| version.clone())
        .ok_or(Error::NoVersionAtAll)
}

/// Every package the targets resolve to, in the order the registry walks them.
pub fn discover(targets: &[TargetRoot], tree: &Path) -> Result<Vec<Package>, Error> {
    if targets.len() > MAX_TARGETS {
        return Err(Error::TooManyTargets {
            count: targets.len(),
            ceiling: MAX_TARGETS,
        });
    }

    let mut packages = Vec::with_capacity(targets.len());
    for target in targets {
        packages.push(resolve(target, tree)?);
    }

    refuse_duplicates(&packages)?;
    refuse_unclaimed(&packages, tree)?;
    Ok(packages)
}

/// The package one target belongs to.
fn resolve(target: &TargetRoot, tree: &Path) -> Result<Package, Error> {
    let directory = package_directory(target)?;
    let mut looked_at = Vec::new();

    for candidate in ascent(&target.root, &directory) {
        looked_at.push(candidate.clone());
        let found = manifests_in(&tree.join(&candidate))?;
        let Some((file, kind)) = found.first() else {
            continue;
        };
        if found.len() > 1 {
            return Err(Error::AmbiguousManifest {
                target: target.name.clone(),
                directory: candidate,
                count: found.len(),
                names: found
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
        let path = PathBuf::from(&candidate).join(file);
        let read = manifest::read(tree, &path, *kind)?;
        return Ok(Package {
            target: target.name.clone(),
            train: train_of(&directory, tree),
            directory,
            manifest: slashed(&path),
            kind: read.kind,
            registry: read.kind.registry(),
            name: read.name,
            version: read.version,
        });
    }

    Err(Error::NoManifest {
        target: target.name.clone(),
        root: target.root.clone(),
        package_dir: directory,
        looked_at: looked_at.join(", "),
    })
}

/// The directory a target's package occupies, read off where the target lands.
///
/// Every package sits directly under [`CLIENTS`], so the first two segments of a target's root name
/// it. Anything else is a target this tool cannot place, which is a refusal rather than a guess.
fn package_directory(target: &TargetRoot) -> Result<String, Error> {
    let mut segments = target.root.split('/');
    match (segments.next(), segments.next()) {
        (Some(CLIENTS), Some(package)) if !package.is_empty() => Ok(format!("{CLIENTS}/{package}")),
        _ => Err(Error::RootOutsideClients {
            target: target.name.clone(),
            root: target.root.clone(),
            clients: CLIENTS.to_string(),
        }),
    }
}

/// Which release a package rides, read off whether a flow of its own sits beside the others.
fn train_of(directory: &str, tree: &Path) -> Train {
    let name = directory.rsplit('/').next().unwrap_or(directory);
    let flow = format!("{}{name}{}", FLOW.0, FLOW.1);
    match tree.join(&flow).is_file() {
        true => Train::Own {
            flow,
            tag_prefix: format!("{name}/v"),
        },
        false => Train::Sdk,
    }
}

/// Every directory from where a target lands up to and including its package, nearest first.
fn ascent(root: &str, package_directory: &str) -> Vec<String> {
    let mut climbed = Vec::new();
    let mut at = Path::new(root);
    loop {
        climbed.push(slashed(at));
        if climbed.len() >= MAX_ASCENT || at == Path::new(package_directory) {
            break;
        }
        match at.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => at = parent,
            _ => break,
        }
    }
    climbed
}

/// What a directory holds, up to the ceiling and refusing past it.
///
/// A directory this tool cannot read at all is empty rather than an error: a target whose generated
/// directory has not been written yet is walked past on the way up to its package, and running out
/// of levels is what refuses. A directory it can read but not all of is the other thing entirely —
/// whatever sat past the last entry read would be missing from the inventory without a word, which
/// is the omission this tool is against, so the ceiling refuses naming the directory.
fn entries_of(directory: &Path) -> Result<Vec<std::fs::DirEntry>, Error> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Ok(Vec::new());
    };

    let mut found = Vec::new();
    for entry in entries {
        if found.len() >= MAX_DIR_ENTRIES {
            return Err(Error::TooManyEntries {
                path: directory.to_path_buf(),
                ceiling: MAX_DIR_ENTRIES,
            });
        }
        found.push(entry.map_err(|source| Error::Read {
            path: directory.to_path_buf(),
            source,
        })?);
    }
    Ok(found)
}

/// The manifests sitting directly in a directory, sorted, so that two of them are two rather than
/// whichever the filesystem answered first.
fn manifests_in(directory: &Path) -> Result<Vec<(String, Kind)>, Error> {
    let mut found = Vec::new();
    for entry in entries_of(directory)? {
        if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(kind) = Kind::of(&name) {
            found.push((name, kind));
        }
    }
    found.sort();
    Ok(found)
}

/// Two packages published as the same thing would overwrite each other on the first release that
/// noticed, so a collision stops the run rather than the registry.
fn refuse_duplicates(packages: &[Package]) -> Result<(), Error> {
    let mut claimed: BTreeMap<(&'static str, &str), &str> = BTreeMap::new();
    for package in packages {
        let key = (package.registry.id(), package.name.as_str());
        if let Some(first) = claimed.insert(key, &package.directory) {
            return Err(Error::DuplicatePackage {
                registry: package.registry.id(),
                name: package.name.clone(),
                first: first.to_string(),
                second: package.directory.clone(),
            });
        }
    }
    Ok(())
}

/// The omission in the other direction: a package sitting under [`CLIENTS`] that no target claims.
///
/// Nothing would ever version or publish it, and nothing would go red — which is the failure this
/// whole tool is against, so it is one here too. A manifest saying it is not for a registry is not
/// a package and is passed over.
fn refuse_unclaimed(packages: &[Package], tree: &Path) -> Result<(), Error> {
    let claimed: Vec<&str> = packages.iter().map(|p| p.directory.as_str()).collect();
    let clients = tree.join(CLIENTS);

    let mut directories = Vec::new();
    for entry in entries_of(&clients)? {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            directories.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    directories.sort();

    for name in directories {
        let directory = format!("{CLIENTS}/{name}");
        if claimed.contains(&directory.as_str()) {
            continue;
        }
        for (file, kind) in manifests_in(&clients.join(&name))? {
            let path = PathBuf::from(&directory).join(&file);
            if manifest::read(tree, &path, kind)?.publishable {
                return Err(Error::UnclaimedPackage { directory, kind });
            }
        }
    }
    Ok(())
}

/// The omission after the one this tool started with: a package that is versioned by everything
/// here and published by nothing.
///
/// Being in the inventory is what gets a package a version; a job that publishes it is what gets it
/// to a user. So the flows are read too, and the two are matched in both directions — a package no
/// job publishes is refused, and a job naming a package the inventory has never heard of is refused
/// with it, since one of the two is wrong either way and neither is knowable from the other alone.
///
/// A registry left alone on purpose is the third case, and it is why this is a match rather than a
/// rule: some ecosystems have nothing to upload to at all, and some are waiting on something only a
/// person can supply. Those are decisions, and a decision is written in [`NO_PUBLISH_JOB`] where
/// this reads it — which is what leaves every other absence an oversight.
pub fn check_publishers(packages: &[Package], tree: &Path) -> Result<(), Error> {
    let published = publish_declarations(tree)?;
    let recorded = recorded_absences(tree)?;

    for (file, directory) in &published {
        if !packages
            .iter()
            .any(|package| package.directory == *directory)
        {
            return Err(Error::UnknownPackageNamed {
                file: file.clone(),
                directory: directory.clone(),
            });
        }
    }

    for absence in &recorded {
        let Some(package) = packages
            .iter()
            .find(|package| package.directory == absence.directory)
        else {
            return Err(Error::UnknownPackageNamed {
                file: NO_PUBLISH_JOB.to_string(),
                directory: absence.directory.clone(),
            });
        };
        for (field, recorded, resolved) in [
            ("published to", &absence.registry, package.registry.id()),
            ("published as", &absence.name, package.name.as_str()),
        ] {
            if recorded != resolved {
                return Err(Error::RecordedDisagrees {
                    record: NO_PUBLISH_JOB,
                    directory: absence.directory.clone(),
                    field,
                    recorded: recorded.clone(),
                    resolved: resolved.to_string(),
                });
            }
        }
        if let Some((file, _)) = published
            .iter()
            .find(|(_, directory)| *directory == absence.directory)
        {
            return Err(Error::RecordedYetPublished {
                record: NO_PUBLISH_JOB,
                directory: absence.directory.clone(),
                file: file.clone(),
            });
        }
    }

    for package in packages {
        let published_by_a_job = published
            .iter()
            .any(|(_, directory)| *directory == package.directory);
        let said_so = recorded
            .iter()
            .any(|absence| absence.directory == package.directory);
        if !published_by_a_job && !said_so {
            return Err(Error::NoPublisher {
                name: package.name.clone(),
                directory: package.directory.clone(),
                registry: package.registry.id(),
                record: NO_PUBLISH_JOB,
            });
        }
    }
    Ok(())
}

/// A package recorded as having no publish job, named there the three ways the inventory names it
/// so that a record cannot go on describing a package this one has stopped being.
///
/// The reason recorded beside it is read and bounded and then let go: what it says is for whoever
/// reads the file, and that it says anything at all is what this tool is checking.
struct Absence {
    directory: String,
    registry: String,
    name: String,
}

/// What every release flow says it publishes, each claim carrying the file that made it — so a
/// claim about nothing can be answered with where it was written.
fn publish_declarations(tree: &Path) -> Result<Vec<(String, String)>, Error> {
    let mut declared = Vec::new();
    for entry in entries_of(&tree.join(CI))? {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(FLOW.1) || !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            continue;
        }
        let file = format!("{CI}/{name}");
        let body = manifest::read_bounded(&tree.join(&file))?;
        for line in body.lines() {
            let Some(claimed) = manifest::after_key(line, PUBLISHES) else {
                continue;
            };
            let claimed = claimed.trim().trim_matches(['"', '\'']).to_string();
            let claimed = bounded(Path::new(&file), PUBLISHES, claimed, MAX_NAMED_CHARS)?;
            declared.push((file.clone(), claimed));
        }
    }
    declared.sort();
    Ok(declared)
}

/// Every package recorded as published by nothing, refusing a record that says which package
/// without saying why.
fn recorded_absences(tree: &Path) -> Result<Vec<Absence>, Error> {
    let path = PathBuf::from(NO_PUBLISH_JOB);
    if !tree.join(&path).is_file() {
        return Ok(Vec::new());
    }

    let body = manifest::read_bounded(&tree.join(&path))?;
    let document: toml::Value = toml::from_str(&body).map_err(|e| Error::Unreadable {
        path: path.clone(),
        reason: e.to_string(),
    })?;
    let entries = document.get("package").and_then(toml::Value::as_array);

    let mut absences = Vec::new();
    for entry in entries.map(Vec::as_slice).unwrap_or_default() {
        let stated = |field: &'static str| -> Result<String, Error> {
            entry
                .get(field)
                .and_then(toml::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or(Error::MissingField {
                    path: path.clone(),
                    field,
                })
        };
        let directory = bounded(&path, "directory", stated("directory")?, MAX_NAMED_CHARS)?;
        let registry = bounded(&path, "registry", stated("registry")?, MAX_NAMED_CHARS)?;
        let name = bounded(
            &path,
            "published_as",
            stated("published_as")?,
            MAX_NAMED_CHARS,
        )?;
        bounded(&path, "reason", stated("reason")?, MAX_REASON_CHARS)?;
        absences.push(Absence {
            directory,
            registry,
            name,
        });
    }
    Ok(absences)
}

/// A value read out of a file this tool does not write, held to a ceiling before it is carried.
fn bounded(
    path: &Path,
    field: &'static str,
    value: String,
    ceiling: usize,
) -> Result<String, Error> {
    let length = value.chars().count();
    match length > ceiling {
        true => Err(Error::FieldTooLong {
            path: path.to_path_buf(),
            field,
            length,
            ceiling,
        }),
        false => Ok(value),
    }
}

/// One column of the inventory: what it is headed, and what it reads off a package.
type Column = (&'static str, fn(&Package) -> String);

/// What a release is about to touch, written out so it is legible rather than assumed.
pub fn render(packages: &[Package]) -> String {
    let columns: [Column; 7] = [
        ("target", |p| p.target.clone()),
        ("package directory", |p| p.directory.clone()),
        ("manifest", |p| {
            p.manifest
                .strip_prefix(&format!("{}/", p.directory))
                .unwrap_or(&p.manifest)
                .to_string()
        }),
        ("registry", |p| p.registry.id().to_string()),
        ("published as", |p| p.name.clone()),
        ("version", |p| p.version.to_string()),
        ("released by", |p| format!("{}*", p.train.tag_prefix())),
    ];

    let rows: Vec<Vec<String>> = packages
        .iter()
        .map(|package| columns.iter().map(|(_, cell)| cell(package)).collect())
        .collect();
    let widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(column, (header, _))| {
            rows.iter()
                .map(|row| row[column].chars().count())
                .chain([header.chars().count()])
                .max()
                .unwrap_or(0)
        })
        .collect();

    let mut out = String::new();
    let line = |out: &mut String, cells: &[String]| {
        let padded: Vec<String> = cells
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:width$}"))
            .collect();
        let _ = writeln!(out, "  {}", padded.join("  ").trim_end());
    };

    let _ = writeln!(out, "{} packages", packages.len());
    line(
        &mut out,
        &columns
            .iter()
            .map(|(header, _)| (*header).to_string())
            .collect::<Vec<_>>(),
    );
    line(
        &mut out,
        &widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>(),
    );
    for row in &rows {
        line(&mut out, row);
    }
    out
}

/// Refuse unless every package that declares a version declares this one.
///
/// What this catches is a tag that says one thing and a tree that says another — a package left
/// behind by a bump, or a tag pushed by hand. A package the host versions by the tag it is reached
/// under has nothing to compare and is passed over rather than made up.
pub fn check_version(packages: &[Package], version: &semver::Version) -> Result<(), Error> {
    for package in packages {
        if let Version::Declared(site) = &package.version
            && site.value != *version
        {
            return Err(Error::VersionMismatch {
                package: package.name.clone(),
                declared: site.value.to_string(),
                expected: version.to_string(),
            });
        }
    }
    Ok(())
}

/// What setting a version did to one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The file, and what it said before.
    Written { file: String, from: String },
    /// Nothing was written, because this package carries no version to write: the host answers it
    /// at the version of the tag it is reached under.
    VersionedByTag,
    /// The file already said so.
    Unchanged { file: String },
}

/// Write `version` into every package that declares one.
///
/// Each write is read back and parsed again before the next one starts, so a replacement that did
/// not take is a refusal naming the file rather than a release that publishes the old number.
pub fn set_version(
    tree: &Path,
    packages: &[Package],
    version: &semver::Version,
) -> Result<Vec<(String, Change)>, Error> {
    let mut changes = Vec::with_capacity(packages.len());
    for package in packages {
        let Version::Declared(site) = &package.version else {
            changes.push((package.name.clone(), Change::VersionedByTag));
            continue;
        };
        let file = slashed(&site.file);
        if site.value == *version {
            changes.push((package.name.clone(), Change::Unchanged { file }));
            continue;
        }

        let path = tree.join(&site.file);
        let body = manifest::read_bounded(&path)?;
        let mut written = String::with_capacity(body.len());
        written.push_str(&body[..site.span.start]);
        written.push_str(&version.to_string());
        written.push_str(&body[site.span.end..]);
        std::fs::write(&path, &written).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;

        let reread = manifest::read(tree, Path::new(&package.manifest), package.kind)?;
        match &reread.version {
            Version::Declared(site) if site.value == *version => {}
            other => {
                return Err(Error::WriteNotTaken {
                    path: path.clone(),
                    requested: version.to_string(),
                    found: other.to_string(),
                });
            }
        }

        changes.push((
            package.name.clone(),
            Change::Written {
                file,
                from: site.value.to_string(),
            },
        ));
    }
    Ok(changes)
}

/// A path written the one way this tool prints and compares them, so that what it reports is what a
/// `.gitlab-ci.yml` and a `git-cliff` invocation are handed.
fn slashed(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}
