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
//!
//! So is what a smoke needs before it can run. A runtime whose packages live outside the system
//! path — Lua is this repository's one — is reachable only through a search path somebody exported,
//! and a harness that inherits one runs or fails on state nobody declared. Such a smoke declares
//! the command that answers where its packages are, and the harness asks rather than inherits.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::Error;
use crate::process;

/// The file a smoke directory declares its command in.
pub const MANIFEST: &str = "smoke.toml";

/// The key a smoke says with when it does not yet drive the whole generated surface.
///
/// Absent means it does, which is the state every smoke ends up in and the only state a client
/// added tomorrow can be in. It is written in the smoke's own manifest rather than as a list of
/// ported languages in this crate for the reason everything else about a smoke is: the day a
/// language starts driving the surface, the line saying it does not is deleted in the same
/// directory as the code that drives it, and once deleted there is no way back to silence — a
/// smoke with no such line and no reports fails the run.
pub const DRIVES_SURFACE: &str = "drives_surface";

/// The most smoke directories read in one run. Twelve exist today; a tree past this is one where
/// somebody should be raising this deliberately.
pub const MAX_SMOKES: usize = 64;

/// The most bytes a manifest is read under. These files are three lines long; anything past this
/// is not one.
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// The most words one command may be spelled with.
pub const MAX_COMMAND_WORDS: usize = 32;

/// The most requirements one smoke may declare. A smoke needing more than this of its machine is
/// one whose setup belongs in an image rather than in a manifest.
pub const MAX_REQUIREMENTS: usize = 8;

/// The most bytes one requirement's answer is read as a value. Search paths are long; a program
/// answering more than this is not answering a search path.
pub const MAX_DERIVED_BYTES: usize = 64 * 1024;

/// How long one requirement is given to answer. These ask a package manager where it put things,
/// which is a local lookup; anything past this is a lookup that has gone looking on the network.
pub const REQUIREMENT_WITHIN: Duration = Duration::from_secs(60);

/// Something that has to hold before a smoke can run, and what to do when it does not.
///
/// A requirement is a command rather than a rule this crate knows, for the same reason the smoke's
/// own command is: `luarocks` belongs to Lua, and a harness that understood it would have to
/// understand the next runtime too. When the command also names a variable, what it prints becomes
/// that variable for the smoke — which is how a search path gets derived rather than inherited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The command, program first. It must exit zero.
    pub run: Vec<String>,
    /// The variable the smoke is given, set to what `run` said with `suffix` after it.
    pub sets: Option<String>,
    /// What the value ends with, for a syntax that has a way of saying "and then the usual places".
    pub suffix: String,
    /// The line that makes the requirement hold, printed when it does not.
    pub remedy: String,
}

/// One client's smoke: where it lives, what runs it, and what it needs first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Smoke {
    /// The target name it answers to, which is also the name of its directory.
    pub target: String,
    pub directory: PathBuf,
    /// The command, program first, run with [`Smoke::directory`] as the working directory.
    pub command: Vec<String>,
    /// What has to hold before the command is run, in the order it is asked.
    pub requires: Vec<Requirement>,
    /// Whether this smoke drives every operation the API document declares, which is what the run
    /// holds it to. True unless its manifest says otherwise.
    pub drives_surface: bool,
}

impl Smoke {
    /// Holds every requirement this smoke declares, answering the variables they set.
    ///
    /// Refusing here rather than letting the smoke start is the whole point: a runtime that cannot
    /// find its packages fails with a list of the directories it looked in, which names everywhere
    /// the answer is not and nowhere it might be. What comes back instead is the one line that
    /// installs the thing.
    pub fn satisfied(&self) -> Result<Vec<(String, String)>, Error> {
        let mut derived = Vec::new();

        for requirement in &self.requires {
            let (program, arguments) = requirement.run.split_at(1);
            let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();

            let unmet = |said: String| Error::RequirementUnmet {
                target: self.target.clone(),
                program: requirement.run.join(" "),
                said,
                remedy: requirement.remedy.clone(),
            };

            let ended = match process::capture(&program[0], &borrowed, REQUIREMENT_WITHIN) {
                Ok(ended) => ended,
                Err(cause) => return Err(unmet(format!("{cause}"))),
            };
            if !ended.ok {
                return Err(unmet(format!("it {}\n{}", ended.status, ended.output)));
            }

            if let Some(name) = &requirement.sets {
                let said = ended.output.trim();
                if said.len() > MAX_DERIVED_BYTES {
                    return Err(unmet(format!(
                        "it answered more than the {MAX_DERIVED_BYTES} bytes read as a value"
                    )));
                }
                derived.push((name.clone(), format!("{said}{}", requirement.suffix)));
            }
        }

        Ok(derived)
    }
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
                let manifest = read_manifest(&directory.join(MANIFEST))?;
                smokes.push(Smoke {
                    target: target.clone(),
                    directory,
                    command: manifest.command,
                    requires: manifest.requires,
                    drives_surface: manifest.drives_surface,
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

/// Everything one manifest declares about its smoke.
struct Manifest {
    command: Vec<String>,
    requires: Vec<Requirement>,
    drives_surface: bool,
}

/// The command one manifest declares, what it needs before that command is run, and whether it
/// drives the whole generated surface.
fn read_manifest(manifest: &Path) -> Result<Manifest, Error> {
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
        .ok_or_else(|| refuse("no `run` key, which is what says how the smoke is started"))?;
    let command = words(run, "run", &refuse)?;

    let requires = match document.get("requires") {
        None => Vec::new(),
        Some(declared) => {
            let entries = declared
                .as_array()
                .ok_or_else(|| refuse("`requires` is not a list of requirements"))?;
            if entries.len() > MAX_REQUIREMENTS {
                return Err(refuse(&format!(
                    "`requires` names more than {MAX_REQUIREMENTS} things to hold first"
                )));
            }
            entries
                .iter()
                .map(|entry| read_requirement(entry, &refuse))
                .collect::<Result<Vec<_>, Error>>()?
        }
    };

    let drives_surface = match document.get(DRIVES_SURFACE) {
        None => true,
        Some(declared) => declared.as_bool().ok_or_else(|| {
            refuse(&format!(
                "`{DRIVES_SURFACE}` is not `true` or `false`, and it is the only thing that tells a smoke still to be written from one that has stopped reporting"
            ))
        })?,
    };

    Ok(Manifest {
        command,
        requires,
        drives_surface,
    })
}

/// One `[[requires]]` entry.
fn read_requirement(
    entry: &toml::Value,
    refuse: &impl Fn(&str) -> Error,
) -> Result<Requirement, Error> {
    let table = entry
        .as_table()
        .ok_or_else(|| refuse("`requires` carries something that is not a requirement"))?;

    let run = table.get("run").ok_or_else(|| {
        refuse("a requirement names no `run`, so nothing decides whether it holds")
    })?;
    let run = words(run, "requires.run", refuse)?;

    let remedy = table
        .get("remedy")
        .ok_or_else(|| {
            refuse("a requirement names no `remedy`, so a machine without it is told nothing")
        })?
        .as_str()
        .ok_or_else(|| refuse("`requires.remedy` is not a line of text"))?
        .to_owned();

    let sets = match table.get("sets") {
        None => None,
        Some(name) => Some(
            name.as_str()
                .ok_or_else(|| refuse("`requires.sets` is not the name of a variable"))?
                .to_owned(),
        ),
    };

    let suffix = match table.get("suffix") {
        None => String::new(),
        Some(text) => text
            .as_str()
            .ok_or_else(|| refuse("`requires.suffix` is not text"))?
            .to_owned(),
    };

    if sets.is_none() && !suffix.is_empty() {
        return Err(refuse(
            "`requires.suffix` sets nothing, so nothing carries it",
        ));
    }

    Ok(Requirement {
        run,
        sets,
        suffix,
        remedy,
    })
}

/// A command spelled as an array of words, bounded and refused when it is anything else.
fn words(
    declared: &toml::Value,
    named: &str,
    refuse: &impl Fn(&str) -> Error,
) -> Result<Vec<String>, Error> {
    let spelled = declared
        .as_array()
        .ok_or_else(|| refuse(&format!("`{named}` is not an array of words")))?;

    if spelled.is_empty() {
        return Err(refuse(&format!("`{named}` names no program")));
    }
    if spelled.len() > MAX_COMMAND_WORDS {
        return Err(refuse(&format!(
            "`{named}` is spelled with more than {MAX_COMMAND_WORDS} words"
        )));
    }

    spelled
        .iter()
        .map(|word| {
            word.as_str()
                .map(str::to_owned)
                .ok_or_else(|| refuse(&format!("`{named}` carries something that is not a word")))
        })
        .collect()
}
