//! Assembling one language's examples into a real project, and running what proves it.
//!
//! Every example of a language is written into one project rather than one project per example:
//! eleven toolchains started once each is the difference between a job that runs and a job nobody
//! waits for. The cost of sharing a project is that a failure names a file rather than a page,
//! which is why every assembled file is remembered next to the page and line it came from, and why
//! a failing run reports the pages whose files the toolchain named.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::discovery::{HARNESS_PREFIX, Language};
use crate::error::Error;
use crate::limits::{MAX_OUTPUT_BYTES, MAX_SCAFFOLD_BYTES, MAX_SCAFFOLD_FILES, POLL_INTERVAL};
use crate::manifest::MANIFEST;
use crate::page::{Example, read_bounded};

/// One assembled example: where it landed, and where it came from.
#[derive(Debug, Clone)]
pub struct Assembled {
    /// The file inside the project, relative to its root.
    pub file: String,
    pub example: Example,
}

/// What running one command answered.
#[derive(Debug)]
pub struct Ran {
    pub command: String,
    /// The exit status, absent when the command was killed by a signal.
    pub code: Option<i32>,
    pub output: String,
    pub cut: bool,
    pub took: Duration,
    /// The one example this command was run against, for the tools that take a file at a time.
    /// Absent for a command run once over the whole project.
    pub against: Option<String>,
}

impl Ran {
    pub fn succeeded(&self) -> bool {
        self.code == Some(0)
    }
}

/// What one language's run answered.
#[derive(Debug)]
pub struct Proven {
    pub language: String,
    pub assembled: Vec<Assembled>,
    pub ran: Vec<Ran>,
    pub took: Duration,
}

impl Proven {
    pub fn succeeded(&self) -> bool {
        self.ran.iter().all(Ran::succeeded)
    }

    /// The examples whatever failed was about.
    ///
    /// Two things can say so. A tool run over the whole project names the file it refused in its
    /// output, and that turns back into the page a reader would open. A tool run one file at a
    /// time was pointed at exactly one example, and that is known without reading anything it
    /// wrote — which is what keeps a linter with a terse message from being reported as a failure
    /// of the project rather than of a page. An example neither names is not reported, so a run
    /// that failed for a reason belonging to the project says so by naming none.
    pub fn blamed(&self) -> Vec<&Assembled> {
        let failed: Vec<&Ran> = self.ran.iter().filter(|ran| !ran.succeeded()).collect();
        let written: String = failed
            .iter()
            .map(|ran| ran.output.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        self.assembled
            .iter()
            .filter(|assembled| {
                written.contains(&assembled.file)
                    || failed
                        .iter()
                        .any(|ran| ran.against.as_deref() == Some(assembled.file.as_str()))
            })
            .collect()
    }
}

/// Assembles a language into `root` and runs everything that proves it.
pub fn prove(
    language: &Language,
    root: &Path,
    repository: &Path,
    client: &Path,
) -> Result<Proven, Error> {
    let started = Instant::now();
    let places = &[
        ("{{repository}}", repository.display().to_string()),
        ("{{client}}", client.display().to_string()),
    ];

    scaffold(&language.directory, root, places)?;
    let assembled = write_examples(language, root)?;

    let mut ran = Vec::new();
    for command in &language.manifest.run {
        let outcome = execute(command, root, language, places, None)?;
        let stop = !outcome.succeeded();
        ran.push(outcome);
        if stop {
            break;
        }
    }

    if ran.iter().all(Ran::succeeded) {
        'commands: for command in &language.manifest.each {
            for one in &assembled {
                let outcome = execute(command, root, language, places, Some(&one.file))?;
                let stop = !outcome.succeeded();
                ran.push(outcome);
                if stop {
                    break 'commands;
                }
            }
        }
    }

    Ok(Proven {
        language: language.name.clone(),
        assembled,
        ran,
        took: started.elapsed(),
    })
}

/// Copies everything of the language's directory that is not the manifest or the harness.
fn scaffold(from: &Path, to: &Path, places: &[(&str, String)]) -> Result<(), Error> {
    let mut copied = 0;
    copy_into(from, to, Path::new(""), places, &mut copied)
}

fn copy_into(
    from: &Path,
    to: &Path,
    relative: &Path,
    places: &[(&str, String)],
    copied: &mut usize,
) -> Result<(), Error> {
    let directory = from.join(relative);
    let entries = fs::read_dir(&directory).map_err(|cause| Error::ReadDirectory {
        path: directory.display().to_string(),
        cause,
    })?;

    for entry in entries {
        let entry = entry.map_err(|cause| Error::ReadDirectory {
            path: directory.display().to_string(),
            cause,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = fs::metadata(entry.path()).map_err(|cause| Error::ReadDirectory {
            path: entry.path().display().to_string(),
            cause,
        })?;
        let inner = relative.join(&name);

        if kind.is_dir() {
            copy_into(from, to, &inner, places, copied)?;
            continue;
        }
        if name == MANIFEST || name.starts_with(HARNESS_PREFIX) {
            continue;
        }

        *copied += 1;
        if *copied > MAX_SCAFFOLD_FILES {
            return Err(Error::TooMany {
                path: from.display().to_string(),
                what: "scaffold files",
                found: *copied,
                maximum: MAX_SCAFFOLD_FILES,
            });
        }

        let body = substitute(
            &read_bounded(&entry.path(), "scaffold file", MAX_SCAFFOLD_BYTES)?,
            places,
        );
        write(&to.join(&inner), &body)?;
    }
    Ok(())
}

/// Writes every example of the language into the project, remembering where each came from.
fn write_examples(language: &Language, root: &Path) -> Result<Vec<Assembled>, Error> {
    let mut ordinals: BTreeMap<String, usize> = BTreeMap::new();
    let mut assembled = Vec::with_capacity(language.examples.len());

    for example in &language.examples {
        let stem = example
            .page
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(&example.page)
            .replace(['-', '.'], "_");
        let ordinal = ordinals.entry(stem.clone()).or_insert(0);
        *ordinal += 1;
        let name = format!("{stem}_{:02}", *ordinal);
        let upper = pascal(&name);
        let places = &[("{{name}}", name.clone()), ("{{Name}}", upper)];

        let region = language
            .regions
            .get(&example.region)
            .ok_or_else(|| Error::UnknownRegion {
                page: example.page.clone(),
                line: example.line,
                region: example.region.clone(),
                harness: language.harness.display().to_string(),
                known: language
                    .regions
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<&str>>()
                    .join(", "),
            })?;

        let file = substitute(&language.manifest.path, places);
        write(
            &root.join(&file),
            &substitute(&region.fill(&example.body), places),
        )?;
        assembled.push(Assembled {
            file,
            example: example.clone(),
        });
    }

    Ok(assembled)
}

/// Runs one command in the project, bounded in time and in how much of its output is kept.
fn execute(
    command: &[String],
    root: &Path,
    language: &Language,
    places: &[(&str, String)],
    file: Option<&str>,
) -> Result<Ran, Error> {
    let words: Vec<String> = command
        .iter()
        .map(|word| {
            let word = substitute(word, places);
            match file {
                Some(file) => word.replace("{{file}}", file),
                None => word,
            }
        })
        .collect();
    let spelled = words.join(" ");

    let log = root.join(".documentation-examples.log");
    let sink = File::create(&log).map_err(|cause| Error::WriteFile {
        path: log.display().to_string(),
        cause,
    })?;
    let errors = sink.try_clone().map_err(|cause| Error::WriteFile {
        path: log.display().to_string(),
        cause,
    })?;

    let started = Instant::now();
    let mut child = Command::new(&words[0])
        .args(&words[1..])
        .current_dir(root)
        .envs(&language.manifest.environment)
        .stdin(Stdio::null())
        .stdout(sink)
        .stderr(errors)
        .spawn()
        .map_err(|cause| Error::CommandNotStarted {
            language: language.name.clone(),
            command: spelled.clone(),
            cause,
        })?;

    let deadline = started + language.manifest.timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(cause) => {
                return Err(Error::CommandNotStarted {
                    language: language.name.clone(),
                    command: spelled,
                    cause,
                });
            }
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Timeout {
                language: language.name.clone(),
                command: spelled,
                seconds: language.manifest.timeout.as_secs(),
                manifest: language.directory.join(MANIFEST).display().to_string(),
            });
        }
        thread::sleep(POLL_INTERVAL);
    };

    let (output, cut) = tail(&log)?;
    Ok(Ran {
        command: spelled,
        code: status.code(),
        output,
        cut,
        took: started.elapsed(),
        against: file.map(str::to_owned),
    })
}

/// The end of a command's output, and whether there was more of it.
fn tail(log: &Path) -> Result<(String, bool), Error> {
    let mut file = File::open(log).map_err(|cause| Error::ReadFile {
        path: log.display().to_string(),
        cause,
    })?;
    let mut body = Vec::new();
    file.read_to_end(&mut body)
        .map_err(|cause| Error::ReadFile {
            path: log.display().to_string(),
            cause,
        })?;

    let cut = body.len() > MAX_OUTPUT_BYTES;
    if cut {
        body = body.split_off(body.len() - MAX_OUTPUT_BYTES);
    }
    Ok((String::from_utf8_lossy(&body).into_owned(), cut))
}

fn write(path: &Path, body: &str) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|cause| Error::WriteFile {
            path: parent.display().to_string(),
            cause,
        })?;
    }
    fs::write(path, body).map_err(|cause| Error::WriteFile {
        path: path.display().to_string(),
        cause,
    })
}

fn substitute(body: &str, places: &[(&str, String)]) -> String {
    let mut out = body.to_owned();
    for (token, value) in places {
        out = out.replace(token, value);
    }
    out
}

/// `javascript_03` as `Javascript03`, for the languages where a file names a type.
fn pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Where a language's project is assembled.
pub fn root_for(work: &Path, language: &str) -> PathBuf {
    work.join(language)
}
