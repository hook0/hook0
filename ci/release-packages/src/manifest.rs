//! What a package manifest is, one shape per ecosystem, and what can be read out of one.
//!
//! Nothing here names a package. A kind is recognised by the file name its ecosystem insists on,
//! and the name and version are whatever that file declares — which is what lets a target nobody
//! wrote this for be read correctly the day it lands, and what makes an unrecognised shape a
//! refusal naming the file rather than a package quietly missing from a release.
//!
//! Two of these declare no version at all, and that is not an omission to paper over: a Go module
//! and a Composer package are versioned by the tag the host answers, so [`Version::FromTag`] says
//! so rather than inventing a number for them.

use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::error::Error;

/// The most bytes a manifest is read under. Every one of these files is written by hand and
/// measured in kilobytes; anything past this is not a manifest.
pub const MAX_MANIFEST_BYTES: u64 = 1 << 20;

/// The longest a package name may be, which is npm's own ceiling and the strictest of the
/// registries reached here.
pub const MAX_NAME_CHARS: usize = 214;

/// The longest a version string may be before it is refused unread.
pub const MAX_VERSION_CHARS: usize = 64;

/// A manifest shape, named after the ecosystem that insists on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Cargo,
    Npm,
    PyProject,
    GoMod,
    Gemspec,
    Composer,
    Csproj,
    Pom,
    Gradle,
    Rockspec,
    Zon,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Cargo => "Cargo.toml",
            Kind::Npm => "package.json",
            Kind::PyProject => "pyproject.toml",
            Kind::GoMod => "go.mod",
            Kind::Gemspec => "gemspec",
            Kind::Composer => "composer.json",
            Kind::Csproj => "csproj",
            Kind::Pom => "pom.xml",
            Kind::Gradle => "Gradle build file",
            Kind::Rockspec => "rockspec",
            Kind::Zon => "build.zig.zon",
        })
    }
}

/// Where a package of this kind is published, and under what a name of it is unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Registry {
    CratesIo,
    Npm,
    PyPi,
    GoProxy,
    RubyGems,
    Packagist,
    NuGet,
    MavenCentral,
    LuaRocks,
    ZigFetch,
}

impl Registry {
    pub fn id(self) -> &'static str {
        match self {
            Registry::CratesIo => "crates.io",
            Registry::Npm => "npm",
            Registry::PyPi => "PyPI",
            Registry::GoProxy => "go-proxy",
            Registry::RubyGems => "RubyGems",
            Registry::Packagist => "Packagist",
            Registry::NuGet => "NuGet",
            Registry::MavenCentral => "Maven-Central",
            Registry::LuaRocks => "LuaRocks",
            Registry::ZigFetch => "zig-fetch",
        }
    }
}

impl fmt::Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl Kind {
    /// The kind a file of this name is, or nothing if the name belongs to no ecosystem read here.
    pub fn of(file_name: &str) -> Option<Kind> {
        match file_name {
            "Cargo.toml" => Some(Kind::Cargo),
            "package.json" => Some(Kind::Npm),
            "pyproject.toml" => Some(Kind::PyProject),
            "go.mod" => Some(Kind::GoMod),
            "composer.json" => Some(Kind::Composer),
            "pom.xml" => Some(Kind::Pom),
            "build.gradle.kts" | "build.gradle" => Some(Kind::Gradle),
            "build.zig.zon" => Some(Kind::Zon),
            _ if file_name.ends_with(".gemspec") => Some(Kind::Gemspec),
            _ if file_name.ends_with(".csproj") => Some(Kind::Csproj),
            _ if file_name.ends_with(".rockspec") => Some(Kind::Rockspec),
            _ => None,
        }
    }

    pub fn registry(self) -> Registry {
        match self {
            Kind::Cargo => Registry::CratesIo,
            Kind::Npm => Registry::Npm,
            Kind::PyProject => Registry::PyPi,
            Kind::GoMod => Registry::GoProxy,
            Kind::Gemspec => Registry::RubyGems,
            Kind::Composer => Registry::Packagist,
            Kind::Csproj => Registry::NuGet,
            Kind::Pom | Kind::Gradle => Registry::MavenCentral,
            Kind::Rockspec => Registry::LuaRocks,
            Kind::Zon => Registry::ZigFetch,
        }
    }
}

/// Where a version is written down, so that setting one replaces exactly what reading one saw.
///
/// The file is not always the manifest: a gemspec names a constant rather than a literal, and the
/// constant lives in the file the gemspec requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionSite {
    pub file: PathBuf,
    pub span: Range<usize>,
    pub value: semver::Version,
}

/// What a package is versioned by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Version {
    /// The manifest — or the file it points at — says so.
    Declared(VersionSite),
    /// Nothing in the tree says so: the host answers this package at the version of the tag it is
    /// reached under, so there is no number here to bump and none to check a tag against.
    FromTag,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Declared(site) => write!(f, "{}", site.value),
            Version::FromTag => f.write_str("(from tag)"),
        }
    }
}

/// A manifest, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub kind: Kind,
    pub name: String,
    pub version: Version,
    /// Whether the ecosystem would accept this being published at all. A manifest saying it is
    /// private is how a package that is deliberately never released says so.
    pub publishable: bool,
}

/// Read the manifest at `path`, which is relative to `tree`.
pub fn read(tree: &Path, path: &Path, kind: Kind) -> Result<Manifest, Error> {
    let body = read_bounded(&tree.join(path))?;
    let directory = path.parent().unwrap_or(Path::new("")).to_path_buf();

    let manifest = match kind {
        Kind::Cargo => toml_table(path, &body, kind, "package", Some("publish"))?,
        Kind::PyProject => toml_table(path, &body, kind, "project", None)?,
        Kind::Npm => npm(path, &body)?,
        Kind::Composer => composer(path, &body)?,
        Kind::GoMod => go_mod(path, &body)?,
        Kind::Gemspec => gemspec(tree, path, &directory, &body)?,
        Kind::Csproj => csproj(path, &body)?,
        Kind::Pom => pom(path, &body)?,
        Kind::Gradle => gradle(tree, path, &directory, &body)?,
        Kind::Rockspec => assignments(path, &body, kind, "package", "version")?,
        Kind::Zon => assignments(path, &body, kind, ".name", ".version")?,
    };

    let length = manifest.name.chars().count();
    if length > MAX_NAME_CHARS {
        return Err(Error::FieldTooLong {
            path: path.to_path_buf(),
            field: "name",
            length,
            ceiling: MAX_NAME_CHARS,
        });
    }
    Ok(manifest)
}

/// Read a file, refusing one past the ceiling rather than pulling it into memory to find out.
pub fn read_bounded(path: &Path) -> Result<String, Error> {
    let size = std::fs::metadata(path)
        .map_err(|source| Error::Read {
            path: path.to_path_buf(),
            source,
        })?
        .len();
    if size > MAX_MANIFEST_BYTES {
        return Err(Error::ManifestTooLarge {
            path: path.to_path_buf(),
            size,
            ceiling: MAX_MANIFEST_BYTES,
        });
    }
    std::fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })
}

// --- One reader per shape -----------------------------------------------------------------------

/// `Cargo.toml` and `pyproject.toml`: one table holding the name beside the version.
///
/// `publish_key` is the member that says the package is not for a registry — `publish = false` in
/// a `Cargo.toml`, and nothing at all in a `pyproject.toml`, which has no such statement.
fn toml_table(
    path: &Path,
    body: &str,
    kind: Kind,
    table: &str,
    publish_key: Option<&str>,
) -> Result<Manifest, Error> {
    let value: toml::Value = toml::from_str(body).map_err(|e| Error::Unreadable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    let section = value.get(table).ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "the package table",
    })?;
    let name = section
        .get("name")
        .and_then(toml::Value::as_str)
        .ok_or(Error::MissingField {
            path: path.to_path_buf(),
            field: "name",
        })?;
    let declared =
        section
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or(Error::MissingField {
                path: path.to_path_buf(),
                field: "version",
            })?;
    let publishable = match publish_key {
        Some(key) => section
            .get(key)
            .and_then(toml::Value::as_bool)
            .unwrap_or(true),
        None => true,
    };

    Ok(Manifest {
        kind,
        name: name.to_string(),
        version: Version::Declared(site_in_table(path, body, table, "version", declared)?),
        publishable,
    })
}

/// `package.json`, which says outright when it is not to be published.
fn npm(path: &Path, body: &str) -> Result<Manifest, Error> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| Error::Unreadable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    let name = json_string(path, &value, "name")?;
    let declared = json_string(path, &value, "version")?;
    let publishable = !value
        .get("private")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    Ok(Manifest {
        kind: Kind::Npm,
        name,
        version: Version::Declared(site_by_key(
            path,
            body,
            "\"version\"",
            &declared,
            &declared,
        )?),
        publishable,
    })
}

/// `composer.json`. Packagist reads the version off the tag rather than out of the file, and
/// declaring one here is what its own documentation tells packages not to do.
fn composer(path: &Path, body: &str) -> Result<Manifest, Error> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| Error::Unreadable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    Ok(Manifest {
        kind: Kind::Composer,
        name: json_string(path, &value, "name")?,
        version: Version::FromTag,
        publishable: true,
    })
}

/// `go.mod`. A module is named by the URL it is reached at and versioned by the tag the host
/// answers, so the file carries the first and nothing of the second.
fn go_mod(path: &Path, body: &str) -> Result<Manifest, Error> {
    let module = body
        .lines()
        .map(code_of)
        .find_map(|line| line.strip_prefix("module "))
        .map(str::trim)
        .filter(|module| !module.is_empty())
        .ok_or(Error::MissingField {
            path: path.to_path_buf(),
            field: "module",
        })?;
    Ok(Manifest {
        kind: Kind::GoMod,
        name: module.to_string(),
        version: Version::FromTag,
        publishable: true,
    })
}

/// A gemspec, whose version is a constant rather than a literal.
///
/// Every gem writes it the same way — a `require_relative` of a file holding `VERSION` — because
/// the gem has to be able to read its own version at runtime too. The indirection is followed
/// rather than guessed at: a gemspec naming no such file, or a file declaring no constant, is a
/// refusal naming both paths.
fn gemspec(tree: &Path, path: &Path, directory: &Path, body: &str) -> Result<Manifest, Error> {
    let name = assigned(body, "spec.name").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "spec.name",
    })?;

    if let Some(declared) = assigned(body, "spec.version") {
        return Ok(Manifest {
            kind: Kind::Gemspec,
            name,
            version: Version::Declared(site_by_key(
                path,
                body,
                "spec.version",
                &declared,
                &declared,
            )?),
            publishable: true,
        });
    }

    let required = quoted_argument(body, "require_relative").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "spec.version, and no require_relative to look for it behind",
    })?;
    let version_file = directory.join(format!("{required}.rb"));
    let version_body = read_bounded(&tree.join(&version_file))?;
    let declared = assigned(&version_body, "VERSION").ok_or(Error::MissingField {
        path: version_file.clone(),
        field: "VERSION",
    })?;
    Ok(Manifest {
        kind: Kind::Gemspec,
        name,
        version: Version::Declared(site_by_key(
            &version_file,
            &version_body,
            "VERSION",
            &declared,
            &declared,
        )?),
        publishable: true,
    })
}

/// A `.csproj`, whose package identity is a property rather than an element of its own.
fn csproj(path: &Path, body: &str) -> Result<Manifest, Error> {
    let property = |name: &str| {
        crate::xml::elements_at(body, &["PropertyGroup", name])
            .into_iter()
            .next()
    };
    let name = property("PackageId").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "PackageId",
    })?;
    let version = property("Version").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "Version",
    })?;
    let publishable = property("IsPackable").is_none_or(|e| !e.text.eq_ignore_ascii_case("false"));
    Ok(Manifest {
        kind: Kind::Csproj,
        name: name.text,
        version: Version::Declared(VersionSite {
            file: path.to_path_buf(),
            value: parse_version(path, &version.text)?,
            span: version.span,
        }),
        publishable,
    })
}

/// A `pom.xml`, whose coordinates are the pair a Maven artefact is reached by.
fn pom(path: &Path, body: &str) -> Result<Manifest, Error> {
    let own = |name: &str| crate::xml::elements_at(body, &[name]).into_iter().next();
    let group = own("groupId").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "groupId",
    })?;
    let artifact = own("artifactId").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "artifactId",
    })?;
    let version = own("version").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "version",
    })?;
    Ok(Manifest {
        kind: Kind::Pom,
        name: format!("{}:{}", group.text, artifact.text),
        version: Version::Declared(VersionSite {
            file: path.to_path_buf(),
            value: parse_version(path, &version.text)?,
            span: version.span,
        }),
        publishable: true,
    })
}

/// A Gradle build file, whose coordinates are split across two files.
///
/// The build declares the group and the version; what the artefact is called is `rootProject.name`
/// in the settings file beside it, and Gradle's own default for that — the directory the build sits
/// in — is what stands in when the settings file does not say.
fn gradle(tree: &Path, path: &Path, directory: &Path, body: &str) -> Result<Manifest, Error> {
    let group = assigned(body, "group").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "group",
    })?;
    let declared = assigned(body, "version").ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "version",
    })?;

    let artifact = ["settings.gradle.kts", "settings.gradle"]
        .into_iter()
        .map(|file| directory.join(file))
        .filter(|file| tree.join(file).is_file())
        .find_map(|file| assigned(&read_bounded(&tree.join(&file)).ok()?, "rootProject.name"))
        .or_else(|| {
            directory
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .ok_or(Error::MissingField {
            path: path.to_path_buf(),
            field: "rootProject.name",
        })?;

    Ok(Manifest {
        kind: Kind::Gradle,
        name: format!("{group}:{artifact}"),
        version: Version::Declared(site_by_key(path, body, "version", &declared, &declared)?),
        publishable: true,
    })
}

/// The shapes that declare what they are as plain assignments: a rockspec and a `build.zig.zon`.
///
/// The name is read quoted or bare, since Zig moved from one spelling to the other and a package
/// released under either is the same package.
///
/// A rockspec version is not a version alone: LuaRocks writes `<version>-<revision>`, where the
/// revision counts repackagings of the same upstream release and has nothing to do with what the
/// package is a version of. Only the part in front of it is read, and only that part is what a bump
/// replaces — so the revision survives being versioned instead of being written over with nothing.
fn assignments(
    path: &Path,
    body: &str,
    kind: Kind,
    name_key: &str,
    version_key: &str,
) -> Result<Manifest, Error> {
    let name = assigned(body, name_key)
        .or_else(|| bare_assigned(body, name_key))
        .ok_or(Error::MissingField {
            path: path.to_path_buf(),
            field: "name",
        })?;
    let declared = assigned(body, version_key).ok_or(Error::MissingField {
        path: path.to_path_buf(),
        field: "version",
    })?;
    let core = match kind {
        Kind::Rockspec => declared.rsplit_once('-').map_or(&declared[..], |(v, _)| v),
        _ => &declared,
    };
    Ok(Manifest {
        kind,
        name,
        version: Version::Declared(site_by_key(path, body, version_key, &declared, core)?),
        publishable: true,
    })
}

// --- Reading a value, and remembering where it was ----------------------------------------------

fn json_string(
    path: &Path,
    value: &serde_json::Value,
    field: &'static str,
) -> Result<String, Error> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or(Error::MissingField {
            path: path.to_path_buf(),
            field,
        })
}

/// A line with any trailing comment removed, so that a commented-out declaration is not read as one.
fn code_of(line: &str) -> &str {
    let mut code = line;
    for marker in ["//", "#", "--"] {
        code = code.split(marker).next().unwrap_or(code);
    }
    code.trim()
}

/// What sits after `key` and its separator on this line, or nothing if the line assigns something
/// else. The key has to be the whole of what precedes the separator, so `spec.version` is not read
/// off the line assigning `spec.required_ruby_version`.
fn after_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (left, right) = code_of(line).split_once(['=', ':'])?;
    (left.trim() == key).then_some(right)
}

/// The quoted string assigned to `key` on the first line that assigns it.
fn assigned(body: &str, key: &str) -> Option<String> {
    body.lines()
        .find_map(|line| assigned_on(line, key))
        .map(str::to_string)
}

/// The bare token assigned to `key`, for the one shape that writes a name unquoted.
fn bare_assigned(body: &str, key: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let token = after_key(line, key)?.trim().trim_start_matches('.');
        let end = token
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(token.len());
        Some(token[..end].to_string()).filter(|token| !token.is_empty())
    })
}

/// The first quoted string handed to a call of `name`, for the one thing read here that is a call
/// rather than an assignment.
fn quoted_argument(body: &str, name: &str) -> Option<String> {
    body.lines()
        .filter_map(|line| code_of(line).strip_prefix(name))
        .find_map(quoted_in)
        .map(str::to_string)
}

fn assigned_on<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    quoted_in(after_key(line, key)?)
}

fn quoted_in(text: &str) -> Option<&str> {
    let (_, after_quote) = text.split_once(['"', '\''])?;
    let end = after_quote.find(['"', '\''])?;
    Some(&after_quote[..end])
}

/// Where the version `declared` is written, searched inside the one table it belongs to rather than
/// across the whole file — a `Cargo.toml` declares a `version` under every dependency it has.
fn site_in_table(
    path: &Path,
    body: &str,
    table: &str,
    key: &str,
    declared: &str,
) -> Result<VersionSite, Error> {
    let header = format!("[{table}]");
    let from = body.find(&header).map_or(0, |at| at + header.len());
    let to = body[from..]
        .find("\n[")
        .map_or(body.len(), |at| from + at + 1);
    let mut site = site_by_key(path, &body[from..to], key, declared, declared)?;
    site.file = path.to_path_buf();
    site.span = from + site.span.start..from + site.span.end;
    Ok(site)
}

/// Where the version is written, as the byte range a bump replaces.
///
/// The whole body is searched rather than one line at a time, because a manifest is free to put
/// everything it says on one line and a release still has to be able to write to it.
///
/// `declared` is the whole of what the manifest assigns and `core` is the part of it that is the
/// version — the two differ only where an ecosystem writes something else alongside, and the span
/// answered covers `core` alone so that the something else is left where it is.
fn site_by_key(
    path: &Path,
    body: &str,
    key: &str,
    declared: &str,
    core: &str,
) -> Result<VersionSite, Error> {
    let value = parse_version(path, core)?;
    let mut from = 0usize;
    while let Some(at) = body[from..].find(key).map(|found| from + found) {
        from = at + key.len();
        if !starts_a_token(body, at) || commented_out(body, at) {
            continue;
        }
        match quoted_after_separator(body, from) {
            Some(span) if body[span.clone()] == *declared => {
                return Ok(VersionSite {
                    file: path.to_path_buf(),
                    span: span.start..span.start + core.len(),
                    value,
                });
            }
            _ => continue,
        }
    }
    Err(Error::NoVersionSite {
        path: path.to_path_buf(),
        found: declared.to_string(),
    })
}

/// Whether the key found at `at` starts there, rather than ending some longer word — so that
/// `version` is not read off `required_ruby_version`.
fn starts_a_token(body: &str, at: usize) -> bool {
    body[..at]
        .chars()
        .next_back()
        .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '-'))
}

/// Whether what was found sits behind a comment marker on its own line, which is a declaration
/// somebody turned off rather than the one a release writes to.
fn commented_out(body: &str, at: usize) -> bool {
    let line_start = body[..at].rfind('\n').map_or(0, |newline| newline + 1);
    let before = &body[line_start..at];
    before.contains('#') || before.contains("//") || before.contains("--")
}

/// The quoted string assigned right after `from`, as the byte range of the value itself.
fn quoted_after_separator(body: &str, from: usize) -> Option<Range<usize>> {
    let rest = &body[from..];
    let separator = rest.find(|c: char| !c.is_whitespace())?;
    if !matches!(rest.as_bytes()[separator], b'=' | b':') {
        return None;
    }
    let after = separator + 1;
    let opening = after + rest[after..].find(|c: char| !c.is_whitespace())?;
    let quote = rest.as_bytes()[opening];
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let start = opening + 1;
    let end = start + rest[start..].find(char::from(quote))?;
    Some(from + start..from + end)
}

fn parse_version(path: &Path, declared: &str) -> Result<semver::Version, Error> {
    let length = declared.chars().count();
    if length > MAX_VERSION_CHARS {
        return Err(Error::FieldTooLong {
            path: path.to_path_buf(),
            field: "version",
            length,
            ceiling: MAX_VERSION_CHARS,
        });
    }
    semver::Version::parse(declared).map_err(|e| Error::NotAVersion {
        path: path.to_path_buf(),
        value: declared.to_string(),
        reason: e.to_string(),
    })
}
