//! What the repository says a Hook0 instance is, read out of the one file that says it.
//!
//! The stack is brought up two ways — Compose where there is a Docker daemon, plain processes on
//! the runner that has none — and the failure that costs the most is the one where those two drift
//! apart quietly: an API configured slightly differently in CI than on a developer's machine gives
//! a smoke that passes in one place and fails in the other for a reason that has nothing to do with
//! any client. So neither way carries its own copy of the configuration. Both read it from
//! `docker-compose.yaml`: the same environment, the same port, the same Mailpit.
//!
//! Only what cannot be shared is written down twice, and each of those is named where it happens.

use std::fs;
use std::path::Path;

use yaml_rust2::{Yaml, YamlLoader};

use crate::error::Error;

/// The most bytes of the compose file read. It is a hundred lines; anything past this is not it.
const MAX_COMPOSE_BYTES: u64 = 1 << 20;

/// The most environment entries read out of one service.
const MAX_ENVIRONMENT: usize = 128;

/// The service that answers the API.
const API: &str = "api";

/// The service that delivers webhooks.
const WORKER: &str = "output-worker";

/// The service that catches what the instance sends by email.
const MAILPIT: &str = "mailpit";

/// What both ways of bringing the stack up are configured from.
#[derive(Debug, Clone)]
pub struct Composition {
    /// The API's environment, exactly as the compose file declares it.
    pub api: Vec<(String, String)>,
    /// The output worker's environment, exactly as the compose file declares it.
    pub worker: Vec<(String, String)>,
    /// The port the API listens on, out of its own environment.
    pub api_port: u16,
    /// The host Mailpit answers on, out of the address the API is told to send mail to.
    pub mailpit_host: String,
    /// The port Mailpit's HTTP API answers on, out of what its health check asks.
    pub mailpit_http_port: u16,
}

impl Composition {
    /// What one environment entry says, or nothing when the service does not declare it.
    pub fn api_setting(&self, name: &str) -> Option<&str> {
        self.api
            .iter()
            .find(|(declared, _)| declared == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Reads the composition, or refuses naming what the file does not say.
pub fn read(path: &Path) -> Result<Composition, Error> {
    let refuse = |detail: String| Error::Composition {
        path: path.display().to_string(),
        detail,
    };

    let size = fs::metadata(path)
        .map_err(|cause| Error::ReadManifest {
            path: path.display().to_string(),
            cause,
        })?
        .len();
    if size > MAX_COMPOSE_BYTES {
        return Err(refuse(format!(
            "larger than {MAX_COMPOSE_BYTES} bytes, so it is not a compose file"
        )));
    }

    let body = fs::read_to_string(path).map_err(|cause| Error::ReadManifest {
        path: path.display().to_string(),
        cause,
    })?;
    let documents =
        YamlLoader::load_from_str(&body).map_err(|cause| refuse(format!("unreadable: {cause}")))?;
    let document = documents
        .first()
        .ok_or_else(|| refuse("holds no document".to_owned()))?;
    let services = &document["services"];

    let api = environment(&services[API], API, path)?;
    let worker = environment(&services[WORKER], WORKER, path)?;

    let api_port = api
        .iter()
        .find(|(name, _)| name == "PORT")
        .ok_or_else(|| refuse(format!("the `{API}` service declares no PORT")))?
        .1
        .parse()
        .map_err(|cause| refuse(format!("the `{API}` service's PORT is not a port: {cause}")))?;

    let smtp = api
        .iter()
        .find(|(name, _)| name == "SMTP_CONNECTION_URL")
        .ok_or_else(|| {
            refuse(format!(
                "the `{API}` service declares no SMTP_CONNECTION_URL"
            ))
        })?
        .1
        .clone();
    let mailpit_host = host_of(&smtp).ok_or_else(|| {
        refuse(format!(
            "SMTP_CONNECTION_URL names no host this can reach Mailpit at: {smtp}"
        ))
    })?;

    let probe = services[MAILPIT]["healthcheck"]["test"][1]
        .as_str()
        .ok_or_else(|| refuse(format!("the `{MAILPIT}` service declares no health check")))?;
    let mailpit_http_port = port_in(probe).ok_or_else(|| {
        refuse(format!(
            "the `{MAILPIT}` health check names no HTTP port, so where its API answers is unknown: {probe}"
        ))
    })?;

    Ok(Composition {
        api,
        worker,
        api_port,
        mailpit_host,
        mailpit_http_port,
    })
}

/// One service's environment, as a list of pairs in the order the file declares them.
fn environment(service: &Yaml, name: &str, path: &Path) -> Result<Vec<(String, String)>, Error> {
    let refuse = |detail: String| Error::Composition {
        path: path.display().to_string(),
        detail,
    };

    let declared = service["environment"]
        .as_vec()
        .ok_or_else(|| refuse(format!("the `{name}` service declares no environment")))?;
    if declared.len() > MAX_ENVIRONMENT {
        return Err(refuse(format!(
            "the `{name}` service declares more than {MAX_ENVIRONMENT} environment entries"
        )));
    }

    declared
        .iter()
        .map(|entry| {
            let written = entry
                .as_str()
                .ok_or_else(|| refuse(format!("the `{name}` service declares a non-text entry")))?;
            let (variable, value) = written.split_once('=').ok_or_else(|| {
                refuse(format!(
                    "the `{name}` service declares `{written}`, which names no value"
                ))
            })?;
            // Compose doubles a `$` to escape it, which is how the file writes a shell variable it
            // wants passed through rather than expanded. Nothing here expands anything, so the
            // escape is undone and the value is what the container would see.
            Ok((variable.to_owned(), value.replace("$$", "$")))
        })
        .collect()
}

/// The host part of a `scheme://host:port` address.
fn host_of(url: &str) -> Option<String> {
    let after = url.split_once("//")?.1;
    let host = after.split([':', '/']).next()?;
    if host.is_empty() {
        return None;
    }
    Some(host.to_owned())
}

/// The first port named after `http://<something>:` in a command line.
fn port_in(command: &str) -> Option<u16> {
    let at = command.find("http://")?;
    let after = command[at + "http://".len()..].split_once(':')?.1;
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_is_read_out_of_an_address() {
        assert_eq!(host_of("smtp://mailpit:1025").as_deref(), Some("mailpit"));
        assert_eq!(host_of("smtp://mailpit").as_deref(), Some("mailpit"));
        assert_eq!(host_of("mailpit:1025"), None);
    }

    #[test]
    fn a_port_is_read_out_of_a_health_check() {
        assert_eq!(
            port_in("wget --spider http://localhost:8025/api/v1/info || exit 1"),
            Some(8025)
        );
        assert_eq!(port_in("wget --spider http://localhost/ || exit 1"), None);
    }
}
