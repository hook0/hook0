//! Pairing the two sides, and refusing whatever is unpaired.
//!
//! One side is the documentation directory: the pages that exist, and what each says it documents.
//! The other is the generator's registry: the clients that exist. Neither is written down here.
//! Everything this module reports is the difference between them — a page for a client that shows
//! nothing, a client whose examples nothing knows how to assemble, a harness for a client no page
//! documents — because those differences are what the next person adding a target will produce,
//! and what a checker that only looked at one side would pass over.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::TargetRoot;
use crate::error::Error;
use crate::harness::{self, Regions};
use crate::limits::{MAX_EXAMPLES, MAX_PAGES};
use crate::manifest::{self, MANIFEST, Manifest};
use crate::page::{self, Example, NO_TARGET, Page};

/// The directory, under the documentation root, that holds one directory per language.
pub const EXAMPLES_DIRECTORY: &str = "examples";

/// The prefix of the file a language writes its harness in.
pub const HARNESS_PREFIX: &str = "harness.";

/// The prefix a scaffold file carries when what is committed is not yet the file it becomes, and is
/// dropped when the project is assembled.
///
/// `go.mod` is what forced it. The committed one holds `{{client}}` where a module path belongs, so
/// it is not a `go.mod` any Go-aware tool can read — and the dependency scanner reads every file of
/// that name in the tree and fails the build on one it cannot parse, which is a security gate going
/// red over a file that declares no dependency at all. The prefix keeps the template out of that
/// set, and the assembled project still receives it under the name its toolchain insists on.
pub const TEMPLATE_PREFIX: &str = "template.";

/// One language's side of the work: what it shows, and what proving it means.
#[derive(Debug)]
pub struct Language {
    /// The target name, which is also the fence language and the directory name.
    pub name: String,
    /// The page documenting this client, absent when examples of it are shown only elsewhere.
    pub page: Option<String>,
    /// The client this language's examples are assembled against, relative to the repository.
    pub client: String,
    pub directory: PathBuf,
    pub harness: PathBuf,
    pub regions: Regions,
    pub manifest: Manifest,
    /// Every example of this language, in the order a reader meets them.
    pub examples: Vec<Example>,
}

/// Everything one run works from.
#[derive(Debug)]
pub struct Documentation {
    pub pages: Vec<Page>,
    pub languages: Vec<Language>,
    /// Targets the registry produces that no page in this directory documents. Reported rather
    /// than refused: the directory documents the language clients, and the registry also produces
    /// things that are not one.
    pub undocumented: Vec<String>,
}

/// Reads the directory against the registry, refusing every mismatch.
///
/// `targets` is handed in rather than read here, which is what lets a target the checker has never
/// seen be put in front of it.
pub fn discover(targets: &[TargetRoot], sdk: &Path) -> Result<Documentation, Error> {
    let names: Vec<String> = targets.iter().map(|target| target.name.clone()).collect();
    let pages = read_pages(sdk, &names)?;

    let mut claims: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for page in &pages {
        let Some(target) = &page.target else {
            return Err(Error::PageClaimsNothing {
                page: page.name.clone(),
            });
        };
        if target == NO_TARGET {
            continue;
        }
        if !names.contains(target) {
            return Err(Error::PageClaimsUnknownTarget {
                page: page.name.clone(),
                target: target.clone(),
                known: names.join(", "),
            });
        }
        claims
            .entry(target.clone())
            .or_default()
            .push(page.name.clone());
    }

    for (target, pages) in &claims {
        if pages.len() > 1 {
            return Err(Error::TargetClaimedTwice {
                target: target.clone(),
                pages: pages.join(", "),
            });
        }
    }

    for page in &pages {
        let Some(target) = &page.target else { continue };
        if target == NO_TARGET {
            continue;
        }
        if !page
            .examples
            .iter()
            .any(|example| &example.language == target)
        {
            return Err(Error::PageWithoutExample {
                page: page.name.clone(),
                target: target.clone(),
            });
        }
    }

    let mut examples: BTreeMap<String, Vec<Example>> = BTreeMap::new();
    let mut total = 0;
    for page in &pages {
        for example in &page.examples {
            total += 1;
            if total > MAX_EXAMPLES {
                return Err(Error::TooMany {
                    path: sdk.display().to_string(),
                    what: "examples",
                    found: total,
                    maximum: MAX_EXAMPLES,
                });
            }
            examples
                .entry(example.language.clone())
                .or_default()
                .push(example.clone());
        }
    }

    // A language is worked on when a page documents it, and when an example of it is shown
    // anywhere — the overview shows the first snippet a reader ever meets, and it is proven like
    // the rest. The registry's own order is kept, which is the order the generator walks.
    let wanted: Vec<&TargetRoot> = targets
        .iter()
        .filter(|target| claims.contains_key(&target.name) || examples.contains_key(&target.name))
        .collect();

    let examples_root = sdk.join(EXAMPLES_DIRECTORY);
    let mut directories = read_directories(&examples_root)?;

    let mut languages = Vec::with_capacity(wanted.len());
    for target in &wanted {
        let name = &target.name;
        let page = claims.get(name).and_then(|pages| pages.first()).cloned();
        match directories.iter().position(|found| found == name) {
            Some(at) => {
                directories.remove(at);
            }
            None => {
                return Err(Error::TargetWithoutHarness {
                    target: name.clone(),
                    page: match &page {
                        Some(page) => page.clone(),
                        None => "the overview".to_owned(),
                    },
                    examples: examples_root.display().to_string(),
                });
            }
        }

        let directory = examples_root.join(name);
        let harness = harness_file(&directory)?;
        languages.push(Language {
            name: name.clone(),
            client: target.client.clone(),
            page,
            regions: harness::read(&harness)?,
            manifest: manifest::read(&directory.join(MANIFEST))?,
            examples: examples.remove(name).unwrap_or_default(),
            harness,
            directory,
        });
    }

    if let Some(directory) = directories.first() {
        let claimed: Vec<&str> = wanted.iter().map(|target| target.name.as_str()).collect();
        return Err(Error::HarnessWithoutTarget {
            examples: examples_root.display().to_string(),
            directory: directory.clone(),
            claimed: claimed.join(", "),
        });
    }

    for language in &languages {
        for example in &language.examples {
            if !language.regions.contains_key(&example.region) {
                let known: Vec<&str> = language.regions.keys().map(String::as_str).collect();
                return Err(Error::UnknownRegion {
                    page: example.page.clone(),
                    line: example.line,
                    region: example.region.clone(),
                    harness: language.harness.display().to_string(),
                    known: known.join(", "),
                });
            }
        }
    }

    let undocumented = names
        .iter()
        .filter(|name| !claims.contains_key(*name))
        .cloned()
        .collect();

    Ok(Documentation {
        pages,
        languages,
        undocumented,
    })
}

/// Every page of the directory, in the order a reader would list them.
fn read_pages(sdk: &Path, targets: &[String]) -> Result<Vec<Page>, Error> {
    let mut names = Vec::new();
    let entries = fs::read_dir(sdk).map_err(|cause| Error::ReadDirectory {
        path: sdk.display().to_string(),
        cause,
    })?;
    for entry in entries {
        let entry = entry.map_err(|cause| Error::ReadDirectory {
            path: sdk.display().to_string(),
            cause,
        })?;
        // The entry is followed rather than read off the directory, so that a tree assembled out
        // of symbolic links — which is how one language is worked on alone — reads the same as a
        // checkout.
        let kind = fs::metadata(entry.path()).map_err(|cause| Error::ReadDirectory {
            path: entry.path().display().to_string(),
            cause,
        })?;
        if !kind.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".md") && !name.ends_with(".mdx") {
            continue;
        }
        if names.len() == MAX_PAGES {
            return Err(Error::TooMany {
                path: sdk.display().to_string(),
                what: "pages",
                found: names.len() + 1,
                maximum: MAX_PAGES,
            });
        }
        names.push(name);
    }
    names.sort();

    names
        .iter()
        .map(|name| page::read(&sdk.join(name), name, targets))
        .collect()
}

/// The names of the directories directly under the examples root.
fn read_directories(examples: &Path) -> Result<Vec<String>, Error> {
    let entries = fs::read_dir(examples).map_err(|cause| Error::ReadDirectory {
        path: examples.display().to_string(),
        cause,
    })?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|cause| Error::ReadDirectory {
            path: examples.display().to_string(),
            cause,
        })?;
        let kind = fs::metadata(entry.path()).map_err(|cause| Error::ReadDirectory {
            path: entry.path().display().to_string(),
            cause,
        })?;
        if !kind.is_dir() {
            continue;
        }
        if names.len() == MAX_PAGES {
            return Err(Error::TooMany {
                path: examples.display().to_string(),
                what: "harness directories",
                found: names.len() + 1,
                maximum: MAX_PAGES,
            });
        }
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

/// The one harness file of a language's directory.
fn harness_file(directory: &Path) -> Result<PathBuf, Error> {
    let entries = fs::read_dir(directory).map_err(|cause| Error::ReadDirectory {
        path: directory.display().to_string(),
        cause,
    })?;

    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|cause| Error::ReadDirectory {
            path: directory.display().to_string(),
            cause,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(HARNESS_PREFIX) {
            found.push(entry.path());
        }
    }
    found.sort();

    let [harness] = &found[..] else {
        return Err(Error::TooMany {
            path: directory.display().to_string(),
            what: "files named harness.*, and exactly one is wanted",
            found: found.len(),
            maximum: 1,
        });
    };
    Ok(harness.clone())
}
