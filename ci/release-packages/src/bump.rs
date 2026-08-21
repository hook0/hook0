//! What the commits since the last release demand of the next version number.
//!
//! The bump is an argument to the release scripts, and an argument is whatever somebody typed. The
//! number in the manifest is the last release and nothing more, so adding one to it says nothing
//! about what happened since — a change that breaks the wire goes out as a patch the moment anybody
//! asks for one, and the first thing that notices is somebody's build.
//!
//! What happened since is written down, though: this repository writes Conventional Commits, where a
//! `!` before the colon and a `BREAKING CHANGE:` footer are the two ways a commit says it broke
//! something and `feat` is how it says it added something. So the release can be held to its own
//! history rather than to an argument.
//!
//! Only the floor is held. A bump smaller than the commits demand is refused naming the commits that
//! demand more; a larger one is left alone, because deciding that a release is a major is a decision
//! about what it means to whoever installs it, and no commit log can say that decision is wrong.
//!
//! Every read is bounded — how many commits, how many bytes of history, how many paths, how much of
//! a subject — and a ceiling crossed is a refusal rather than a truncation, since a breaking commit
//! sitting just past the last one read is exactly the one this exists to find.

use std::fmt;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use crate::error::Error;

/// The command the history is read with.
const GIT: &str = "git";

/// The most commits read out of one range. The whole history of this repository fits inside it
/// several times over; a range past this is one where somebody should be raising the ceiling
/// deliberately.
pub const MAX_COMMITS: usize = 4096;

/// The most bytes of history read in one run.
pub const MAX_HISTORY_BYTES: usize = 8 << 20;

/// The most paths one package is read over. One package occupies one directory; a release covering
/// every SDK at once names one apiece.
pub const MAX_PATHS: usize = 64;

/// The most characters of what git said carried into a refusal.
const MAX_SAID_CHARS: usize = 512;

/// The most characters of a subject carried into a report, so that a commit written as an essay is
/// still one line of it.
const MAX_SUBJECT_CHARS: usize = 120;

/// The most characters in front of a subject's colon read as a type and a scope. A conventional
/// header is a word and a parenthesis; anything longer is a sentence that happens to hold a colon.
const MAX_HEADER_CHARS: usize = 100;

/// The most commits named in one report. Past this the count stands in for the rest: a list nobody
/// reads to the end names nothing.
const MAX_NAMED: usize = 20;

/// What one commit is separated from the next by, and one field from the next inside it — the two
/// characters ASCII has for exactly this, and the two a commit message has no way to carry.
const RECORD: char = '\u{1e}';
const FIELD: char = '\u{1f}';

/// The two spellings the specification gives the footer. The phrase is uppercase in both, which is
/// the one part of a conventional commit that is case-sensitive.
const BREAKING: [&str; 2] = ["BREAKING CHANGE", "BREAKING-CHANGE"];

/// How big a release something is, ordered so that "smaller than what is required" is a comparison
/// rather than a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bump {
    Patch,
    Minor,
    Major,
}

impl Bump {
    /// The bump that word names, or nothing if it names none.
    pub fn named(word: &str) -> Option<Bump> {
        match word {
            "patch" => Some(Bump::Patch),
            "minor" => Some(Bump::Minor),
            "major" => Some(Bump::Major),
            _ => None,
        }
    }

    /// What a commit demanding this is called where it is listed.
    fn demand(self) -> &'static str {
        match self {
            Bump::Major => "breaking",
            Bump::Minor => "feature",
            Bump::Patch => "neither",
        }
    }
}

impl fmt::Display for Bump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Bump::Patch => "patch",
            Bump::Minor => "minor",
            Bump::Major => "major",
        })
    }
}

/// Where the commits were read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Since {
    /// The last tag of this package's shape reachable from HEAD, which is its last release.
    Tag(String),
    /// No tag of that shape has ever been pushed, so this is the first release and every commit
    /// that ever touched the package counts towards it.
    FirstCommit { pattern: String },
}

impl fmt::Display for Since {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Since::Tag(tag) => f.write_str(tag),
            Since::FirstCommit { pattern } => {
                write!(f, "the first commit, since no tag matches `{pattern}` yet")
            }
        }
    }
}

/// One commit, and what it demands of the next release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    /// The abbreviated hash, which is what names it in a report.
    pub id: String,
    pub subject: String,
    pub demands: Bump,
}

/// What the commits since the last release demand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Required {
    /// The smallest release they allow.
    pub bump: Bump,
    /// What they were read since.
    pub since: Since,
    /// How many were read, whatever each of them demands.
    pub read: usize,
    /// The ones demanding more than a patch, in the order the history gives them.
    pub reasons: Vec<Commit>,
}

/// The smallest release the commits since the last tag matching `tag_glob` allow, read over `paths`.
///
/// `paths` scopes the read to the package: the tag covers those directories, so a commit touching
/// none of them says nothing about this release. An empty slice reads the whole tree.
pub fn required_bump(tree: &Path, tag_glob: &str, paths: &[&str]) -> Result<Required, Error> {
    if paths.len() > MAX_PATHS {
        return Err(Error::TooManyPaths {
            count: paths.len(),
            ceiling: MAX_PATHS,
        });
    }

    let since = last_release(tree, tag_glob)?;
    let read = history(tree, &since, paths)?;
    let reasons: Vec<Commit> = read
        .iter()
        .filter(|commit| commit.demands > Bump::Patch)
        .cloned()
        .collect();

    Ok(Required {
        bump: reasons
            .iter()
            .map(|commit| commit.demands)
            .max()
            .unwrap_or(Bump::Patch),
        since,
        read: read.len(),
        reasons,
    })
}

/// Refuse a release smaller than the commits demand.
///
/// The refusal names them, because the answer to it is a judgement rather than a bigger number: the
/// commit that demands more is either mismarked, in which case the message is what to fix, or it is
/// marked correctly and the release is bigger than whoever started it thought.
pub fn check_bump(required: &Required, requested: Bump) -> Result<(), Error> {
    if requested >= required.bump {
        return Ok(());
    }
    Err(Error::BumpTooSmall {
        requested,
        required: required.bump,
        since: required.since.clone(),
        commits: naming(&required.reasons, requested).trim_end().to_string(),
    })
}

/// The commits demanding more than `above`, one per line and bounded.
///
/// Ordered by what they demand rather than by when they landed, so that the ones deciding the answer
/// are the ones that survive the ceiling: a list truncated in history order can end up saying `major`
/// with nothing breaking left in it, which is a report that has to be taken on trust. Within a level
/// the history's own order is kept.
pub fn naming(commits: &[Commit], above: Bump) -> String {
    let mut over: Vec<&Commit> = commits
        .iter()
        .filter(|commit| commit.demands > above)
        .collect();
    over.sort_by_key(|commit| std::cmp::Reverse(commit.demands));

    let mut out = String::new();
    for commit in over.iter().take(MAX_NAMED) {
        let _ = writeln!(
            out,
            "  {:8}  {}  {}",
            commit.demands.demand(),
            commit.id,
            commit.subject
        );
    }
    if over.len() > MAX_NAMED {
        let _ = writeln!(out, "  and {} more", over.len() - MAX_NAMED);
    }
    out
}

/// The last release of this package, or the fact that it has never had one.
fn last_release(tree: &Path, glob: &str) -> Result<Since, Error> {
    let described = git(
        tree,
        &["describe", "--tags", "--abbrev=0", "--match", glob, "HEAD"],
    )?;
    if described.ok {
        let tag = described.out.trim();
        if !tag.is_empty() {
            return Ok(Since::Tag(tag.to_string()));
        }
    }

    // `git describe` fails two ways that look the same from here: no tag of this shape has ever been
    // pushed, which is a first release, and one has but sits where HEAD cannot see it, which is a
    // question this cannot answer. Reading every commit that ever touched the package would answer
    // the second with a major nobody asked for, so it is a refusal instead.
    let existing = insisted(tree, &["tag", "--list", glob])?;
    match existing.trim().is_empty() {
        true => Ok(Since::FirstCommit {
            pattern: glob.to_string(),
        }),
        false => Err(Error::NoReachableTag {
            pattern: glob.to_string(),
        }),
    }
}

/// Every commit in the range that touches the paths, each read for what it demands.
fn history(tree: &Path, since: &Since, paths: &[&str]) -> Result<Vec<Commit>, Error> {
    let range = match since {
        Since::Tag(tag) => format!("{tag}..HEAD"),
        Since::FirstCommit { .. } => "HEAD".to_string(),
    };
    // One past the ceiling, so that a range at the ceiling is read and a range past it is known to
    // be past it rather than assumed to end there.
    let ceiling = format!("--max-count={}", MAX_COMMITS + 1);
    let shape = format!("--format={RECORD}%h{FIELD}%s{FIELD}%b");

    let mut arguments = vec!["log", &range, &ceiling, &shape, "--"];
    arguments.extend_from_slice(paths);
    let body = insisted(tree, &arguments)?;

    let mut read = Vec::new();
    for record in body.split(RECORD).skip(1) {
        if read.len() >= MAX_COMMITS {
            return Err(Error::TooManyCommits {
                ceiling: MAX_COMMITS,
                since: since.clone(),
            });
        }
        let mut fields = record.split(FIELD);
        let id = fields.next().unwrap_or_default().trim().to_string();
        let subject = fields.next().unwrap_or_default();
        let message = fields.next().unwrap_or_default();
        read.push(Commit {
            id,
            subject: shortened(subject),
            demands: demanded(subject, message),
        });
    }
    Ok(read)
}

/// What one commit demands, read off its message the way Conventional Commits says to read it.
fn demanded(subject: &str, body: &str) -> Bump {
    let (kind, marked) = header(subject).unwrap_or_default();
    if marked || breaking_footer(body) {
        return Bump::Major;
    }
    match kind == "feat" {
        true => Bump::Minor,
        false => Bump::Patch,
    }
}

/// The type a conventional-commit subject declares, and whether a `!` marks it breaking.
///
/// `type(scope)!: description`, where everything but the type is optional. Nothing else is one: a
/// merge commit, or a sentence that happens to hold a colon, declares no type and demands nothing.
/// The type is compared without case, which is what the specification says to do with every part of
/// a conventional commit except the breaking-change footer.
fn header(subject: &str) -> Option<(String, bool)> {
    let (left, _) = subject.split_once(':')?;
    if left.chars().count() > MAX_HEADER_CHARS {
        return None;
    }

    let (left, marked) = match left.trim_end().strip_suffix('!') {
        Some(without) => (without, true),
        None => (left, false),
    };
    let kind = match left.split_once('(') {
        Some((kind, scope)) => match scope.strip_suffix(')') {
            Some(inner) if !inner.contains(')') => kind,
            _ => return None,
        },
        None => left,
    };

    let kind = kind.trim();
    match !kind.is_empty() && kind.chars().all(|c| c.is_ascii_alphabetic()) {
        true => Some((kind.to_ascii_lowercase(), marked)),
        false => None,
    }
}

/// Whether the body declares a breaking change in a footer.
///
/// A footer is a line that starts with the token. The phrase written anywhere else is prose about a
/// breaking change rather than a declaration of one — "this is not a BREAKING CHANGE: the wire is
/// unchanged" says the opposite of what matching it anywhere would read it as saying.
fn breaking_footer(body: &str) -> bool {
    body.lines().any(|line| {
        BREAKING.iter().any(|token| {
            line.strip_prefix(token)
                .is_some_and(|rest| rest.starts_with(':'))
        })
    })
}

fn shortened(subject: &str) -> String {
    let subject = subject.trim();
    match subject.chars().count() > MAX_SUBJECT_CHARS {
        true => subject
            .chars()
            .take(MAX_SUBJECT_CHARS)
            .chain(['…'])
            .collect(),
        false => subject.to_string(),
    }
}

/// What git answered, whether or not it could do what it was asked.
struct Answer {
    ok: bool,
    out: String,
    said: String,
}

/// Run git in the tree, refusing history past the ceiling rather than carrying it.
fn git(tree: &Path, arguments: &[&str]) -> Result<Answer, Error> {
    let ran = Command::new(GIT)
        .args(arguments)
        .current_dir(tree)
        .output()
        .map_err(|source| Error::Git {
            command: arguments.join(" "),
            reason: source.to_string(),
        })?;

    if ran.stdout.len() > MAX_HISTORY_BYTES {
        return Err(Error::HistoryTooLarge {
            bytes: ran.stdout.len(),
            ceiling: MAX_HISTORY_BYTES,
        });
    }
    Ok(Answer {
        ok: ran.status.success(),
        out: String::from_utf8_lossy(&ran.stdout).into_owned(),
        said: String::from_utf8_lossy(&ran.stderr)
            .trim()
            .chars()
            .take(MAX_SAID_CHARS)
            .collect(),
    })
}

/// The same, where anything but success is a refusal carrying what git said about it.
fn insisted(tree: &Path, arguments: &[&str]) -> Result<String, Error> {
    let answer = git(tree, arguments)?;
    match answer.ok {
        true => Ok(answer.out),
        false => Err(Error::Git {
            command: arguments.join(" "),
            reason: answer.said,
        }),
    }
}
