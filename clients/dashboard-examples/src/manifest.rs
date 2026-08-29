//! What the dashboard cannot read off an example itself.
//!
//! The dashboard renders a snippet by substituting strings and nothing else: it knows no language,
//! and must not learn one. Everything about a language that a substitution cannot express is
//! declared in a `dashboard.toml` beside the code it describes, so that the day one of them is the
//! language somebody is reading there is still only one place to look.

use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::limits::{
    MAX_ALSO_NEEDS_CHARS, MAX_DELIMITER_CHARS, MAX_DISPLAY_NAME_CHARS, MAX_ESCAPE_CHARS,
    MAX_ESCAPE_RULES, MAX_MANIFEST_BYTES, MAX_PROVES_CHARS, MAX_REACH_CHARS, MAX_REACH_LINES,
    MAX_SEPARATOR_CHARS, MAX_SOURCE_CHARS, MAX_USAGE_SHARE, MIN_USAGE_SHARE,
};

/// The file each language declares itself in.
pub const FILE: &str = "dashboard.toml";

/// The file whose configuration puts this language's examples under the job that proves them, and
/// the lines in it that do.
const NAMED_IN: &str = "examples_named_in";
const NAMED_BY: &str = "examples_named_by";

/// What reads them without naming them, for the languages where nothing does.
const SWEPT_BY: &str = "examples_swept_by";

/// What a reader installs or wires beyond the package before the snippet beside it builds.
const ALSO_NEEDS: &str = "snippet_also_needs";

/// How far the job carrying a client goes towards proving its examples.
///
/// The vocabulary is the repository's own, and so is the rule that comes with it: the claim a
/// report prints must not overstate what the command behind it does. It exists because the
/// assumption that a renamed method fails every client's job was measured and found false — `ruff`,
/// `rubocop` and `phpcs` parse and lint without resolving a single symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proof {
    /// Built against the real client, so a renamed method or a changed signature stops the job.
    Compiled,
    /// Every name resolved against the real client without an artefact being produced.
    TypeChecked,
    /// Read as the language, which catches a syntax error and nothing about the client.
    Parsed,
}

impl Proof {
    /// How it is written in a manifest, which is also what the dashboard shows.
    pub fn declared(self) -> &'static str {
        match self {
            Proof::Compiled => "compiled",
            Proof::TypeChecked => "type-checked",
            Proof::Parsed => "parsed",
        }
    }

    /// The level a manifest names, or nothing when it names something else.
    ///
    /// Three levels and no more: a fourth would be a claim nobody has defined, and the point of the
    /// field is that what is claimed is one of a closed set somebody agreed on.
    pub fn of(declared: &str) -> Option<Proof> {
        [Proof::Compiled, Proof::TypeChecked, Proof::Parsed]
            .into_iter()
            .find(|level| level.declared() == declared)
    }

    /// Every level there is, for a refusal to name what it would have accepted.
    pub fn accepted() -> String {
        [Proof::Compiled, Proof::TypeChecked, Proof::Parsed]
            .map(|level| format!("`{}`", level.declared()))
            .join(", ")
    }
}

/// What puts a language's examples under the job that proves them.
///
/// The level above rests on something, and which of the two shapes it is decides whether there is
/// anything to hold. A line of build configuration can be deleted while every example file stays
/// exactly where it was: the job stops reading them, the manifest goes on claiming `compiled`, and
/// the guard for a *missing* example has nothing to say about one that is merely no longer read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reach {
    /// A file names the directory: this is that file, and the lines in it that do the naming.
    Named { file: String, lines: Vec<String> },
    /// Nothing names it. The command reads a tree and the examples happen to be in it, so there is
    /// no line to delete and no line to hold; this sentence is what stands in its place.
    Swept { by: String },
}

/// How a value somebody typed becomes a literal of one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLiteral {
    pub open: String,
    pub close: String,
    /// Applied in order, which is why the backslash comes first: escaping it afterwards would
    /// escape the backslashes the other rules had just introduced.
    pub escape: Vec<(String, String)>,
}

impl StringLiteral {
    /// The value, as this language would write it inside a string.
    pub fn escaped(&self, value: &str) -> String {
        self.escape
            .iter()
            .fold(value.to_owned(), |carried, (from, to)| {
                carried.replace(from.as_str(), to)
            })
    }
}

/// One language, as the dashboard needs it and as nothing else can say it.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    /// How the language is named on screen. Neither the target's name nor the package's carries it:
    /// the target is `csharp`, the package is `Hook0.Client`, and the screen says `C#`.
    pub display_name: String,
    /// What share of developers use it, which is the order the languages are offered in. A datum
    /// rather than an opinion: it is declared beside its source, in the file it describes.
    pub usage_share: f64,
    /// What is written above that share, which is the survey it was read off.
    ///
    /// Read out of the comment rather than out of a key, because that is where it is written and a
    /// guard describes the tree rather than the other way round. It is compared across every
    /// language: one survey replaces the last in one go, so a manifest whose source reads
    /// differently is a manifest somebody forgot on the day the figures moved.
    pub usage_source: String,
    /// How far the job carrying this client goes towards proving its examples.
    pub proof: Proof,
    /// The command and the job that level rests on, in prose.
    ///
    /// Read, bounded, and then let go: what it says is for whoever reads the manifest, and that it
    /// says anything at all is what this checks. A level on its own reads as a label; the sentence
    /// beside it is what makes the label answerable.
    pub proves: String,
    /// What puts these examples under that job, which is the half of the claim that can be held
    /// rather than only read.
    pub reach: Reach,
    /// What a reader installs or wires beyond the package itself before the snippet beside this
    /// manifest builds, for the languages where installing the package does not get them there.
    ///
    /// Declared rather than derived, because what is missing is a fact about this client — that its
    /// sending API is async and has no blocking form, that its module has to be handed to the build
    /// — and the install command is a function of the registry serving the package, which knows
    /// none of that. Most languages say nothing here and have nothing appended.
    pub snippet_also_needs: Option<String>,
    /// What joins two rendered labels. It carries the separator; the region does not.
    pub label_separator: String,
    pub string: StringLiteral,
}

/// Read the manifest at `path`.
pub fn read(path: &Path) -> Result<Manifest, Error> {
    let shown = path.display().to_string();
    let body = read_bounded(path, "manifest", MAX_MANIFEST_BYTES)?;
    let document: toml::Value =
        toml::from_str(&body).map_err(|cause| Error::UnreadableManifest {
            path: shown.clone(),
            reason: cause.to_string(),
        })?;

    let display_name = text(&shown, &document, "display_name", MAX_DISPLAY_NAME_CHARS)?;
    let usage_share = share(&shown, &document)?;
    let usage_source = source(&shown, &body)?;
    let label_separator = text(&shown, &document, "label_separator", MAX_SEPARATOR_CHARS)?;
    let proves = text(&shown, &document, "proves", MAX_PROVES_CHARS)?;

    let declared = text(&shown, &document, "proof", MAX_DISPLAY_NAME_CHARS)?;
    let proof = Proof::of(&declared).ok_or_else(|| Error::UnknownProof {
        path: shown.clone(),
        declared,
        accepted: Proof::accepted(),
    })?;

    let literal = document.get("string").ok_or(Error::MissingField {
        path: shown.clone(),
        field: "the string table",
    })?;
    let string = StringLiteral {
        open: text(&shown, literal, "open", MAX_DELIMITER_CHARS)?,
        close: text(&shown, literal, "close", MAX_DELIMITER_CHARS)?,
        escape: escape(&shown, literal)?,
    };

    Ok(Manifest {
        display_name,
        usage_share,
        usage_source,
        proof,
        proves,
        reach: reach(&shown, &document)?,
        snippet_also_needs: also_needs(&shown, &document)?,
        label_separator,
        string,
    })
}

/// What else this language says a reader needs, when it says anything at all.
///
/// Saying nothing is the ordinary answer and means the package carries the whole of what the
/// snippet uses. Saying it and naming nothing is refused by the reader below, since a language
/// declaring there is more and then leaving it blank has answered worse than one that stayed quiet.
///
/// No word the dashboard substitutes survives here. The screen renders the install block through
/// the same table as the two snippets, so a marker in it is either replaced by whatever the reader
/// typed into the form — their payload landing in a command they were told to run — or replaced by
/// nothing at all and copied out as it stands.
fn also_needs(path: &str, document: &toml::Value) -> Result<Option<String>, Error> {
    if document.get(ALSO_NEEDS).is_none() {
        return Ok(None);
    }

    let declared = text(path, document, ALSO_NEEDS, MAX_ALSO_NEEDS_CHARS)?;
    match crate::written_markers(&declared).into_iter().next() {
        Some(marker) => Err(Error::MarkerInAlsoNeeds {
            path: path.to_owned(),
            field: ALSO_NEEDS,
            marker,
        }),
        None => Ok(Some(declared)),
    }
}

/// What the manifest says puts its examples under the job, which is one of the two shapes and not
/// both.
///
/// Both at once is refused rather than resolved: a language declaring a line to hold *and* a
/// command that names no directory has said nothing about which of them the level rests on, and
/// picking one here would decide it on the language's behalf.
fn reach(path: &str, document: &toml::Value) -> Result<Reach, Error> {
    let unsaid = || Error::ExamplesReachUnsaid {
        path: path.to_owned(),
        named: NAMED_IN,
        swept: SWEPT_BY,
    };

    match (
        document.get(NAMED_IN).is_some(),
        document.get(SWEPT_BY).is_some(),
    ) {
        (true, true) => Err(Error::ExamplesReachSaidTwice {
            path: path.to_owned(),
            named: NAMED_IN,
            swept: SWEPT_BY,
        }),
        (false, false) => Err(unsaid()),
        (true, false) => Ok(Reach::Named {
            file: text(path, document, NAMED_IN, MAX_REACH_CHARS)?,
            lines: named_by(path, document)?,
        }),
        (false, true) => Ok(Reach::Swept {
            by: text(path, document, SWEPT_BY, MAX_REACH_CHARS)?,
        }),
    }
}

/// The lines of that file which put the examples under the job.
///
/// A list rather than one line, because one naming is not always one line: Rust declares a target
/// per example, and holding only the first would leave the second deletable in silence.
fn named_by(path: &str, document: &toml::Value) -> Result<Vec<String>, Error> {
    let declared = document
        .get(NAMED_BY)
        .and_then(toml::Value::as_array)
        .filter(|lines| !lines.is_empty())
        .ok_or(Error::MissingField {
            path: path.to_owned(),
            field: NAMED_BY,
        })?;
    if declared.len() > MAX_REACH_LINES {
        return Err(Error::FieldTooLong {
            path: path.to_owned(),
            field: NAMED_BY,
            unit: "entry",
            length: declared.len(),
            ceiling: MAX_REACH_LINES,
        });
    }

    let mut lines = Vec::with_capacity(declared.len());
    for line in declared {
        let value = line
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or(Error::MissingField {
                path: path.to_owned(),
                field: NAMED_BY,
            })?;
        lines.push(bounded(path, NAMED_BY, value, MAX_REACH_CHARS)?);
    }
    Ok(lines)
}

/// The file, refused rather than truncated when it is above the ceiling.
pub fn read_bounded(path: &Path, what: &'static str, maximum: u64) -> Result<String, Error> {
    let size = fs::metadata(path)
        .map_err(|cause| Error::ReadFile {
            path: path.display().to_string(),
            cause,
        })?
        .len();
    if size > maximum {
        return Err(Error::FileTooLarge {
            path: path.display().to_string(),
            what,
            size,
            maximum,
        });
    }
    fs::read_to_string(path).map_err(|cause| Error::ReadFile {
        path: path.display().to_string(),
        cause,
    })
}

/// One declared string, held to a ceiling and refused when it says nothing.
fn text(
    path: &str,
    document: &toml::Value,
    field: &'static str,
    ceiling: usize,
) -> Result<String, Error> {
    let value = document
        .get(field)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(Error::MissingField {
            path: path.to_owned(),
            field,
        })?;
    bounded(path, field, value, ceiling)
}

/// What is written above the share, which is the survey it was read off.
///
/// The contiguous run of comment lines directly above the key, taken as it stands. A share declared
/// with nothing above it is refused: a figure off one survey is only a figure while the survey is
/// named beside it.
fn source(path: &str, body: &str) -> Result<String, Error> {
    let lines: Vec<&str> = body.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.starts_with("usage_share"))
        .ok_or(Error::MissingField {
            path: path.to_owned(),
            field: "usage_share",
        })?;

    let mut from = at;
    while from > 0 && lines[from - 1].trim_start().starts_with('#') {
        from -= 1;
    }
    let declared = lines[from..at].join("\n");
    match declared.trim().is_empty() {
        true => Err(Error::UsageShareWithoutSource {
            path: path.to_owned(),
        }),
        false => bounded(
            path,
            "the source above `usage_share`",
            &declared,
            MAX_SOURCE_CHARS,
        ),
    }
}

/// The share of developers using this language, held to what a share can be.
fn share(path: &str, document: &toml::Value) -> Result<f64, Error> {
    let declared = document.get("usage_share").ok_or(Error::MissingField {
        path: path.to_owned(),
        field: "usage_share",
    })?;
    // TOML reads `66` as an integer and `66.0` as a float, and a round figure among them is the one
    // most likely to be written the first way.
    if let Some(whole) = declared.as_integer() {
        return Err(Error::UsageShareNotAFloat {
            path: path.to_owned(),
            value: whole,
        });
    }
    let value = declared.as_float().ok_or(Error::MissingField {
        path: path.to_owned(),
        field: "usage_share",
    })?;
    match value.is_finite() && (MIN_USAGE_SHARE..=MAX_USAGE_SHARE).contains(&value) {
        true => Ok(value),
        false => Err(Error::UsageShareOutOfRange {
            path: path.to_owned(),
            value,
            minimum: MIN_USAGE_SHARE,
            maximum: MAX_USAGE_SHARE,
        }),
    }
}

/// The replacements that make a typed value safe to put inside a literal, in the order they run.
///
/// The order is the whole of what makes them correct, so it is checked rather than trusted: a rule
/// introducing a backslash while none escapes a backslash first means every later rule escapes what
/// the earlier ones had just written.
fn escape(path: &str, document: &toml::Value) -> Result<Vec<(String, String)>, Error> {
    let declared = document
        .get("escape")
        .and_then(toml::Value::as_array)
        .filter(|rules| !rules.is_empty())
        .ok_or(Error::MissingField {
            path: path.to_owned(),
            field: "escape",
        })?;
    if declared.len() > MAX_ESCAPE_RULES {
        return Err(Error::FieldTooLong {
            path: path.to_owned(),
            field: "escape",
            unit: "entry",
            length: declared.len(),
            ceiling: MAX_ESCAPE_RULES,
        });
    }

    let mut rules = Vec::with_capacity(declared.len());
    for (at, rule) in declared.iter().enumerate() {
        let pair: Vec<&str> = rule
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .filter_map(toml::Value::as_str)
            .collect();
        let [from, to] = pair[..] else {
            return Err(Error::EscapeNotAPair {
                path: path.to_owned(),
                at,
                found: pair.len(),
            });
        };
        if from.is_empty() {
            return Err(Error::EscapeReplacesNothing {
                path: path.to_owned(),
                at,
            });
        }
        rules.push((
            bounded(path, "escape", from, MAX_ESCAPE_CHARS)?,
            bounded(path, "escape", to, MAX_ESCAPE_CHARS)?,
        ));
    }

    let introduces_a_backslash = rules.iter().any(|(_, to)| to.contains('\\'));
    let escapes_one_first = rules.first().is_some_and(|(from, _)| from == "\\");
    match introduces_a_backslash && !escapes_one_first {
        true => Err(Error::BackslashNotEscapedFirst {
            path: path.to_owned(),
        }),
        false => Ok(rules),
    }
}

/// A value read out of a file this crate does not write, held to a ceiling before it is carried.
fn bounded(path: &str, field: &'static str, value: &str, ceiling: usize) -> Result<String, Error> {
    let length = value.chars().count();
    match length > ceiling {
        true => Err(Error::FieldTooLong {
            path: path.to_owned(),
            field,
            unit: "character",
            length,
            ceiling,
        }),
        false => Ok(value.to_owned()),
    }
}
