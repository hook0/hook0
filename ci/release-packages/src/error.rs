//! Every way discovery refuses, each one naming what it was looking at when it did.
//!
//! There is no variant standing for "skipped": a target whose package cannot be established is the
//! failure mode this tool exists to remove, so it is an error rather than a shorter report.

use std::path::PathBuf;

use crate::bump::{Bump, Since};
use crate::manifest::Kind;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path} is {size} bytes, past the {ceiling}-byte ceiling a file is read under")]
    FileTooLarge {
        path: PathBuf,
        size: u64,
        ceiling: u64,
    },

    #[error(
        "the registry holds {count} targets, past the ceiling of {ceiling} this tool walks; raise \
         the ceiling deliberately rather than by accident"
    )]
    TooManyTargets { count: usize, ceiling: usize },

    #[error(
        "`{path}` holds more than {ceiling} entries, past the ceiling this tool reads a directory \
         under; raise the ceiling deliberately rather than by accident"
    )]
    TooManyEntries { path: PathBuf, ceiling: usize },

    #[error(
        "the release would push {count} mirrors, past the ceiling of {ceiling} one run pushes; \
         raise the ceiling deliberately rather than by accident"
    )]
    TooManyMirrors { count: usize, ceiling: usize },

    #[error(
        "`{directory}` is fetched from {registry}, where a package is named by the address it is \
         reached at; it declares `{declared}` and its mirror answers at `{expected}`, so what a \
         user would install is not what this release publishes"
    )]
    ModulePathNotTheMirror {
        directory: String,
        registry: &'static str,
        declared: String,
        expected: String,
    },

    #[error(
        "target `{target}` lands at `{root}`, which does not sit under `{clients}/<package>/`; \
         every package occupies one directory directly under `{clients}`, and the package a target \
         belongs to is read off that shape"
    )]
    RootOutsideClients {
        target: String,
        root: String,
        clients: String,
    },

    #[error(
        "target `{target}`: no manifest this tool recognises anywhere between `{root}` and \
         `{package_dir}` — looked at {looked_at}"
    )]
    NoManifest {
        target: String,
        root: String,
        package_dir: String,
        looked_at: String,
    },

    #[error(
        "target `{target}`: `{directory}` carries {count} manifests ({names}); which one the \
         package is released from is not something to guess at"
    )]
    AmbiguousManifest {
        target: String,
        directory: String,
        count: usize,
        names: String,
    },

    #[error("{path}: {reason}")]
    Unreadable { path: PathBuf, reason: String },

    #[error(
        "{path}: {field} is absent, and a package cannot be published under a name or a version \
         nobody wrote down"
    )]
    MissingField { path: PathBuf, field: &'static str },

    #[error("{path}: `{value}` is not a version ({reason})")]
    NotAVersion {
        path: PathBuf,
        value: String,
        reason: String,
    },

    #[error("{path}: {field} is {length} characters, past the ceiling of {ceiling}")]
    FieldTooLong {
        path: PathBuf,
        field: &'static str,
        length: usize,
        ceiling: usize,
    },

    #[error(
        "`{name}` is claimed on {registry} by both `{first}` and `{second}`; two packages cannot \
         be published as the same thing"
    )]
    DuplicatePackage {
        registry: &'static str,
        name: String,
        first: String,
        second: String,
    },

    #[error(
        "`{directory}` carries a {kind} manifest and no target claims it, so nothing would ever \
         version or publish it; register it with the generator or mark it unpublishable"
    )]
    UnclaimedPackage { directory: String, kind: Kind },

    #[error(
        "`{name}` ({directory}) is published to {registry} by no job under `ci/`, and `{record}` \
         does not say why; a package nothing publishes ships nowhere and nothing goes red about it"
    )]
    NoPublisher {
        name: String,
        directory: String,
        registry: &'static str,
        record: &'static str,
    },

    #[error(
        "`{file}` names `{directory}`, which is not a package this repository releases; what \
         publishes a package and what says one is published by nothing both have to name one that \
         exists"
    )]
    UnknownPackageNamed { file: String, directory: String },

    #[error(
        "`{record}` records `{directory}` as {field} `{recorded}`, and its manifest resolves to \
         `{resolved}`; a reason about a package this one is no longer is not a reason about this one"
    )]
    RecordedDisagrees {
        record: &'static str,
        directory: String,
        field: &'static str,
        recorded: String,
        resolved: String,
    },

    #[error(
        "`{record}` says `{directory}` has no publish job, and `{file}` publishes it; a reason \
         cannot outlive the absence it explains"
    )]
    RecordedYetPublished {
        record: &'static str,
        directory: String,
        file: String,
    },

    #[error(
        "{path}: the version reads `{found}`, but no line declares it in a form that could be \
         rewritten, so a release would have nowhere to write the new one"
    )]
    NoVersionSite { path: PathBuf, found: String },

    #[error("{path}: written as {requested}, but reads back as `{found}`")]
    WriteNotTaken {
        path: PathBuf,
        requested: String,
        found: String,
    },

    #[error(
        "`{package}` is at {declared} and the release is {expected}; publishing it now would put \
         {declared} on the registry under the {expected} tag"
    )]
    VersionMismatch {
        package: String,
        declared: String,
        expected: String,
    },

    #[error(
        "the SDKs are not at one version — `{first}` is at {first_version} and `{second}` is at \
         {second_version} — so there is no version for a bump to start from"
    )]
    TrainDisagrees {
        first: String,
        first_version: String,
        second: String,
        second_version: String,
    },

    #[error("no package on this release declares a version, so there is nothing to bump")]
    NoVersionAtAll,

    #[error("`git {command}`: {reason}")]
    Git { command: String, reason: String },

    #[error(
        "tags matching `{pattern}` exist and none of them is reachable from HEAD, so there is no \
         last release to read the commits since; releasing from here would weigh every commit that \
         ever touched the package"
    )]
    NoReachableTag { pattern: String },

    #[error(
        "more than {ceiling} commits touch this package since {since}, past the ceiling this tool \
         reads a history under; reading part of it would miss a breaking commit sitting just past \
         the last one read, so raise the ceiling deliberately rather than by accident"
    )]
    TooManyCommits { ceiling: usize, since: Since },

    #[error(
        "`git log` answered {bytes} bytes, past the {ceiling}-byte ceiling a history is read under"
    )]
    HistoryTooLarge { bytes: usize, ceiling: usize },

    #[error(
        "a package is read over {count} paths, past the ceiling of {ceiling} this tool reads one \
         under; raise the ceiling deliberately rather than by accident"
    )]
    TooManyPaths { count: usize, ceiling: usize },

    #[error(
        "`{requested}` is smaller than the release the commits since {since} demand: they require \
         `{required}`, and these are why\n{commits}\nEither the release is a {required} one, or a \
         commit above is marked as something it is not — in which case the fix is a message, not a \
         smaller number."
    )]
    BumpTooSmall {
        requested: Bump,
        required: Bump,
        since: Since,
        commits: String,
    },
}
