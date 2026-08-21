//! Refuses a file that git tracks while an ignore rule covers it.
//!
//! The two states contradict each other: the ignore rule is somebody writing down that this path
//! does not belong in the repository, and the tracking is the repository carrying it anyway.
//! Whichever of the two is right, they cannot both be, and nothing else in the build notices —
//! ignoring a path has no effect on one already in the index, so the rule reads as though it worked
//! and the file keeps being committed. A build cache reached 841 MB and 119 files that way, present
//! since the target that produced it arrived and churning on every build.
//!
//! It is worth a guard rather than a fix because the fix does not hold: every ecosystem brings a
//! directory it wants left alone, and it is committed once before the rule lands often enough that
//! the next language added will do it again.
//!
//! Nothing here names a path, a directory or a pattern. What is tracked comes from `git ls-files`
//! and what is ignored comes from `git check-ignore`, so a rule written after this file was last
//! read is still the rule that decides.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Most tracked paths examined. Far above what this repository holds, and a repository above it is
/// one this guard reports rather than one it half-checks: a truncated answer here reads as a clean
/// bill of health, which is the one answer a guard must never invent.
const MAX_TRACKED_PATHS: usize = 100_000;

/// Most bytes read back from one git invocation, which bounds what a repository of unexpected shape
/// can cost in memory.
const MAX_GIT_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

/// How many offenders the failure names. The count that follows is the whole truth; printing all of
/// them buries it — the cache that prompted this guard would have printed 119 paths.
const MAX_REPORTED: usize = 10;

/// How many NUL-terminated fields `check-ignore -v -z` writes per match: the file the rule is
/// written in, the line it is on, the rule itself, and the path it caught.
const FIELDS_PER_MATCH: usize = 4;

/// One tracked path, and the ignore rule that contradicts its being tracked.
struct Contradiction {
    source: String,
    line: String,
    pattern: String,
    path: String,
}

/// What `git` said, once it has been checked over.
struct GitOutput {
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: String,
}

/// Runs one git command in `root`, writing `stdin` to it.
///
/// Returns `None` only when there is no `git` to run. Every other failure is raised: a guard that
/// reads "something went wrong" as "nothing is wrong" is one that has stopped guarding.
fn git(root: &Path, arguments: &[&str], stdin: &[u8]) -> Option<GitOutput> {
    let spawned = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => return None,
        Err(failure) => panic!("could not run `git {}`: {failure}", arguments.join(" ")),
    };

    // Written from a thread of its own: a list of every tracked path is larger than a pipe buffer,
    // and a parent that writes it all before reading a word deadlocks against a child doing the
    // same thing in the other direction.
    let mut pipe = child
        .stdin
        .take()
        .expect("the child was given a stdin pipe");
    let payload = stdin.to_vec();
    let writing = std::thread::spawn(move || pipe.write_all(&payload));

    let finished = child.wait_with_output().unwrap_or_else(|failure| {
        panic!("could not read `git {}`: {failure}", arguments.join(" "))
    });

    // A child that stopped reading before the whole list arrived is a child that failed; the exit
    // status below says why, and it says it better than a broken pipe does.
    let _ = writing.join().expect("the thread writing to git finished");

    assert!(
        finished.stdout.len() <= MAX_GIT_OUTPUT_BYTES,
        "`git {}` answered more than the {MAX_GIT_OUTPUT_BYTES} bytes this guard reads",
        arguments.join(" ")
    );

    Some(GitOutput {
        code: finished.status.code(),
        stdout: finished.stdout,
        stderr: String::from_utf8_lossy(&finished.stderr).trim().to_string(),
    })
}

/// The repository this crate sits in, or nothing when it sits in no repository at all.
///
/// Nothing is the answer for a source archive rather than a checkout — this crate is built from one
/// whenever it is vendored — and there the invariant does not apply rather than holding unverified:
/// with no index there is nothing tracked, so there is nothing a tracked file could contradict. The
/// skip is printed, because the failure this guard is written against is a guard nobody notices has
/// stopped running.
fn repository() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let answer = git(manifest, &["rev-parse", "--show-toplevel"], b"")?;

    if answer.code != Some(0) {
        println!("skipped: {} is not in a git repository", manifest.display());
        return None;
    }

    let root = String::from_utf8_lossy(&answer.stdout).trim().to_string();
    assert!(
        !root.is_empty(),
        "git named no repository root for {}",
        manifest.display()
    );
    Some(PathBuf::from(root))
}

/// Every path in the index, which is the set of files a commit would carry.
fn tracked(root: &Path) -> Vec<String> {
    let answer = git(root, &["ls-files", "-z"], b"").expect("git was found a moment ago");
    assert_eq!(
        answer.code,
        Some(0),
        "`git ls-files` failed in {}: {}",
        root.display(),
        answer.stderr
    );

    let paths = answer
        .stdout
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect::<Vec<_>>();

    assert!(
        paths.len() <= MAX_TRACKED_PATHS,
        "{} tracks more than the {MAX_TRACKED_PATHS} paths this guard examines",
        root.display()
    );
    paths
}

/// Which of those paths an ignore rule covers, and which rule covers each.
///
/// `--no-index` is what makes the question answerable at all: without it `git check-ignore` reports
/// nothing for a tracked path, on the grounds that ignore rules do not apply to one — which is true,
/// and is exactly the contradiction being looked for.
///
/// `-v` reports the rule that decided a path, and a rule beginning with `!` decided to re-include
/// it — the path is not ignored, and a file deliberately kept against a broad rule is the one shape
/// of exception git offers. Reading those as offences would make this guard impossible to satisfy:
/// the negation it asks for in its own failure message would leave it failing.
fn contradictions(root: &Path, paths: &[String]) -> Vec<Contradiction> {
    if paths.is_empty() {
        return Vec::new();
    }

    let mut payload = Vec::new();
    for path in paths {
        payload.extend_from_slice(path.as_bytes());
        payload.push(0);
    }

    let answer = git(
        root,
        &["check-ignore", "--no-index", "--stdin", "-z", "-v"],
        &payload,
    )
    .expect("git was found a moment ago");

    // One means it matched nothing, which is the answer this guard hopes for rather than a failure.
    // Anything past one is git declining to answer, and an unanswered question is not a pass.
    match answer.code {
        Some(0) => {}
        Some(1) => return Vec::new(),
        other => panic!(
            "`git check-ignore` failed in {} with status {other:?}: {}",
            root.display(),
            answer.stderr
        ),
    }

    let fields = answer
        .stdout
        .split(|byte| *byte == 0)
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect::<Vec<_>>();
    // The output ends with a terminator, so the split leaves one empty field past the last record.
    let complete = fields.len() / FIELDS_PER_MATCH * FIELDS_PER_MATCH;

    let (records, _) = fields[..complete].as_chunks::<FIELDS_PER_MATCH>();
    let mut found = records
        .iter()
        // A pattern written `\!` is a file whose name begins with one, and it is still an exclusion.
        .filter(|record| !record[2].starts_with('!'))
        .map(|record| Contradiction {
            source: record[0].clone(),
            line: record[1].clone(),
            pattern: record[2].clone(),
            path: record[3].clone(),
        })
        .collect::<Vec<_>>();

    found.sort_by(|left, right| left.path.cmp(&right.path));
    found
}

#[test]
fn no_tracked_file_is_covered_by_an_ignore_rule() {
    let Some(root) = repository() else {
        return;
    };

    let paths = tracked(&root);
    assert!(
        !paths.is_empty(),
        "{} tracks no files at all — this guard is looking in the wrong place",
        root.display()
    );

    let found = contradictions(&root, &paths);
    if found.is_empty() {
        return;
    }

    let mut report = found
        .iter()
        .take(MAX_REPORTED)
        .map(|one| {
            format!(
                "  {}\n      ignored by {}:{}  `{}`",
                one.path, one.source, one.line, one.pattern
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if found.len() > MAX_REPORTED {
        report.push_str(&format!("\n  ... and {} more", found.len() - MAX_REPORTED));
    }

    let counted = match found.len() {
        1 => "One file is".to_string(),
        many => format!("{many} files are"),
    };

    panic!(
        "{counted} tracked by git and covered by an ignore rule. Ignoring a path does nothing to \
         one already in the index, so the rule below reads as though it worked while the file keeps \
         being committed:\n\n{report}\n\nResolve each one in whichever direction is meant, since \
         only one of the two can be right:\n\n  \
         git rm --cached -- <path>   stop tracking it; the file stays on disk\n  \
         or drop the rule, or add a `!<path>` line under it, when the file is meant to be committed"
    );
}
