//! The instance the clients are held against, brought up and taken down.
//!
//! There are two ways up and they exist because of the runner rather than despite it: the CI
//! executor this repository has carries no Docker at all (see the note at the top of
//! `tests-e2e/.gitlab-ci.yml`), so there the API and the output worker are the binaries another job
//! built and Postgres and Mailpit are GitLab services. Everywhere else they are containers, from
//! the repository's own `docker-compose.yaml`, unmodified and with no override beside it.
//!
//! What is deliberately *not* two of, because that is where the two ways would drift apart:
//!
//! - the environment both are configured with, read from the compose file by [`crate::compose`];
//! - where the API is expected to answer, and how long it is given to;
//! - everything downstream — provisioning, the delivery, the smokes — which is handed a stack and
//!   cannot tell which way it came up.
//!
//! Three things cannot be shared, and each is named where it happens: `DISABLE_SERVING_WEBAPP`
//! comes from the API's image rather than the compose file; Mailpit and the receiver are reached at
//! different addresses depending on which side of a container boundary they sit; and the binaries
//! CI runs were built by another job rather than by the image build.
//!
//! Down runs whatever happened above it, including when a step in between refused.

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::compose::{self, Composition};
use crate::error::Error;
use crate::process;

/// Where the compose file is, relative to the repository root.
pub const COMPOSE_FILE: &str = "docker-compose.yaml";

/// The name the harness's own stack runs under, so it can never collide with a developer's.
pub const PROJECT: &str = "hook0-smoke";

/// A directory holding `hook0-api` and `hook0-output-worker`. Set when there is no Docker daemon
/// to bring containers up with, which is what CI does; its presence is the whole of the choice.
pub const BINARIES: &str = "HOOK0_SMOKE_BINARIES";

/// Set to anything to leave a failed run's stack standing, so that what it was doing can be asked
/// rather than guessed. A successful run always takes it down.
pub const KEEP: &str = "HOOK0_SMOKE_KEEP";

/// The compose service the reachability check is made from, and whose logs are shown when no
/// webhook arrives.
const WORKER_SERVICE: &str = "output-worker";

/// The compose service that carries an HTTP client, which is not an assumption: the compose file's
/// own health check for it is a `curl`.
const CURL_SERVICE: &str = "api";

/// The most log lines shown when a stack is asked what it was doing, and the most characters kept
/// of one of them.
const MAX_LOG_LINES: usize = 40;
const MAX_LOG_LINE_CHARS: usize = 400;

/// What the API is called where CI leaves it.
const API_BINARY: &str = "hook0-api";

/// What the output worker is called where CI leaves it.
const WORKER_BINARY: &str = "hook0-output-worker";

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

/// How long the reachability check is given. Generous for what it does — one hop over a local
/// bridge — because what it is measuring is whether an answer comes at all, not how fast.
const REACHABLE_WITHIN: Duration = Duration::from_secs(20);

/// A running instance, however it came to be running.
pub struct Stack {
    pub api: String,
    pub mailpit: String,
    /// The address the output worker reaches this harness's receiver on.
    pub receiver_host: String,
    /// What to do about it if the worker cannot reach that address, in a command.
    remedy: String,
    running: Running,
}

/// Which way the instance was brought up, which is the only thing tearing down needs to know.
enum Running {
    Containers,
    Processes(Vec<(&'static str, Child)>),
}

/// Brings the stack up, whichever way this machine can, and waits for the API to answer.
pub fn up() -> Result<Stack, Error> {
    let composition = compose::read(Path::new(COMPOSE_FILE))?;

    match std::env::var(BINARIES) {
        Ok(directory) if !directory.is_empty() => processes(&composition, Path::new(&directory)),
        _ => containers(&composition),
    }
}

/// Brings up the repository's compose file.
fn containers(composition: &Composition) -> Result<Stack, Error> {
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
    wait_for_api(&api_url(composition.api_port))?;
    let bridge = bridge()?;

    Ok(Stack {
        api: api_url(composition.api_port),
        // Published on the host by the compose file, so it is reached there rather than under the
        // name the containers know it by.
        mailpit: format!("http://127.0.0.1:{}", composition.mailpit_http_port),
        // The worker is on the other side of a container boundary, so it reaches this harness at
        // the gateway of the network Compose made rather than on loopback.
        receiver_host: bridge.gateway,
        remedy: format!(
            "sudo ufw allow from {} to any comment 'docker bridge'",
            bridge.subnet
        ),
        running: Running::Containers,
    })
}

/// Starts the API and the output worker as processes, configured exactly as the compose file
/// configures the containers.
fn processes(composition: &Composition, binaries: &Path) -> Result<Stack, Error> {
    println!(
        "== stack: no Docker daemon here, so the API and the worker are the binaries in {}",
        binaries.display()
    );

    let api = spawn(
        &binaries.join(API_BINARY),
        &composition.api,
        // The compose file does not carry this one: the API's image sets it (`ENV
        // DISABLE_SERVING_WEBAPP=true` in `api/Dockerfile`), so a container has it and a bare
        // process does not. Without it the API looks for a web application to serve and refuses to
        // start.
        &[("DISABLE_SERVING_WEBAPP", "true")],
    )?;

    // The order is not this file's invention: the compose file declares `output-worker` as
    // depending on `api` being *healthy*, and the API is what runs the migrations. Started
    // alongside instead of behind it, the worker asks for `infrastructure.worker` before that table
    // exists and exits — leaving a stack that answers, provisions, and then never delivers.
    wait_for_api(&api_url(composition.api_port))?;
    let worker = spawn(&binaries.join(WORKER_BINARY), &composition.worker, &[])?;

    Ok(Stack {
        api: api_url(composition.api_port),
        // A service of the job, reached under the alias the compose file already names it by —
        // which is why the API's own `SMTP_CONNECTION_URL` works here unchanged.
        mailpit: format!(
            "http://{}:{}",
            composition.mailpit_host, composition.mailpit_http_port
        ),
        // The worker is a process beside this one, so loopback is where it reaches the receiver.
        receiver_host: "127.0.0.1".to_owned(),
        // Nothing sits between two processes on one host, so there is no rule to add: loopback that
        // does not answer is a receiver that is not there, which is this harness's own doing.
        remedy: "nothing stands between two processes on one host, so this is the harness's fault \
                 rather than the machine's"
            .to_owned(),
        running: Running::Processes(vec![(API_BINARY, api), (WORKER_BINARY, worker)]),
    })
}

/// Starts one of them, saying nothing this harness has not been asked for: what the API and the
/// worker write goes where the harness's own output goes, so a stack that refuses to start says why.
fn spawn(
    binary: &Path,
    environment: &[(String, String)],
    also: &[(&str, &str)],
) -> Result<Child, Error> {
    let mut command = Command::new(binary);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for (name, value) in environment {
        command.env(name, value);
    }
    for (name, value) in also {
        command.env(name, value);
    }

    command.spawn().map_err(|cause| Error::Command {
        program: binary.display().to_string(),
        cause,
    })
}

/// Where the API answers, which is the same expression either way: the compose file publishes the
/// port it tells the API to listen on.
fn api_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/api/v1")
}

/// The network Compose made, as the containers see this host across it.
struct Bridge {
    /// The address the containers reach this host on.
    gateway: String,
    /// The range those containers' own addresses come out of, which is what a firewall rule has to
    /// name for the delivery to arrive.
    subnet: String,
}

/// Reads both off the daemon rather than assuming either, because Docker hands out bridge subnets
/// out of a pool and the one a given machine got is not something to guess.
fn bridge() -> Result<Bridge, Error> {
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

    let described = process::must(
        "docker",
        &[
            "network",
            "inspect",
            &network,
            "--format",
            "{{range .IPAM.Config}}{{.Gateway}} {{.Subnet}}{{end}}",
        ],
        PROBE_WITHIN,
    )?;
    let mut said = described.split_whitespace();
    let (Some(gateway), Some(subnet)) = (said.next(), said.next()) else {
        return Err(Error::NoGateway {
            project: PROJECT.to_owned(),
        });
    };

    println!("== stack: the worker reaches this host at {gateway}, from {subnet}");
    Ok(Bridge {
        gateway: gateway.to_owned(),
        subnet: subnet.to_owned(),
    })
}

/// Fetches `url` from inside the stack, saying what came back or why nothing did.
///
/// `--write-out %{http_code}` is what makes a refusal readable: curl prints `000` and says on
/// standard error what stopped it, which is the sentence worth quoting.
fn fetched_in_container(url: &str) -> Result<String, String> {
    let seconds = REACHABLE_WITHIN.as_secs().to_string();
    let ended = process::capture(
        "docker",
        &[
            "compose",
            "--file",
            COMPOSE_FILE,
            "--project-name",
            PROJECT,
            "exec",
            "--no-TTY",
            CURL_SERVICE,
            "curl",
            "--silent",
            "--show-error",
            "--output",
            "/dev/null",
            "--max-time",
            &seconds,
            "--write-out",
            "%{http_code}",
            url,
        ],
        REACHABLE_WITHIN + PROBE_WITHIN,
    );

    match ended {
        Ok(ended) if ended.ok => Ok(format!("HTTP {}", ended.output.trim())),
        Ok(ended) => Err(format!("curl {}: {}", ended.status, ended.output.trim())),
        Err(refused) => Err(format!("{refused}")),
    }
}

/// Fetches `url` from this host, which is where a spawned worker would reach it from.
fn fetched_here(url: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REACHABLE_WITHIN))
        .build()
        .into();
    match agent.get(url).call() {
        Ok(answer) => Ok(format!("HTTP {}", answer.status())),
        Err(cause) => Err(format!("{cause}")),
    }
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
    /// Refuses if something this harness started has already stopped.
    ///
    /// Asked before anything is waited for, because a process that died takes its failure with it:
    /// without this, a worker that exited at startup shows up two minutes later as a webhook that
    /// never arrived, which names the wrong thing.
    pub fn still_running(&mut self) -> Result<(), Error> {
        let Running::Processes(running) = &mut self.running else {
            return Ok(());
        };
        for (what, child) in running.iter_mut() {
            if let Ok(Some(status)) = child.try_wait() {
                return Err(Error::StackDied {
                    what: (*what).to_owned(),
                    status: match status.code() {
                        Some(code) => format!("exited with {code}"),
                        None => "was killed by a signal".to_owned(),
                    },
                });
            }
        }
        Ok(())
    }

    /// Proves, from where the worker sits, that the receiver answers — before anything is
    /// provisioned and long before anything is waited for.
    ///
    /// This is the check that turns the harness's worst failure into its clearest one. Without it,
    /// a receiver the containers cannot reach is indistinguishable from a worker that does not
    /// dequeue, from a subscription that was never routed, from an event type that was never bound:
    /// all four arrive as the same silence, a minute or so after the last thing that printed. With
    /// it, the difference is settled in a fraction of a second, by a request over exactly the path
    /// the delivery will take, and the run stops before it has built anything on top of a stack
    /// that cannot do the one thing it is here for.
    ///
    /// In containers it is asked of the `api` service. That is not a guess about the image: the
    /// compose file's own health check for it *is* a `curl`, so an `api` that is up has one, and it
    /// sits on the same bridge network as the worker, which is what makes their route to this host
    /// the same route. In processes the worker is a sibling on this host, so the check is made from
    /// here, over the loopback it would use.
    pub fn reaches_receiver(&self, port: u16) -> Result<(), Error> {
        let url = format!("http://{}:{}/", self.receiver_host, port);
        let (from, answered) = match &self.running {
            Running::Containers => (
                format!("the `{CURL_SERVICE}` container, which shares `{WORKER_SERVICE}`'s network"),
                fetched_in_container(&url),
            ),
            Running::Processes(_) => (
                format!("this host, where `{WORKER_BINARY}` is running beside the harness"),
                fetched_here(&url),
            ),
        };

        match answered {
            Ok(status) => {
                println!("== control: {url} answers {status} from {from}, so a delivery can arrive");
                Ok(())
            }
            Err(said) => Err(Error::ReceiverUnreachable {
                from,
                url,
                said,
                remedy: self.remedy.clone(),
            }),
        }
    }

    /// What the output worker has been saying, bounded, for the moment it becomes worth reading.
    ///
    /// Which is the moment a delivery does not arrive, and it is the only moment: the logs of a
    /// worker that delivered are noise. Thrown away then — taken down before anything asks — they
    /// are the difference between a diagnosis and another run.
    pub fn worker_said(&self) -> String {
        match &self.running {
            Running::Containers => {
                let said = process::regardless(
                    "docker",
                    &[
                        "compose",
                        "--file",
                        COMPOSE_FILE,
                        "--project-name",
                        PROJECT,
                        "logs",
                        "--no-color",
                        "--tail",
                        &MAX_LOG_LINES.to_string(),
                        WORKER_SERVICE,
                    ],
                    PROBE_WITHIN,
                );
                // `--tail` bounds how many lines come back but nothing bounds how long one is, and
                // this worker logs response bodies.
                let mut kept: Vec<String> = said
                    .lines()
                    .rev()
                    .take(MAX_LOG_LINES)
                    .map(|line| {
                        let short: String = line.chars().take(MAX_LOG_LINE_CHARS).collect();
                        format!("    {short}")
                    })
                    .collect();
                kept.reverse();
                kept.join("\n")
            }
            // Nothing to fetch: a spawned worker writes to this harness's own output, so what it
            // said is already above the refusal rather than behind a command.
            Running::Processes(_) => format!(
                "    `{WORKER_BINARY}` is a process here, so everything it said is above this line"
            ),
        }
    }

    /// Takes down whatever this harness brought up, and says what happened.
    ///
    /// Called on the way out of every path, the failing ones included: a stack left standing after
    /// a refused run is one the next run inherits.
    ///
    /// The one exception is asked for out loud. With [`KEEP`] set, a run that refused leaves its
    /// stack standing, because the questions worth asking of a broken instance are the ones nobody
    /// thought of in advance — and none of them can be asked of containers that have been removed.
    /// A run that succeeded takes it down whatever is set.
    pub fn down(&mut self, refused: bool) {
        if refused && std::env::var_os(KEEP).is_some() {
            let by_hand = match &self.running {
                Running::Containers => format!(
                    "docker compose --file {COMPOSE_FILE} --project-name {PROJECT} down --volumes"
                ),
                Running::Processes(running) => format!(
                    "kill {}",
                    running
                        .iter()
                        .map(|(_, child)| child.id().to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            };
            println!("== stack: left standing because {KEEP} is set; take it down with\n   {by_hand}");
            return;
        }

        match &mut self.running {
            Running::Containers => {
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
                // Skipping past the ceiling rather than refusing at it: the teardown has already
                // happened by the time there is anything to print, and `down` returns nothing for a
                // caller to refuse with. What is bounded here is the console, not a decision.
                for line in said.lines().take(64) {
                    println!("{line}");
                }
            }
            Running::Processes(running) => {
                println!("== stack: stopping the API and the worker");
                for (_, child) in running.iter_mut() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                running.clear();
            }
        }
    }
}
