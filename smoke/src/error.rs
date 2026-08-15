//! Every way this harness gives up, said in a sentence that names what to do about it.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    ReadDirectory {
        path: String,
        cause: io::Error,
    },
    TooManySmokes {
        path: String,
        maximum: usize,
    },
    ReadManifest {
        path: String,
        cause: io::Error,
    },
    ManifestTooLarge {
        path: String,
        maximum: u64,
    },
    ParseManifest {
        path: String,
        cause: String,
    },
    TargetsWithoutSmoke {
        targets: Vec<String>,
        languages: String,
    },
    SmokesWithoutTarget {
        directories: Vec<String>,
    },
    Command {
        program: String,
        cause: io::Error,
    },
    CommandFailed {
        program: String,
        status: String,
        output: String,
    },
    StackNeverHealthy {
        url: String,
        seconds: u64,
        last: String,
    },
    Http {
        what: String,
        cause: String,
    },
    Api {
        what: String,
        status: u16,
        body: String,
    },
    Answer {
        what: String,
        detail: String,
    },
    NoVerificationEmail {
        address: String,
        seconds: u64,
    },
    NoDelivery {
        seconds: u64,
    },
    NoGateway {
        project: String,
    },
    Receiver {
        cause: io::Error,
    },
    MissingSetting {
        name: String,
        why: String,
    },
    SmokesFailed {
        failed: Vec<String>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDirectory { path, cause } => write!(f, "cannot read `{path}`: {cause}"),
            Self::TooManySmokes { path, maximum } => write!(
                f,
                "`{path}` holds more than {maximum} smoke directories; raise the ceiling deliberately"
            ),
            Self::ReadManifest { path, cause } => write!(f, "cannot read `{path}`: {cause}"),
            Self::ManifestTooLarge { path, maximum } => {
                write!(
                    f,
                    "`{path}` is larger than {maximum} bytes, so it is not a manifest"
                )
            }
            Self::ParseManifest { path, cause } => write!(f, "`{path}` is not readable: {cause}"),
            Self::TargetsWithoutSmoke { targets, languages } => write!(
                f,
                "the generator declares {} that no smoke exercises: {}. Add `{languages}/<name>/{}` for each, or the client ships untested against a real instance",
                if targets.len() == 1 {
                    "a target"
                } else {
                    "targets"
                },
                targets.join(", "),
                crate::discovery::MANIFEST,
            ),
            Self::SmokesWithoutTarget { directories } => write!(
                f,
                "these smoke directories name no target the generator declares: {}. Nothing runs them",
                directories.join(", "),
            ),
            Self::Command { program, cause } => write!(f, "cannot run `{program}`: {cause}"),
            Self::CommandFailed {
                program,
                status,
                output,
            } => {
                write!(f, "`{program}` {status}\n{output}")
            }
            Self::StackNeverHealthy { url, seconds, last } => write!(
                f,
                "the API at {url} did not answer within {seconds}s; last attempt: {last}"
            ),
            Self::Http { what, cause } => write!(f, "{what}: {cause}"),
            Self::Api { what, status, body } => {
                write!(f, "{what}: the API answered {status} {body}")
            }
            Self::Answer { what, detail } => write!(f, "{what}: {detail}"),
            Self::NoVerificationEmail { address, seconds } => write!(
                f,
                "no verification email for {address} reached Mailpit within {seconds}s"
            ),
            Self::NoDelivery { seconds } => write!(
                f,
                "the output worker delivered no webhook within {seconds}s, so there is no server-produced signature to verify"
            ),
            Self::NoGateway { project } => write!(
                f,
                "no Compose network is labelled with the project `{project}`, so the address the worker reaches this host on is unknown"
            ),
            Self::Receiver { cause } => write!(f, "the webhook receiver failed: {cause}"),
            Self::MissingSetting { name, why } => write!(f, "{name} is not set, and {why}"),
            Self::SmokesFailed { failed } => {
                write!(f, "{} failed: {}", failed.len(), failed.join(", "))
            }
        }
    }
}

impl std::error::Error for Error {}
