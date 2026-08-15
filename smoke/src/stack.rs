//! The instance the clients are held against, brought up and taken down.
//!
//! Up is the repository's own `docker-compose.yaml`, unmodified and with no override beside it:
//! what a client is smoked against is what a user gets when they follow the README, not a
//! composition written for the test. Down runs whatever happened above it, including when a step
//! in between refused.
//!
//! There is a second way in, and it exists because of the runner rather than despite it: the CI
//! executor this repository has carries no Docker at all (see the note at the top of
//! `tests-e2e/.gitlab-ci.yml`), so there the same processes are started by the job as GitLab
//! services and this attaches to them. Both ways answer the same three questions — where the API
//! is, where Mailpit is, and what address the output worker reaches this host on — and everything
//! downstream is identical.

use std::time::{Duration, Instant};

use crate::error::Error;
use crate::process;

/// Where the compose file is, relative to the repository root.
pub const COMPOSE_FILE: &str = "docker-compose.yaml";

/// The name the harness's own stack runs under, so it can never collide with a developer's.
pub const PROJECT: &str = "hook0-smoke";

/// Set to the base URL of an API already running, which is what CI does. Its presence is what
/// picks the second way in.
pub const ATTACH: &str = "HOOK0_SMOKE_API_URL";

/// Where Mailpit answers, when attaching. On a Kubernetes executor that is a service alias rather
/// than loopback, so it cannot be assumed.
pub const ATTACH_MAILPIT: &str = "HOOK0_SMOKE_MAILPIT_URL";

/// The address the output worker reaches this harness on, when attaching. Loopback when the worker
/// is a process beside it, which is what CI runs.
pub const ATTACH_RECEIVER_HOST: &str = "HOOK0_SMOKE_RECEIVER_HOST";

/// How long the stack is given to answer before it is called broken rather than slow.
const HEALTHY_WITHIN: Duration = Duration::from_secs(300);

/// How long `up` itself is given, which is longer than the wait above because it may pull.
const UP_WITHIN: Duration = Duration::from_secs(600);

/// How long tearing down is given before it is abandoned.
const DOWN_WITHIN: Duration = Duration::from_secs(180);

/// How long one readiness probe is given.
const PROBE_WITHIN: Duration = Duration::from_secs(15);

/// How often the API is asked whether it is up yet.
const PROBE_EVERY: Duration = Duration::from_secs(2);

/// A running instance, however it came to be running.
pub struct Stack {
    pub api: String,
    pub mailpit: String,
    /// The address the output worker reaches this harness's receiver on.
    pub receiver_host: String,
    /// Whether tearing down is this harness's job.
    owned: bool,
}

/// Brings the stack up, or attaches to one already running, and waits for the API to answer.
pub fn up() -> Result<Stack, Error> {
    let stack = match std::env::var(ATTACH) {
        Ok(api) if !api.is_empty() => attach(api)?,
        _ => compose_up()?,
    };
    wait_for_api(&stack.api)?;
    Ok(stack)
}

/// Attaches to processes somebody else started.
fn attach(api: String) -> Result<Stack, Error> {
    let mailpit = std::env::var(ATTACH_MAILPIT).unwrap_or_default();
    if mailpit.is_empty() {
        return Err(Error::MissingSetting {
            name: ATTACH_MAILPIT.to_owned(),
            why: "attaching to a running stack means this harness cannot know where Mailpit is; \
                  the address a registration's verification email is read from has to be named"
                .to_owned(),
        });
    }
    let receiver_host = match std::env::var(ATTACH_RECEIVER_HOST) {
        Ok(host) if !host.is_empty() => host,
        _ => "127.0.0.1".to_owned(),
    };
    Ok(Stack {
        api: api.trim_end_matches('/').to_owned(),
        mailpit: mailpit.trim_end_matches('/').to_owned(),
        receiver_host,
        owned: false,
    })
}

/// Brings up the repository's compose file.
fn compose_up() -> Result<Stack, Error> {
    println!("== stack: docker compose up ({PROJECT})");
    process::must(
        "docker",
        &[
            "compose",
            "--file",
            COMPOSE_FILE,
            "--project-name",
            PROJECT,
            "up",
            "--detach",
            "--wait",
            "--wait-timeout",
            &HEALTHY_WITHIN.as_secs().to_string(),
        ],
        UP_WITHIN,
    )?;

    Ok(Stack {
        api: "http://127.0.0.1:8081/api/v1".to_owned(),
        mailpit: "http://127.0.0.1:8025".to_owned(),
        receiver_host: gateway()?,
        owned: true,
    })
}

/// The address the containers reach this host on: the gateway of the network Compose made.
///
/// Read off the daemon rather than assumed, because Docker hands out bridge subnets and the one a
/// given machine got is not something to guess.
fn gateway() -> Result<String, Error> {
    let networks = process::must(
        "docker",
        &[
            "network",
            "ls",
            "--filter",
            &format!("label=com.docker.compose.project={PROJECT}"),
            "--format",
            "{{.Name}}",
        ],
        PROBE_WITHIN,
    )?;
    let network = networks
        .lines()
        .map(str::trim)
        .find(|name| !name.is_empty())
        .ok_or_else(|| Error::NoGateway {
            project: PROJECT.to_owned(),
        })?
        .to_owned();

    let address = process::must(
        "docker",
        &[
            "network",
            "inspect",
            &network,
            "--format",
            "{{range .IPAM.Config}}{{.Gateway}}{{end}}",
        ],
        PROBE_WITHIN,
    )?;
    let address = address.trim();
    if address.is_empty() {
        return Err(Error::NoGateway {
            project: PROJECT.to_owned(),
        });
    }
    println!("== stack: the worker reaches this host at {address}");
    Ok(address.to_owned())
}

/// Asks the API for its OpenAPI document until it answers, or until the deadline.
fn wait_for_api(api: &str) -> Result<(), Error> {
    let url = format!("{api}/swagger.json");
    let deadline = Instant::now() + HEALTHY_WITHIN;
    let mut last = String::from("not attempted");

    while Instant::now() < deadline {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(PROBE_WITHIN))
            .build()
            .into();
        match agent.get(&url).call() {
            Ok(answer) if answer.status().is_success() => {
                println!("== stack: the API answers at {api}");
                return Ok(());
            }
            Ok(answer) => last = format!("HTTP {}", answer.status()),
            Err(cause) => last = format!("{cause}"),
        }
        std::thread::sleep(PROBE_EVERY);
    }

    Err(Error::StackNeverHealthy {
        url,
        seconds: HEALTHY_WITHIN.as_secs(),
        last,
    })
}

impl Stack {
    /// Takes down whatever this harness brought up, volumes included, and says what happened.
    ///
    /// Called on the way out of every path, the failing ones included: a stack left standing after
    /// a refused run is one the next run inherits.
    pub fn down(&self) {
        if !self.owned {
            println!("== stack: attached rather than started, so nothing to take down");
            return;
        }
        println!("== stack: docker compose down");
        let said = process::regardless(
            "docker",
            &[
                "compose",
                "--file",
                COMPOSE_FILE,
                "--project-name",
                PROJECT,
                "down",
                "--volumes",
                "--remove-orphans",
                "--timeout",
                "30",
            ],
            DOWN_WITHIN,
        );
        for line in said.lines().take(64) {
            println!("{line}");
        }
    }
}
