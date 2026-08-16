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
    Composition {
        path: String,
        detail: String,
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
        expectation: String,
        worker_said: String,
    },
    ReceiverUnreachable {
        from: String,
        url: String,
        said: String,
        remedy: String,
    },
    StackDied {
        what: String,
        status: String,
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
    RequirementUnmet {
        target: String,
        program: String,
        said: String,
        remedy: String,
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
            Self::Composition { path, detail } => write!(
                f,
                "`{path}` is what both ways of bringing the stack up are configured from, and it is {detail}"
            ),
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
            Self::NoDelivery {
                seconds,
                expectation,
                worker_said,
            } => write!(
                f,
                "waited {seconds}s for the output worker to deliver one webhook to the subscription \
                 just created, and none arrived, so there is no server-produced signature for any \
                 client to verify.\n  The receiver was proved reachable from inside the stack before \
                 provisioning, so this is not the network between them.\n  {expectation}.\n  What the \
                 output worker said while it was waited for:\n{worker_said}"
            ),
            Self::ReceiverUnreachable {
                from,
                url,
                said,
                remedy,
            } => write!(
                f,
                "the webhook receiver at {url} cannot be reached from {from}: {said}.\n  That is the \
                 path every delivery takes, so nothing provisioned after this could ever be \
                 delivered and no client would have a server-produced signature to verify. The \
                 stack is not at fault and neither is the harness: something between them is \
                 dropping what the containers send, which on Linux is almost always the host \
                 firewall.\n  {remedy}"
            ),
            Self::StackDied { what, status } => write!(
                f,
                "`{what}` {status} after it was started; what it said is above this line"
            ),
            Self::NoGateway { project } => write!(
                f,
                "no Compose network is labelled with the project `{project}`, so the address the worker reaches this host on is unknown"
            ),
            Self::Receiver { cause } => write!(f, "the webhook receiver failed: {cause}"),
            Self::MissingSetting { name, why } => write!(f, "{name} is not set, and {why}"),
            Self::RequirementUnmet {
                target,
                program,
                said,
                remedy,
            } => write!(
                f,
                "the {target} smoke needs `{program}` to answer before it can run, and it did not: \
                 {said}\n  That command is how this harness settles where {target}'s packages are \
                 and whether they are installed, neither of which the runtime works out on its \
                 own. It is asked rather than read out of the environment on purpose: a run that \
                 depended on whoever started it having exported the right search path would pass \
                 here and fail everywhere else.\n  {remedy}"
            ),
            Self::SmokesFailed { failed } => {
                write!(f, "{} failed: {}", failed.len(), failed.join(", "))
            }
        }
    }
}

impl std::error::Error for Error {}
