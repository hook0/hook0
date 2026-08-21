//! How one language's examples are assembled and what proving one of them means.
//!
//! There is no table of toolchains in this crate. Each language declares its own, in an
//! `examples.toml` beside its harness, the way each smoke declares the command that starts it —
//! which is what keeps `dotnet`, `luac` and `zig` out of the orchestrator, and what lets a twelfth
//! language arrive as a directory rather than as a patch here.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use crate::error::Error;
use crate::limits::{MAX_COMMAND_WORDS, MAX_COMMANDS, MAX_MANIFEST_BYTES, MAX_TIMEOUT};
use crate::page::read_bounded;

/// The file a language declares its proof in.
pub const MANIFEST: &str = "examples.toml";

/// How far a language's examples are taken, stated so that a report cannot overclaim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proof {
    /// Assembled against the real client and built. Catches a renamed method.
    Compiled,
    /// Resolved against the real client's declared types, without producing an artefact. Catches
    /// a renamed method wherever the client's types reach.
    TypeChecked,
    /// Read by the language's own parser. Catches syntax, and nothing about the client.
    Parsed,
}

impl Proof {
    fn read(word: &str) -> Option<Proof> {
        match word {
            "compiled" => Some(Proof::Compiled),
            "type-checked" => Some(Proof::TypeChecked),
            "parsed" => Some(Proof::Parsed),
            _ => None,
        }
    }

    pub fn word(&self) -> &'static str {
        match self {
            Proof::Compiled => "compiled",
            Proof::TypeChecked => "type-checked",
            Proof::Parsed => "parsed",
        }
    }

    /// The same level said as something an example failed to do.
    pub fn verb(&self) -> &'static str {
        match self {
            Proof::Compiled => "compile",
            Proof::TypeChecked => "type-check",
            Proof::Parsed => "parse",
        }
    }
}

/// What one language declares about its examples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub proof: Proof,
    /// One line saying what this level catches and what it does not, printed with every run so
    /// that the report says what was actually done.
    pub proves: String,
    /// Where an assembled example lands inside the project, relative to its root.
    pub path: String,
    /// Commands run once, in the project root, in order.
    pub run: Vec<Vec<String>>,
    /// Commands run once per assembled example.
    pub each: Vec<Vec<String>>,
    /// What every command of this language is given on top of the environment it inherits.
    pub environment: BTreeMap<String, String>,
    /// The budget one command of this language gets.
    pub timeout: Duration,
}

pub fn read(path: &Path) -> Result<Manifest, Error> {
    let body = read_bounded(path, "manifest", MAX_MANIFEST_BYTES)?;
    parse(&body, &path.display().to_string())
}

pub fn parse(body: &str, path: &str) -> Result<Manifest, Error> {
    let refuse = |detail: String| Error::Manifest {
        path: path.to_owned(),
        cause: detail,
    };

    let document: toml::Table = body.parse().map_err(|cause| refuse(format!("{cause}")))?;

    let word = |key: &str| -> Result<String, Error> {
        document
            .get(key)
            .ok_or_else(|| refuse(format!("no `{key}` key")))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| refuse(format!("`{key}` is not a string")))
    };

    let proof = word("proof")?;
    let proof = Proof::read(&proof).ok_or_else(|| {
        refuse(format!(
            "`proof` is `{proof}`, which is not one of compiled, type-checked or parsed"
        ))
    })?;

    let proves = word("proves")?;
    if proves.trim().is_empty() {
        return Err(refuse(
            "`proves` is empty; it is the sentence the report prints about this level".to_owned(),
        ));
    }

    let path_template = word("path")?;
    if !path_template.contains("{{name}}") && !path_template.contains("{{Name}}") {
        return Err(refuse(
            "`path` carries neither {{name}} nor {{Name}}, so every example of this language \
             would be written over the last one"
                .to_owned(),
        ));
    }

    let run = commands(&document, "run", &refuse)?;
    let each = commands(&document, "each", &refuse)?;
    if run.is_empty() && each.is_empty() {
        return Err(refuse(
            "neither `run` nor `each` names a command, so nothing would prove anything".to_owned(),
        ));
    }
    if run.len() + each.len() > MAX_COMMANDS {
        return Err(refuse(format!(
            "more than {MAX_COMMANDS} commands are declared"
        )));
    }
    for command in &each {
        if !command.iter().any(|word| word.contains("{{file}}")) {
            return Err(refuse(format!(
                "the `each` command `{}` never names {{{{file}}}}, so it would be run once per \
                 example against the same thing every time",
                command.join(" ")
            )));
        }
    }

    let seconds = document
        .get("timeout_seconds")
        .ok_or_else(|| {
            refuse(
                "no `timeout_seconds` key, which is the budget one command of this language gets"
                    .to_owned(),
            )
        })?
        .as_integer()
        .ok_or_else(|| refuse("`timeout_seconds` is not a whole number".to_owned()))?;
    let seconds =
        u64::try_from(seconds).map_err(|_| refuse("`timeout_seconds` is negative".to_owned()))?;
    let timeout = Duration::from_secs(seconds);
    if timeout.is_zero() || timeout > MAX_TIMEOUT {
        return Err(refuse(format!(
            "`timeout_seconds` is {seconds}, outside 1..={}",
            MAX_TIMEOUT.as_secs()
        )));
    }

    let mut environment = BTreeMap::new();
    if let Some(declared) = document.get("environment") {
        let declared = declared
            .as_table()
            .ok_or_else(|| refuse("`environment` is not a table".to_owned()))?;
        if declared.len() > MAX_COMMAND_WORDS {
            return Err(refuse(format!(
                "`environment` carries more than {MAX_COMMAND_WORDS} entries"
            )));
        }
        for (key, value) in declared {
            let value = value
                .as_str()
                .ok_or_else(|| refuse(format!("`environment.{key}` is not a string")))?;
            environment.insert(key.clone(), value.to_owned());
        }
    }

    Ok(Manifest {
        proof,
        proves,
        path: path_template,
        run,
        each,
        environment,
        timeout,
    })
}

/// The commands one key names, each a list of words.
fn commands(
    document: &toml::Table,
    key: &str,
    refuse: &impl Fn(String) -> Error,
) -> Result<Vec<Vec<String>>, Error> {
    let Some(value) = document.get(key) else {
        return Ok(Vec::new());
    };
    let listed = value
        .as_array()
        .ok_or_else(|| refuse(format!("`{key}` is not an array of commands")))?;

    let mut out = Vec::with_capacity(listed.len());
    for entry in listed {
        let words = entry
            .as_array()
            .ok_or_else(|| refuse(format!("`{key}` carries something that is not a command")))?;
        if words.is_empty() {
            return Err(refuse(format!(
                "`{key}` carries a command naming no program"
            )));
        }
        if words.len() > MAX_COMMAND_WORDS {
            return Err(refuse(format!(
                "`{key}` carries a command spelled with more than {MAX_COMMAND_WORDS} words"
            )));
        }
        let mut command = Vec::with_capacity(words.len());
        for word in words {
            command.push(
                word.as_str().map(str::to_owned).ok_or_else(|| {
                    refuse(format!("`{key}` carries something that is not a word"))
                })?,
            );
        }
        out.push(command);
    }
    Ok(out)
}
