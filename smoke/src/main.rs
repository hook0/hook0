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
//! What it does ask, once per language, is the whole of the generated surface: every operation the
//! API document declares, driven against the real instance, reported one line at a time and held to
//! a bijection with what the generator declares. That is the part no loopback suite can do for
//! itself — each of them drives its surface against a server the suite wrote, and nothing checks
//! that server against the real one.
//!
//! The set of clients is the generator's registry, and the set of operations is the API document.
//! Nothing here lists either, and a target that has no smoke stops the run rather than being
//! skipped.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use hook0_sdkgen::targets::Decoding;
use hook0_smoke::error::Error;
use hook0_smoke::{api, discovery, process, receiver, stack, surface, worker};

/// Where the per-language smokes live, relative to this crate.
const LANGUAGES: &str = "languages";

/// The API document every client is generated from, relative to the repository root. It is what
/// the set of operations each smoke is held to comes from, read through the generator itself.
const SNAPSHOT: &str = "api/openapi.snapshot.json";

/// Where what the instance delivered is written for the smokes to read.
///
/// One plain file per part rather than one document: eleven of the twelve languages would
/// otherwise have to parse JSON before they could verify anything, and a smoke that spends most of
/// itself on a parser is one nobody reads. The body is written as the bytes that arrived.
const DELIVERY: &str = "delivery";

/// How long one language's smoke is given, compiling included. Java and C# build from cold here;
/// anything past this is a run that is stuck rather than one that is slow.
const SMOKE_WITHIN: Duration = Duration::from_secs(900);

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

    // Read before anything is started. It costs nothing, and a document that cannot be read is a
    // refusal worth having before a stack has been brought up for it.
    let snapshot = repository.join(SNAPSHOT);
    let declared = declared_per_target(&snapshot)?;
    let models = models_per_target(&snapshot)?;
    let awaiting: Vec<&str> = smokes
        .iter()
        .filter(|smoke| !smoke.drives_surface)
        .map(|smoke| smoke.target.as_str())
        .collect();
    println!(
        "== document: every smoke below is held to the operations its own client is generated from \
         — {}{}",
        surfaces(&declared),
        if awaiting.is_empty() {
            String::new()
        } else {
            format!(
                " — except {}, whose manifests still say `{} = false`",
                awaiting.join(", "),
                discovery::DRIVES_SURFACE,
            )
        }
    );

    let mut stack = stack::up()?;
    let outcome = exercise(&mut stack, &crate_root, &smokes, &declared, &models);
    stack.down(outcome.is_err());
    outcome
}

/// What each target's client is generated from, keyed by the target it belongs to.
///
/// Per target rather than one set for the run, and read off the generator's registry rather than
/// decided here: the registry says which tag each target selects out of the document, and they do
/// not all select the same one — the eleven SDKs are generated from the SDK surface and the MCP
/// server from its own. A single set written down here would hold one of them to the wrong thing.
fn declared_per_target(snapshot: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, Error> {
    let mut per_tag: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    let mut per_target = BTreeMap::new();

    for target in hook0_sdkgen::targets::targets() {
        let operations = match per_tag.get(target.tag) {
            Some(read) => read.clone(),
            None => {
                let read = surface::declared(snapshot, target.tag)?;
                per_tag.insert(target.tag, read.clone());
                read
            }
        };
        per_target.insert(target.name.to_owned(), operations);
    }

    Ok(per_target)
}

/// What each target's client is generated from in the way of model types, keyed by target.
fn models_per_target(snapshot: &Path) -> Result<BTreeMap<String, surface::Models>, Error> {
    // Keyed by the tag *and* how the target reads an answer, because the two together are what the
    // set depends on. Keying by the tag alone would hand a pass-through target the set derived for
    // the modelled ones that share its tag — which is the very mistake this argument exists to
    // stop, reintroduced one layer up by a cache.
    let mut per_reading: BTreeMap<(&str, Decoding), surface::Models> = BTreeMap::new();
    let mut per_target = BTreeMap::new();

    for target in hook0_sdkgen::targets::targets() {
        let reading = (target.tag, target.decoding);
        let models = match per_reading.get(&reading) {
            Some(read) => read.clone(),
            None => {
                let read = surface::models(snapshot, target.tag, target.decoding)?;
                per_reading.insert(reading, read.clone());
                read
            }
        };
        per_target.insert(target.name.to_owned(), models);
    }

    Ok(per_target)
}

/// How many operations each tag the registry selects carries, said once for the run's banner.
fn surfaces(declared: &BTreeMap<String, BTreeSet<String>>) -> String {
    let mut counted: BTreeMap<&str, usize> = BTreeMap::new();
    for target in hook0_sdkgen::targets::targets() {
        if let Some(operations) = declared.get(target.name) {
            counted.insert(target.tag, operations.len());
        }
    }
    counted
        .iter()
        .map(|(tag, count)| format!("{count} tagged `{tag}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Provisions, catches one real delivery, and runs every smoke against both.
fn exercise(
    stack: &mut stack::Stack,
    crate_root: &Path,
    smokes: &[discovery::Smoke],
    declared: &BTreeMap<String, BTreeSet<String>>,
    models: &BTreeMap<String, surface::Models>,
) -> Result<(), Error> {
    let nonce = nonce()?;
    let listening = receiver::listen()?;
    let receiver_url = format!("http://{}:{}/", stack.receiver_host, listening.port);

    println!(
        "\n== control: every smoke below verifies a signature this instance produced, so the \
         instance has to produce one first. Nothing runs until it has."
    );

    // Asked here, before an account exists, because it is the one thing that can be settled in a
    // fraction of a second and would otherwise be discovered as a minute of silence with four
    // plausible causes.
    stack.reaches_receiver(listening.port)?;

    let account = api::account(&stack.api, &stack.mailpit, &nonce)?;
    let provisioned = api::provision(&stack.api, &account, &nonce, &receiver_url)?;
    api::emit(&stack.api, &provisioned, &nonce)?;

    stack.still_running()?;
    println!(
        "== control: waiting up to {}s for one signed webhook ({})",
        worker::DELIVERS_WITHIN.as_secs(),
        worker::expectation()
    );
    let delivered = match listening.first(worker::DELIVERS_WITHIN) {
        Some(delivered) => delivered,
        None => {
            return Err(Error::NoDelivery {
                seconds: worker::DELIVERS_WITHIN.as_secs(),
                expectation: worker::expectation(),
                worker_said: stack.worker_said(),
            });
        }
    };
    println!(
        "== control: passed. The instance signed and delivered {} bytes as {}, and that is what \
         all {} smokes verify",
        delivered.body.len(),
        delivered.signature,
        smokes.len()
    );

    let delivery = crate_root.join(DELIVERY);
    write_delivery(&delivery, &delivered, &provisioned)?;

    // The second credential every language is handed. An application secret cannot reach what
    // belongs to the organization, so a smoke holding only one of the two could report those
    // operations but never be answered by one, and the types they answer would go undecoded.
    let service_token = api::service_token(&stack.api, &account, &format!("smoke-{nonce}"))?;

    // Two things a client can only be answered once the instance has done something on its own,
    // asked for here, once, rather than waited for twelve times: the attempt and the response the
    // delivery above left behind, and the per-day counts the API refreshes on its own cycle. Every
    // language reads all three back with the organization credential it is handed.
    let attempted = api::attempted(
        &stack.api,
        &account,
        &provisioned.application.application_id,
    )?;
    api::counted(&stack.api, &account)?;

    // What every language is handed the same copy of: where the API is, which organization its
    // application belongs to, the organization credential, and the one delivery they all verify.
    let shared = vec![
        ("HOOK0_API_URL".to_owned(), stack.api.clone()),
        (
            "HOOK0_ORGANIZATION_ID".to_owned(),
            account.organization_id.clone(),
        ),
        ("HOOK0_SERVICE_TOKEN".to_owned(), service_token),
        (
            "HOOK0_SEEDED_APPLICATION_ID".to_owned(),
            provisioned.application.application_id.clone(),
        ),
        (
            "HOOK0_REQUEST_ATTEMPT_ID".to_owned(),
            attempted.request_attempt_id,
        ),
        ("HOOK0_RESPONSE_ID".to_owned(), attempted.response_id),
        ("HOOK0_DELIVERY".to_owned(), delivery.display().to_string()),
    ];

    let mut failed = Vec::new();
    for smoke in smokes {
        println!(
            "\n== {} =========================================",
            smoke.target
        );

        // One application per language, created here rather than shared. Deleting an application,
        // an event type and a subscription are operations the clients declare and therefore
        // operations a smoke has to drive, and with one application between twelve the first
        // language to delete it would take the eleven behind it with it.
        let application = api::application(
            &stack.api,
            &account,
            &format!("smoke-{nonce}-{}", smoke.target),
        )?;

        let mut environment = shared.clone();
        environment.push((
            "HOOK0_APPLICATION_ID".to_owned(),
            application.application_id.clone(),
        ));
        environment.push(("HOOK0_TOKEN".to_owned(), application.token.clone()));
        environment.push((
            "HOOK0_EVENT_TYPE".to_owned(),
            application.event_type.clone(),
        ));
        // Asked before the smoke starts rather than left to it. A runtime whose packages sit
        // outside the system path needs a search path pointed at them, and one inherited from
        // whoever started this harness is state nobody declared.
        environment.extend(smoke.satisfied()?);

        let (program, arguments) = smoke.command.split_at(1);
        let ended = process::stream(
            &program[0],
            arguments,
            &smoke.directory,
            &environment,
            SMOKE_WITHIN,
            process::Keep {
                worth: surface::reported,
                most: surface::MAX_REPORTS,
            },
        )?;

        if !ended.ok {
            println!("== {}: FAILED, the smoke {}", smoke.target, ended.status);
            failed.push(smoke.target.clone());
            continue;
        }

        // Only of a smoke that passed. A smoke that failed has already said why, and holding what
        // it managed to report to a bijection would bury that under a second refusal.
        // Discovery already refused a directory naming no target, so neither of these can be
        // absent; refusing rather than defaulting keeps it that way.
        let (Some(operations), Some(types)) =
            (declared.get(&smoke.target), models.get(&smoke.target))
        else {
            return Err(Error::SmokesWithoutTarget {
                directories: vec![smoke.target.clone()],
            });
        };

        match surface::held(
            &smoke.target,
            smoke.drives_surface,
            operations,
            types,
            &ended.kept,
        ) {
            Ok(surface::Held { operations: 0, .. }) => println!(
                "== {}: passed. It drives no operation yet, which its `{}` says",
                smoke.target,
                discovery::MANIFEST
            ),
            Ok(held) => println!(
                "== {}: passed, and drove all {} operations the document declares, decoding {} of \
                 its model types",
                smoke.target, held.operations, held.models
            ),
            Err(refused) => {
                println!("== {}: FAILED. {refused}", smoke.target);
                failed.push(smoke.target.clone());
            }
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
    write("signature", delivered.signature.as_bytes())?;
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
