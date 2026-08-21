//! What both ways of bringing the stack up are configured from.
//!
//! Two questions. The first is asked of fabricated files, because it is about reading: what a
//! compose file says, and what is refused when it says nothing. The second is asked of the
//! repository's own `docker-compose.yaml`, and it is the one that earns its place — the harness
//! depends on a handful of settings being declared there, and if one of them is removed the
//! failure without this test is a smoke that provisions fine and then waits two minutes for a
//! webhook that was never going to arrive.

use std::fs;
use std::path::Path;

use hook0_smoke::compose::read;
use hook0_smoke::error::Error;

/// A compose file with nothing in it but what a test puts there.
struct Written(tempfile::TempDir);

impl Written {
    fn saying(body: &str) -> Written {
        let directory = tempfile::tempdir().expect("a temporary directory");
        fs::write(directory.path().join("docker-compose.yaml"), body).expect("the compose file");
        Written(directory)
    }

    fn path(&self) -> std::path::PathBuf {
        self.0.path().join("docker-compose.yaml")
    }
}

/// A compose file naming everything the harness reads, so that a case can take one thing away.
fn complete() -> String {
    "\
services:
  api:
    environment:
      - PORT=8081
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/hook0
      - SMTP_CONNECTION_URL=smtp://mailpit:1025
      - ESCAPED=$${KEPT}
  output-worker:
    environment:
      - WORKER_NAME=default
  mailpit:
    healthcheck:
      test: [\"CMD-SHELL\", \"wget --spider http://localhost:8025/api/v1/info || exit 1\"]
"
    .to_owned()
}

fn refusal(body: &str) -> Error {
    let written = Written::saying(body);
    read(&written.path()).expect_err("a refusal")
}

#[test]
fn a_service_environment_is_read_as_the_pairs_it_declares() {
    let written = Written::saying(&complete());

    let composition = read(&written.path()).expect("the composition");

    assert_eq!(composition.api_port, 8081);
    assert_eq!(
        composition.api_setting("DATABASE_URL"),
        Some("postgres://postgres:postgres@postgres:5432/hook0")
    );
    assert_eq!(composition.worker.len(), 1);
    assert_eq!(composition.worker[0].0, "WORKER_NAME");
}

#[test]
fn what_compose_escapes_is_handed_on_unescaped() {
    let written = Written::saying(&complete());

    let composition = read(&written.path()).expect("the composition");

    assert_eq!(
        composition.api_setting("ESCAPED"),
        Some("${KEPT}"),
        "`$$` is how compose writes a literal `$`; a process is handed the one it means"
    );
}

#[test]
fn mailpit_is_found_where_the_api_is_told_to_send_and_where_its_health_check_asks() {
    let written = Written::saying(&complete());

    let composition = read(&written.path()).expect("the composition");

    assert_eq!(composition.mailpit_host, "mailpit");
    assert_eq!(composition.mailpit_http_port, 8025);
}

#[test]
fn a_service_declaring_no_environment_is_refused() {
    let refused = refusal(&complete().replace("      - WORKER_NAME=default\n", ""));

    assert!(
        format!("{refused}").contains("output-worker"),
        "the service that says nothing is named: {refused}"
    );
}

#[test]
fn an_api_naming_no_port_is_refused_rather_than_guessed() {
    let refused = refusal(&complete().replace("      - PORT=8081\n", ""));

    assert!(format!("{refused}").contains("PORT"), "{refused}");
}

#[test]
fn what_this_repository_declares_today_carries_what_the_harness_depends_on() {
    let composition = read(Path::new("../docker-compose.yaml")).expect("the composition");

    for named in [
        // Without these the API does not come up at all, either way it is started.
        "DATABASE_URL",
        "BISCUIT_PRIVATE_KEY",
        // The account the harness registers is verified out of the email this address sends to.
        "SMTP_CONNECTION_URL",
        "EMAIL_SENDER_ADDRESS",
        "PORT",
    ] {
        assert!(
            composition.api_setting(named).is_some(),
            "the compose file's `api` service no longer declares {named}, which the smoke needs"
        );
    }

    let worker: Vec<&str> = composition
        .worker
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        worker.contains(&"DISABLE_TARGET_IP_CHECK"),
        "the compose file's `output-worker` no longer disables the target IP check; the smoke's \
         subscription points at a private address, so no webhook would ever be delivered and the \
         signature every client verifies would never be produced"
    );
}
