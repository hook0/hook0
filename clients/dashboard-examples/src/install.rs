//! What a reader runs to get the SDK the snippet beside it is written against.
//!
//! Nothing here is looked up. The package's name, the registry serving it, the version it is at and
//! whether any job publishes it all come from `ci/release-packages`, which reads them out of the
//! tree; what is written here is one command per ecosystem, keyed on the registry. That keeps the
//! screen right through a rename and through a bump without a line of it being touched, and it is a
//! total function of the registry rather than a list of languages — an ecosystem added to the
//! inventory stops this compiling until somebody says what installs from it.

use release_packages::{Mirror, Package, Registry};

use crate::error::Error;

/// Where this repository is cloned from, which is what the READMEs of the packages no registry
/// carries tell a reader to do.
const REPOSITORY: &str = "https://gitlab.com/hook0/hook0.git";

/// The directory a clone of it lands in.
const CHECKOUT: &str = "hook0";

/// What a Zig consumer saves this package under, which is what they then `@import`.
///
/// Zig has no registry, so the name is the consumer's to pick rather than the package's to declare —
/// `clients/zig/build.zig.zon` and the README both show it saved as this.
const ZIG_DEPENDENCY: &str = "hook0";

/// How a registry is named on screen.
///
/// `Registry::id()` is what the release tooling matches records against, and two of its values are
/// identifiers rather than names — a reader is told a package is on Go modules, not on `go-proxy`.
pub fn name(registry: Registry) -> &'static str {
    match registry {
        Registry::CratesIo => "crates.io",
        Registry::Npm => "npm",
        Registry::PyPi => "PyPI",
        Registry::GoProxy => "Go modules",
        Registry::RubyGems => "RubyGems",
        Registry::Packagist => "Packagist",
        Registry::NuGet => "NuGet",
        Registry::MavenCentral => "Maven Central",
        Registry::LuaRocks => "LuaRocks",
        Registry::ZigFetch => "a tagged archive",
    }
}

/// What installs this package, from its registry when a job publishes it there and from a checkout
/// when none does.
pub fn command(
    package: &Package,
    version: &str,
    published: bool,
    mirrors: &[Mirror],
) -> Result<String, Error> {
    match published {
        true => from_registry(package, version, mirrors),
        false => from_a_checkout(package),
    }
}

/// The one command the package's own ecosystem installs it with.
fn from_registry(package: &Package, version: &str, mirrors: &[Mirror]) -> Result<String, Error> {
    let name = package.name.as_str();
    Ok(match package.registry {
        Registry::CratesIo => format!("cargo add {name}"),
        Registry::Npm => format!("npm install {name}"),
        Registry::PyPi => format!("pip install {name}"),
        // A Go module is named by the address it is fetched at, so its own name is the whole of the
        // command.
        Registry::GoProxy => format!("go get {name}"),
        Registry::RubyGems => format!("gem install {name}"),
        Registry::Packagist => format!("composer require {name}"),
        Registry::NuGet => format!("dotnet add package {name}"),
        // Maven installs nothing from a command line: what a reader does is declare the dependency,
        // which is why this one is a block of XML rather than a line of shell.
        Registry::MavenCentral => {
            let (group, artifact) =
                name.split_once(':')
                    .ok_or_else(|| Error::NotAMavenCoordinate {
                        name: package.name.clone(),
                    })?;
            format!(
                "<dependency>\n  <groupId>{group}</groupId>\n  <artifactId>{artifact}\
                 </artifactId>\n  <version>{version}</version>\n</dependency>"
            )
        }
        Registry::LuaRocks => format!("luarocks install {name}"),
        // Zig resolves a URL serving an archive whose root is the package, which a monorepo cannot
        // be: the archive is a tag of the mirror this client is pushed to on every release.
        Registry::ZigFetch => {
            let mirror = mirrors
                .iter()
                .find(|mirror| mirror.directory == package.directory)
                .ok_or_else(|| Error::NoMirror {
                    directory: package.directory.clone(),
                    reason: "no mirror of this release carries it".to_owned(),
                })?;
            format!(
                "zig fetch --save={ZIG_DEPENDENCY} https://{}/archive/refs/tags/v{version}.tar.gz",
                mirror.url_path()
            )
        }
    })
}

/// What a reader does instead when no job publishes the package: build it out of a checkout, which
/// is what its README already tells them.
///
/// Only the ecosystems that have a package waiting on a registry are written here, and the rest is
/// a refusal rather than a guess. The absence itself is recorded — with what closing it would take
/// — in the file the refusal names.
fn from_a_checkout(package: &Package) -> Result<String, Error> {
    let clone = format!("git clone {REPOSITORY}");
    let directory = package.directory.as_str();
    match package.registry {
        Registry::MavenCentral => Ok(format!(
            "{clone}\nmvn -f {CHECKOUT}/{} install",
            package.manifest
        )),
        // The rockspec carries the version in its own file name, so the command names the file the
        // inventory read the package out of rather than spelling one out of the version.
        Registry::LuaRocks => {
            let rockspec = package
                .manifest
                .rsplit('/')
                .next()
                .unwrap_or(&package.manifest);
            Ok(format!(
                "{clone}\ncd {CHECKOUT}/{directory}\nluarocks install --deps-only {rockspec}\
                 \nluarocks make {rockspec}"
            ))
        }
        other => Err(Error::NoCloneRecipe {
            directory: package.directory.clone(),
            registry: other.id(),
            record: release_packages::NO_PUBLISH_JOB,
        }),
    }
}
