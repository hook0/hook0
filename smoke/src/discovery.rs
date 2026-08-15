//! Which clients must be smoked, and what runs each one.
//!
//! There is no list of clients here. The set comes from the generator's own registry, and a smoke
//! is a directory named after a target — so a twelfth client added tomorrow is smoked without
//! anyone editing this crate, and a target that has no directory beside it is a refusal rather
//! than a client quietly going untested. The mirror-image omission is refused too: a directory
//! that names no target is one whose smoke nothing runs, and it is reported instead of passed
//! over.
//!
//! What runs a smoke is data belonging to the smoke, not a table here: each directory declares its
//! own command in a `smoke.toml`, which is what keeps a language's toolchain out of this file.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;

/// The file a smoke directory declares its command in.
pub const MANIFEST: &str = "smoke.toml";

/// The most smoke directories read in one run. Twelve exist today; a tree past this is one where
/// somebody should be raising this deliberately.
pub const MAX_SMOKES: usize = 64;

/// The most bytes a manifest is read under. These files are three lines long; anything past this
/// is not one.
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// The most words one command may be spelled with.
pub const MAX_COMMAND_WORDS: usize = 32;

/// One client's smoke: where it lives and what runs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Smoke {
    /// The target name it answers to, which is also the name of its directory.
    pub target: String,
    pub directory: PathBuf,
    /// The command, program first, run with [`Smoke::directory`] as the working directory.
    pub command: Vec<String>,
}

/// Pairs every target with the smoke that exercises it, or refuses naming what is unpaired.
///
/// `targets` is handed in rather than read here, which is what lets a target the tool has never
/// seen be put in front of it.
pub fn discover(targets: &[String], languages: &Path) -> Result<Vec<Smoke>, Error> {
    let mut directories = read_directories(languages)?;
    directories.sort();

    let mut smokes = Vec::with_capacity(targets.len());
    let mut missing = Vec::new();
    for target in targets {
        match directories.iter().position(|name| name == target) {
            Some(at) => {
                directories.remove(at);
                let directory = languages.join(target);
                let command = read_command(&directory.join(MANIFEST))?;
                smokes.push(Smoke {
                    target: target.clone(),
                    directory,
                    command,
                });
            }
            None => missing.push(target.clone()),
        }
    }

    if !missing.is_empty() {
        return Err(Error::TargetsWithoutSmoke {
            targets: missing,
            languages: languages.display().to_string(),
        });
    }
    if !directories.is_empty() {
        return Err(Error::SmokesWithoutTarget {
            directories: directories.clone(),
        });
    }

    Ok(smokes)
}

/// The names of the directories directly under `languages`, bounded.
fn read_directories(languages: &Path) -> Result<Vec<String>, Error> {
    let entries = fs::read_dir(languages).map_err(|cause| Error::ReadDirectory {
        path: languages.display().to_string(),
        cause,
    })?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|cause| Error::ReadDirectory {
            path: languages.display().to_string(),
            cause,
        })?;
        let kind = entry.file_type().map_err(|cause| Error::ReadDirectory {
            path: entry.path().display().to_string(),
            cause,
        })?;
        if !kind.is_dir() {
            continue;
        }
        if names.len() == MAX_SMOKES {
            return Err(Error::TooManySmokes {
                path: languages.display().to_string(),
                maximum: MAX_SMOKES,
            });
        }
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    Ok(names)
}

/// The command one manifest declares.
fn read_command(manifest: &Path) -> Result<Vec<String>, Error> {
    let size = fs::metadata(manifest)
        .map_err(|cause| Error::ReadManifest {
            path: manifest.display().to_string(),
            cause,
        })?
        .len();
    if size > MAX_MANIFEST_BYTES {
        return Err(Error::ManifestTooLarge {
            path: manifest.display().to_string(),
            maximum: MAX_MANIFEST_BYTES,
        });
    }

    let body = fs::read_to_string(manifest).map_err(|cause| Error::ReadManifest {
        path: manifest.display().to_string(),
        cause,
    })?;
    let document: toml::Table = body.parse().map_err(|cause| Error::ParseManifest {
        path: manifest.display().to_string(),
        cause: format!("{cause}"),
    })?;

    let refuse = |detail: &str| Error::ParseManifest {
        path: manifest.display().to_string(),
        cause: detail.to_owned(),
    };

    let run = document
        .get("run")
        .ok_or_else(|| refuse("no `run` key, which is what says how the smoke is started"))?
        .as_array()
        .ok_or_else(|| refuse("`run` is not an array of words"))?;

    if run.is_empty() {
        return Err(refuse("`run` names no program"));
    }
    if run.len() > MAX_COMMAND_WORDS {
        return Err(refuse(&format!(
            "`run` is spelled with more than {MAX_COMMAND_WORDS} words"
        )));
    }

    run.iter()
        .map(|word| {
            word.as_str()
                .map(str::to_owned)
                .ok_or_else(|| refuse("`run` carries something that is not a word"))
        })
        .collect()
}
