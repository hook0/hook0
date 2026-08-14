//! Every way discovery refuses, each one naming what it was looking at when it did.
//!
//! There is no variant standing for "skipped": a target whose package cannot be established is the
//! failure mode this tool exists to remove, so it is an error rather than a shorter report.

use std::path::PathBuf;

use crate::manifest::Kind;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path} is {size} bytes, past the {ceiling}-byte ceiling a manifest is read under")]
    ManifestTooLarge {
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
}
