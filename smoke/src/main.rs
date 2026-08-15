//! Holds every client this repository generates against a Hook0 that is really running.
//!
//! The suites each client ships are exhaustive about behaviour — retry counts, timing, truncated
//! answers, hostile headers — and they get that by talking to a server the suite starts itself.
//! What none of them can do is prove the client can talk to Hook0 at all: that the token is
//! accepted, that a problem document is shaped the way the client reads it, that a duplicated
//! ingestion is refused the way the client expects, and that a signature the *server* computed
//! verifies. A client can pass everything it has and still fail on first contact. This closes that,
//! once per language, and no more than once: a second copy of each suite run against a real server
//! would be slow, flaky and would prove nothing the loopback one did not.
//!
//! The set of clients is the generator's registry. Nothing here lists them, and a target that has
//! no smoke stops the run rather than being skipped.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use hook0_smoke::error::Error;
use hook0_smoke::{api, discovery, process, receiver, stack};

/// Where the per-language smokes live, relative to this crate.
const LANGUAGES: &str = "languages";

/// Where what the instance delivered is written for the smokes to read.
///
/// One plain file per part rather than one document: eleven of the twelve languages would
/// otherwise have to parse JSON before they could verify anything, and a smoke that spends most of
/// itself on a parser is one nobody reads. The body is written as the bytes that arrived.
const DELIVERY: &str = "delivery";

/// How long one language's smoke is given, compiling included. Java and C# build from cold here;
/// anything past this is a run that is stuck rather than one that is slow.
const SMOKE_WITHIN: Duration = Duration::from_secs(900);

/// How long the instance is given to deliver the webhook the signature is read off.
const DELIVERY_WITHIN: Duration = Duration::from_secs(120);

/// How far from the moment it names a signature may be held. Wide on purpose: the delivery is
/// caught once, at the start, and the last language to verify it does so after every toolchain
/// ahead of it has compiled. What is under test here is the code over bytes the server produced,
/// not this harness's clock — the tolerance window itself is what the shared conformance corpus
/// exercises, in every client, against vectors with a moment pinned in them.
const TOLERANCE_SECONDS: u64 = 3600;

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("\n== every client talked to a real Hook0");
            ExitCode::SUCCESS
        }
        Err(refused) => {
            eprintln!("\n== refused: {refused}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Error> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository = crate_root.parent().unwrap_or(&crate_root).to_path_buf();
    std::env::set_current_dir(&repository).map_err(|cause| Error::ReadDirectory {
        path: repository.display().to_string(),
        cause,
    })?;

    let targets: Vec<String> = hook0_sdkgen::targets::targets()
        .iter()
        .map(|target| target.name.to_owned())
        .collect();
    let smokes = discovery::discover(&targets, &crate_root.join(LANGUAGES))?;
    println!(
        "== registry: {} targets, each with a smoke: {}",
        smokes.len(),
        smokes
            .iter()
            .map(|smoke| smoke.target.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let stack = stack::up()?;
    let outcome = exercise(&stack, &crate_root, &smokes);
    stack.down();
    outcome
}

/// Provisions, catches one real delivery, and runs every smoke against both.
fn exercise(
    stack: &stack::Stack,
    crate_root: &Path,
    smokes: &[discovery::Smoke],
) -> Result<(), Error> {
    let nonce = nonce()?;
    let listening = receiver::listen()?;
    let receiver_url = format!("http://{}:{}/", stack.receiver_host, listening.port);

    let provisioned = api::provision(&stack.api, &stack.mailpit, &nonce, &receiver_url)?;
    api::emit(&stack.api, &provisioned, &nonce)?;

    let delivered = listening.first(DELIVERY_WITHIN)?;
    let signature = delivered
        .header("x-hook0-signature")
        .ok_or_else(|| Error::Answer {
            what: "catching the delivery".to_owned(),
            detail: "the instance delivered a webhook carrying no `x-hook0-signature`".to_owned(),
        })?
        .to_owned();
    println!("== delivery: caught, signed by the instance as {signature}");

    let delivery = crate_root.join(DELIVERY);
    write_delivery(&delivery, &delivered, &signature, &provisioned)?;

    let environment = vec![
        ("HOOK0_API_URL".to_owned(), stack.api.clone()),
        (
            "HOOK0_APPLICATION_ID".to_owned(),
            provisioned.application_id.clone(),
        ),
        ("HOOK0_TOKEN".to_owned(), provisioned.token.clone()),
        (
            "HOOK0_EVENT_TYPE".to_owned(),
            provisioned.event_type.clone(),
        ),
        ("HOOK0_DELIVERY".to_owned(), delivery.display().to_string()),
    ];

    let mut failed = Vec::new();
    for smoke in smokes {
        println!(
            "\n== {} =========================================",
            smoke.target
        );
        let (program, arguments) = smoke.command.split_at(1);
        let ended = process::stream(
            &program[0],
            arguments,
            &smoke.directory,
            &environment,
            SMOKE_WITHIN,
        )?;
        if ended.ok {
            println!("== {}: passed", smoke.target);
        } else {
            println!("== {}: FAILED, the smoke {}", smoke.target, ended.status);
            failed.push(smoke.target.clone());
        }
    }

    if failed.is_empty() {
        Ok(())
    } else {
        Err(Error::SmokesFailed { failed })
    }
}

/// Writes the delivery exactly as it arrived, for every smoke to verify with its own code.
///
/// `headers` is one `name: value` per line, lowercased names, values as delivered, in the order
/// they arrived — which is the shape they came off the socket in and the shape every client's
/// verification wants them back in.
fn write_delivery(
    at: &Path,
    delivered: &receiver::Delivery,
    signature: &str,
    provisioned: &api::Provisioned,
) -> Result<(), Error> {
    let write = |name: &str, what: &[u8]| -> Result<(), Error> {
        let path = at.join(name);
        fs::write(&path, what).map_err(|cause| Error::ReadManifest {
            path: path.display().to_string(),
            cause,
        })
    };

    fs::create_dir_all(at).map_err(|cause| Error::ReadManifest {
        path: at.display().to_string(),
        cause,
    })?;

    let headers = delivered
        .headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\n"))
        .collect::<String>();

    write("secret", provisioned.subscription_secret.as_bytes())?;
    write("signature", signature.as_bytes())?;
    write("body", &delivered.body)?;
    write("headers", headers.as_bytes())?;
    write("tolerance", TOLERANCE_SECONDS.to_string().as_bytes())
}

/// Something no other run of this harness will have used, so a rerun against a stack somebody kept
/// is a rerun rather than a collision.
fn nonce() -> Result<String, Error> {
    let mut random = [0u8; 8];
    fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut random))
        .map_err(|cause| Error::ReadManifest {
            path: "/dev/urandom".to_owned(),
            cause,
        })?;
    Ok(random.iter().map(|byte| format!("{byte:02x}")).collect())
}
