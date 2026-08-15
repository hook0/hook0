//! Running something else, always under a deadline.
//!
//! Nothing here waits on a child indefinitely. A compiler that hangs, a stack that never settles
//! and a smoke that loops are all the same failure from the outside — a run that will not end —
//! and each of them ends the same way: the child is killed and the deadline is reported as the
//! reason, rather than the run holding whoever started it.

use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::Error;

/// How often a running child is checked against its deadline.
const POLL: Duration = Duration::from_millis(100);

/// What was said and how it ended.
pub struct Ended {
    pub ok: bool,
    pub status: String,
    pub output: String,
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

    let ended = wait(child, program, within, true)?;
    Ok(ended)
}

/// Runs a command to completion under `within`, with what it says going where this harness's own
/// output goes: a smoke's failure is worth reading as it happens rather than after the fact.
pub fn stream(
    program: &str,
    args: &[String],
    directory: &Path,
    environment: &[(String, String)],
    within: Duration,
) -> Result<Ended, Error> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for (name, value) in environment {
        command.env(name, value);
    }

    let child = command.spawn().map_err(|cause| Error::Command {
        program: program.to_owned(),
        cause,
    })?;

    wait(child, program, within, false)
}

/// The only place a child is waited on, and it is waited on under a deadline.
fn wait(mut child: Child, program: &str, within: Duration, piped: bool) -> Result<Ended, Error> {
    // Read the pipes in threads rather than after the wait: a child that fills a pipe buffer
    // blocks until somebody drains it, and a deadline that only fires between polls would never
    // be reached.
    let stdout = child.stdout.take().map(drain);
    let stderr = child.stderr.take().map(drain);

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
    if piped {
        if let Some(reading) = stdout {
            said.push_str(&reading.join().unwrap_or_default());
        }
        if let Some(reading) = stderr {
            said.push_str(&reading.join().unwrap_or_default());
        }
    }

    match status {
        Some(status) => Ok(Ended {
            ok: status.success(),
            status: match status.code() {
                Some(code) => format!("exited with {code}"),
                None => "was killed by a signal".to_owned(),
            },
            output: said,
        }),
        None => Ok(Ended {
            ok: false,
            status: format!("did not finish within {}s and was killed", within.as_secs()),
            output: said,
        }),
    }
}

/// Reads one pipe to its end, in a thread of its own.
fn drain<R: Read + Send + 'static>(mut pipe: R) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut read = Vec::new();
        let _ = pipe.read_to_end(&mut read);
        String::from_utf8_lossy(&read).into_owned()
    })
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
