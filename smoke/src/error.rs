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
    NoAttempt {
        application: String,
        seconds: u64,
        expectation: String,
    },
    NoCounts {
        organization: String,
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
    Document {
        path: String,
        detail: String,
    },
    TooManyReports {
        target: String,
        maximum: usize,
    },
    Unreportable {
        target: String,
        line: String,
        detail: String,
    },
    SurfaceSilent {
        target: String,
    },
    SurfaceUnannounced {
        target: String,
        reports: usize,
    },
    SurfaceAmbiguous {
        target: String,
        operation: String,
        first: String,
        second: String,
    },
    SurfaceUnknown {
        target: String,
        unknown: Vec<String>,
    },
    SurfaceThrottled {
        target: String,
        throttled: Vec<String>,
    },
    SurfaceMissing {
        target: String,
        missing: Vec<String>,
        declared: usize,
    },
    ModelsUnknown {
        target: String,
        unknown: Vec<String>,
    },
    ModelsMissing {
        target: String,
        missing: Vec<String>,
        answered: usize,
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
            Self::NoAttempt {
                application,
                seconds,
                expectation,
            } => write!(
                f,
                "waited {seconds}s for the output worker to finish one delivery attempt in application {application}, and it never did, so there is no response for a client to read back — and no client can decode the type the API answers one with.\n  {expectation}"
            ),
            Self::NoCounts {
                organization,
                seconds,
            } => write!(
                f,
                "waited {seconds}s for the instance to refresh the per-day event counts of organization {organization}, and they stayed empty, so the operations that read them answer a list with nothing in it and no client can decode the type they are answered with.\n  Those counts are a materialized view the API refreshes on a cycle of its own; a run that never sees one is a run whose API is not doing its housekeeping"
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
            Self::Document { path, detail } => write!(
                f,
                "`{path}` is what every client is generated from and what every smoke is held to, and {detail}"
            ),
            Self::TooManyReports { target, maximum } => write!(
                f,
                "the {target} smoke reported more than {maximum} operations, which is more than the API declares under any reading; it is printing `{}` in a loop rather than driving a surface",
                crate::surface::PREFIX,
            ),
            Self::Unreportable {
                target,
                line,
                detail,
            } => write!(
                f,
                "the {target} smoke printed `{line}`, and {detail}. A report is `{} <operationId> {}` or `{} <operationId> {}<problemId>`, and nothing else",
                crate::surface::PREFIX,
                crate::surface::ACCEPTED,
                crate::surface::PREFIX,
                crate::surface::REFUSED,
            ),
            Self::SurfaceSilent { target } => write!(
                f,
                "the {target} smoke drove no operation and its `{}` says nothing about that. Either it does not drive the generated surface yet — say so with `{} = false` in `languages/{target}/{}` — or its reports are not reaching the output this harness reads",
                crate::discovery::MANIFEST,
                crate::discovery::DRIVES_SURFACE,
                crate::discovery::MANIFEST,
            ),
            Self::SurfaceUnannounced { target, reports } => write!(
                f,
                "`languages/{target}/{}` says `{} = false`, and the smoke reported {reports} {}. Delete that line: with it gone the run holds {target} to every operation the API document declares, which is what those reports are for",
                crate::discovery::MANIFEST,
                crate::discovery::DRIVES_SURFACE,
                if *reports == 1 {
                    "operation"
                } else {
                    "operations"
                },
            ),
            Self::SurfaceAmbiguous {
                target,
                operation,
                first,
                second,
            } => write!(
                f,
                "the {target} smoke reported `{operation}` as `{first}` and as `{second}`. One operation answered two ways in one run is two call sites disagreeing about which operation they drove"
            ),
            Self::SurfaceThrottled { target, throttled } => write!(
                f,
                "the {target} smoke reported `{}` for {}: {}. That answer says the instance never looked at the request, so it proves nothing about the operation it was asking about — and a smoke that let it through would report every operation as driven while proving nothing about any of them. The answer names how long to wait in its `Retry-After` header: wait it out and ask again",
                crate::surface::THROTTLED,
                if throttled.len() == 1 {
                    "an operation"
                } else {
                    "operations"
                },
                throttled.join(", "),
            ),
            Self::SurfaceUnknown { target, unknown } => write!(
                f,
                "the {target} smoke reported {} the API document does not declare: {}. A report naming an operation that does not exist satisfies nothing, which is what makes it worth refusing rather than ignoring",
                if unknown.len() == 1 {
                    "an operation"
                } else {
                    "operations"
                },
                unknown.join(", "),
            ),
            Self::SurfaceMissing {
                target,
                missing,
                declared,
            } => write!(
                f,
                "the {target} smoke drove {} of the {declared} operations the API document declares. It never drove: {}. Every one of them ships in the {target} client and has never been asked to talk to a real Hook0",
                declared - missing.len(),
                missing.join(", "),
            ),
            Self::ModelsUnknown { target, unknown } => write!(
                f,
                "the {target} smoke said it decoded {} the generator does not emit: {}. Report a model under the name the API document declares it with, not the name the language spells it with",
                if unknown.len() == 1 {
                    "a type"
                } else {
                    "types"
                },
                unknown.join(", "),
            ),
            Self::ModelsMissing {
                target,
                missing,
                answered,
            } => write!(
                f,
                "the {target} smoke decoded {} of the {answered} model types an operation answers. It never decoded: {}. Every operation could be reported and every one of them refused, and a client that decodes nothing at all would still satisfy the other bijection — this is the one that says a generated model was really parsed out of what Hook0 sent",
                answered - missing.len(),
                missing.join(", "),
            ),
        }
    }
}

impl std::error::Error for Error {}
