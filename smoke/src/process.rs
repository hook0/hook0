//! Running something else, always under a deadline.
//!
//! Nothing here waits on a child indefinitely. A compiler that hangs, a stack that never settles
//! and a smoke that loops are all the same failure from the outside — a run that will not end —
//! and each of them ends the same way: the child is killed and the deadline is reported as the
//! reason, rather than the run holding whoever started it.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::Error;

/// How often a running child is checked against its deadline.
const POLL: Duration = Duration::from_millis(100);

/// How much of one line is held while it is being assembled out of the chunks it arrives in.
///
/// The lines worth keeping are three words long. A child that writes megabytes without ever ending
/// a line would otherwise grow this buffer for as long as it ran, so what is past the ceiling is
/// dropped from the *kept* copy — never from what is written through, which is every byte.
const MAX_LINE_BYTES: usize = 4 * 1024;

/// How much is read off a pipe at a time. Small enough that what a child says appears while it is
/// saying it, rather than a screenful at a time.
const CHUNK_BYTES: usize = 8 * 1024;

/// Which line of a child's output is worth keeping, and how many of them at most.
///
/// A stream is written through as it arrives whatever this says; what it decides is only what is
/// kept for the caller to read afterwards. Keeping a predicate's worth rather than the whole
/// stream is what bounds this without ever silently losing the part that matters: `most` is a
/// ceiling on lines that were selected, and one more than it is kept so that crossing it can be
/// seen rather than guessed at.
#[derive(Clone, Copy)]
pub struct Keep {
    /// Whether this line is one of the ones worth keeping.
    pub worth: fn(&str) -> bool,
    /// The most of them kept before the rest are dropped.
    pub most: usize,
}

/// What was said and how it ended.
pub struct Ended {
    pub ok: bool,
    pub status: String,
    pub output: String,
    /// The lines a [`Keep`] selected as they streamed past, at most one more than it allows.
    pub kept: Vec<String>,
}

/// Runs a command to completion under `within`, keeping what it said.
pub fn capture(program: &str, args: &[&str], within: Duration) -> Result<Ended, Error> {
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|cause| Error::Command {
            program: program.to_owned(),
            cause,
        })?;

    let ended = wait(child, program, within, Reading::Captured)?;
    Ok(ended)
}

/// Runs a command to completion under `within`, with what it says going where this harness's own
/// output goes *and* the lines `keep` selects held for the caller.
///
/// Both, rather than either. A smoke's failure is worth reading as it happens, which is why this
/// used to simply inherit; and the run has to read what the smoke reported, which inheriting makes
/// impossible. So each pipe is written through byte for byte the moment it is read, and the lines
/// worth keeping are picked out of the same pass.
pub fn stream(
    program: &str,
    args: &[String],
    directory: &Path,
    environment: &[(String, String)],
    within: Duration,
    keep: Keep,
) -> Result<Ended, Error> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in environment {
        command.env(name, value);
    }

    let child = command.spawn().map_err(|cause| Error::Command {
        program: program.to_owned(),
        cause,
    })?;

    wait(child, program, within, Reading::Teed(keep))
}

/// What is done with a child's pipes while it runs.
enum Reading {
    /// Read and held for the caller, and shown nowhere.
    Captured,
    /// Written through to this harness's own output as it arrives, with the lines worth keeping
    /// picked out of the same pass.
    Teed(Keep),
}

/// The only place a child is waited on, and it is waited on under a deadline.
fn wait(
    mut child: Child,
    program: &str,
    within: Duration,
    reading: Reading,
) -> Result<Ended, Error> {
    // Read the pipes in threads rather than after the wait: a child that fills a pipe buffer
    // blocks until somebody drains it, and a deadline that only fires between polls would never
    // be reached. That is as true of the teed pipes as of the captured ones — a smoke printing
    // more than a pipe buffer holds would otherwise stop and never reach its own deadline.
    let (stdout, stderr) = match &reading {
        Reading::Captured => (
            child
                .stdout
                .take()
                .map(|pipe| drain(pipe, Through::Nowhere, None)),
            child
                .stderr
                .take()
                .map(|pipe| drain(pipe, Through::Nowhere, None)),
        ),
        Reading::Teed(keep) => (
            child
                .stdout
                .take()
                .map(|pipe| drain(pipe, Through::Out, Some(*keep))),
            child
                .stderr
                .take()
                .map(|pipe| drain(pipe, Through::Err, Some(*keep))),
        ),
    };

    let deadline = Instant::now() + within;
    let status = loop {
        match child.try_wait().map_err(|cause| Error::Command {
            program: program.to_owned(),
            cause,
        })? {
            Some(status) => break Some(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            None => thread::sleep(POLL),
        }
    };

    let mut said = String::new();
    let mut kept = Vec::new();
    for reading in [stdout, stderr].into_iter().flatten() {
        let read = reading.join().unwrap_or_default();
        said.push_str(&read.said);
        kept.extend(read.kept);
    }

    match status {
        Some(status) => Ok(Ended {
            ok: status.success(),
            status: match status.code() {
                Some(code) => format!("exited with {code}"),
                None => "was killed by a signal".to_owned(),
            },
            output: said,
            kept,
        }),
        None => Ok(Ended {
            ok: false,
            status: format!("did not finish within {}s and was killed", within.as_secs()),
            output: said,
            kept,
        }),
    }
}

/// Where a pipe is written through to as it is read.
#[derive(Clone, Copy)]
enum Through {
    /// Nowhere, with the whole stream held for the caller instead.
    Nowhere,
    Out,
    Err,
}

impl Through {
    /// Writes what just arrived where it belongs, before anything is done with it.
    ///
    /// Flushed every time on purpose: what makes streaming worth doing is that a smoke's last words
    /// are on screen when it hangs, and a buffered writer's are not.
    fn wrote(self, chunk: &[u8]) {
        let written = match self {
            Self::Nowhere => return,
            Self::Out => std::io::stdout()
                .write_all(chunk)
                .and_then(|()| std::io::stdout().flush()),
            Self::Err => std::io::stderr()
                .write_all(chunk)
                .and_then(|()| std::io::stderr().flush()),
        };
        // Nothing to do about a console that will not take what it is given, and refusing the run
        // over it would report the wrong failure.
        let _ = written;
    }
}

/// One pipe, read to its end.
#[derive(Default)]
struct Drained {
    /// Everything it said, when nothing was written through.
    said: String,
    /// The lines worth keeping, at most one past the ceiling so that crossing it can be seen.
    kept: Vec<String>,
}

/// Reads one pipe to its end, in a thread of its own, writing it through as it arrives.
fn drain<R: Read + Send + 'static>(
    mut pipe: R,
    through: Through,
    keep: Option<Keep>,
) -> thread::JoinHandle<Drained> {
    thread::spawn(move || {
        let mut read = Drained::default();
        let mut whole: Vec<u8> = Vec::new();
        let mut line: Vec<u8> = Vec::new();
        let mut buffer = [0u8; CHUNK_BYTES];

        loop {
            let arrived = match pipe.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => count,
            };
            let chunk = &buffer[..arrived];
            through.wrote(chunk);

            match keep {
                None => whole.extend_from_slice(chunk),
                Some(keep) => {
                    for byte in chunk {
                        if *byte == b'\n' {
                            consider(&line, keep, &mut read.kept);
                            line.clear();
                        } else if line.len() < MAX_LINE_BYTES {
                            line.push(*byte);
                        }
                    }
                }
            }
        }

        // A last line the child never ended, which is what a report printed without a newline
        // before the process exited would arrive as.
        if let Some(keep) = keep {
            consider(&line, keep, &mut read.kept);
        }
        read.said = String::from_utf8_lossy(&whole).into_owned();
        read
    })
}

/// Keeps one line if it is worth keeping and there is room for it.
///
/// One line past the ceiling is kept so that crossing it is something the caller can see rather
/// than a set that quietly stopped growing.
fn consider(line: &[u8], keep: Keep, kept: &mut Vec<String>) {
    if line.is_empty() || kept.len() > keep.most {
        return;
    }
    let read = String::from_utf8_lossy(line);
    let read = read.trim_end_matches('\r');
    if (keep.worth)(read) {
        kept.push(read.to_owned());
    }
}

/// A command that must have succeeded, with what it said.
pub fn must(program: &str, args: &[&str], within: Duration) -> Result<String, Error> {
    let ended = capture(program, args, within)?;
    if !ended.ok {
        return Err(Error::CommandFailed {
            program: format!("{program} {}", args.join(" ")),
            status: ended.status,
            output: ended.output,
        });
    }
    Ok(ended.output)
}

/// Runs something whose failure changes nothing, which is what tearing down is: the stack is going
/// away either way, and a teardown that refused would hide the failure that made it necessary.
pub fn regardless(program: &str, args: &[&str], within: Duration) -> String {
    match capture(program, args, within) {
        Ok(ended) => ended.output,
        Err(refused) => format!("{refused}"),
    }
}
