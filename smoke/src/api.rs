//! Everything a smoke needs, obtained the way a user obtains it.
//!
//! Nothing is inserted into the database and no master key is handed to the harness. An account is
//! registered, the verification email that registration sends is read out of the instance's own
//! Mailpit, the session that verification opens is what creates the application, and the token the
//! clients are given is an application secret the API minted. That is the point of the exercise:
//! the loopback suites already prove the clients behave, and what they cannot prove is that a
//! client can authenticate against a real Hook0 and read what a real Hook0 answers.
//!
//! Every request is bounded in time and every answer is bounded in size.

use std::io::Read;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::error::Error;

/// How long one request to the API or to Mailpit is given.
const REQUEST_WITHIN: Duration = Duration::from_secs(30);

/// The most bytes read out of one answer.
const MAX_ANSWER_BYTES: u64 = 1024 * 1024;

/// How long a registration's verification email is waited for.
const EMAIL_WITHIN: Duration = Duration::from_secs(60);

/// How often Mailpit is asked whether it arrived.
const EMAIL_EVERY: Duration = Duration::from_millis(500);

/// The most messages read out of Mailpit in one look.
const MAX_MESSAGES: usize = 50;

/// What the clients are handed, and what the signature they verify was produced with.
#[derive(Debug, Clone)]
pub struct Provisioned {
    pub application_id: String,
    /// An application secret, which is what an SDK authenticates with.
    pub token: String,
    pub event_type: String,
    pub subscription_secret: String,
}

/// An HTTP client that reports what the API said rather than only that it refused.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_WITHIN))
        .http_status_as_error(false)
        .build()
        .into()
}

/// Sends a document and reads one back, or refuses naming the status and the body.
fn post(what: &str, url: &str, token: Option<&str>, body: Value) -> Result<Value, Error> {
    let agent = agent();
    let mut request = agent.post(url).header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", &format!("Bearer {token}"));
    }

    let mut answer = request.send_json(&body).map_err(|cause| Error::Http {
        what: what.to_owned(),
        cause: format!("{cause}"),
    })?;
    let status = answer.status().as_u16();
    let read = read_body(&mut answer, what)?;

    if !(200..300).contains(&status) {
        return Err(Error::Api {
            what: what.to_owned(),
            status,
            body: read,
        });
    }
    if read.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&read).map_err(|cause| Error::Answer {
        what: what.to_owned(),
        detail: format!("the answer is not JSON ({cause}): {read}"),
    })
}

/// Reads a document, or refuses naming the status and the body.
fn get(what: &str, url: &str) -> Result<Value, Error> {
    let agent = agent();
    let mut answer = agent.get(url).call().map_err(|cause| Error::Http {
        what: what.to_owned(),
        cause: format!("{cause}"),
    })?;
    let status = answer.status().as_u16();
    let read = read_body(&mut answer, what)?;

    if !(200..300).contains(&status) {
        return Err(Error::Api {
            what: what.to_owned(),
            status,
            body: read,
        });
    }
    serde_json::from_str(&read).map_err(|cause| Error::Answer {
        what: what.to_owned(),
        detail: format!("the answer is not JSON ({cause}): {read}"),
    })
}

/// The body, up to the ceiling this harness sets for itself.
fn read_body(answer: &mut ureq::http::Response<ureq::Body>, what: &str) -> Result<String, Error> {
    let mut read = String::new();
    answer
        .body_mut()
        .as_reader()
        .take(MAX_ANSWER_BYTES)
        .read_to_string(&mut read)
        .map_err(|cause| Error::Http {
            what: what.to_owned(),
            cause: format!("the answer could not be read: {cause}"),
        })?;
    Ok(read)
}

/// The string a document declares under that name, or a refusal saying what came instead.
fn string(document: &Value, name: &str, what: &str) -> Result<String, Error> {
    document
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::Answer {
            what: what.to_owned(),
            detail: format!("the answer carries no `{name}`: {document}"),
        })
}

/// Registers an account, verifies it out of Mailpit, and answers the session and organization.
fn account(api: &str, mailpit: &str, nonce: &str) -> Result<(String, String), Error> {
    let address = format!("smoke-{nonce}@hook0.local");
    let password = format!("Sk-{nonce}-{nonce}");

    let registered = post(
        "registering an account",
        &format!("{api}/register"),
        None,
        json!({
            "first_name": "Live",
            "last_name": "Smoke",
            "email": address,
            "password": password,
        }),
    )?;
    let organization_id = string(&registered, "organization_id", "registering an account")?;
    println!("== provision: registered {address}, organization {organization_id}");

    let token = verification_token(mailpit, &address)?;
    let session = post(
        "verifying the account's email",
        &format!("{api}/auth/verify-email"),
        None,
        json!({ "token": token }),
    )?;
    let access_token = string(&session, "access_token", "verifying the account's email")?;

    Ok((organization_id, access_token))
}

/// The verification token out of the email the instance really sent.
fn verification_token(mailpit: &str, address: &str) -> Result<String, Error> {
    let deadline = Instant::now() + EMAIL_WITHIN;
    while Instant::now() < deadline {
        let listed = get(
            "listing what Mailpit received",
            &format!("{mailpit}/api/v1/messages?limit={MAX_MESSAGES}"),
        )?;
        let messages = listed
            .get("messages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for message in messages.iter().take(MAX_MESSAGES) {
            let addressed = message
                .get("To")
                .and_then(Value::as_array)
                .map(|to| {
                    to.iter()
                        .any(|one| one.get("Address").and_then(Value::as_str) == Some(address))
                })
                .unwrap_or(false);
            if !addressed {
                continue;
            }
            let id = match message.get("ID").and_then(Value::as_str) {
                Some(id) => id,
                None => continue,
            };
            let body = get(
                "reading the verification email",
                &format!("{mailpit}/api/v1/message/{id}"),
            )?;
            let read = [
                body.get("HTML").and_then(Value::as_str).unwrap_or_default(),
                body.get("Text").and_then(Value::as_str).unwrap_or_default(),
            ]
            .concat();
            if let Some(token) = token_in(&read) {
                return Ok(token);
            }
        }
        std::thread::sleep(EMAIL_EVERY);
    }

    Err(Error::NoVerificationEmail {
        address: address.to_owned(),
        seconds: EMAIL_WITHIN.as_secs(),
    })
}

/// The value of the `token` parameter of the verification link the email carries.
///
/// The longest of every candidate the message holds, rather than the first. The email is sent in
/// two parts and the plain-text one is wrapped at a column, so its copy of the link is cut into
/// pieces; the first `token=` found there is a prefix of the real one, and a prefix verifies as an
/// expired link rather than as a malformed one — which is a confusing way to fail.
fn token_in(message: &str) -> Option<String> {
    const NAMED: &str = "token=";
    let mut longest: Option<&str> = None;

    for (at, _) in message.match_indices(NAMED) {
        let rest = &message[at + NAMED.len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '='))
            .unwrap_or(rest.len());
        let candidate = &rest[..end];
        if candidate.len() > longest.map(str::len).unwrap_or(0) {
            longest = Some(candidate);
        }
    }

    longest.filter(|token| !token.is_empty()).map(str::to_owned)
}

/// Everything the smokes are run against, created through the API.
///
/// `receiver` is where the output worker is told to deliver, which is what makes the signature the
/// clients verify one the instance produced rather than one this harness computed.
pub fn provision(
    api: &str,
    mailpit: &str,
    nonce: &str,
    receiver: &str,
) -> Result<Provisioned, Error> {
    let (organization_id, session) = account(api, mailpit, nonce)?;

    let application = post(
        "creating an application",
        &format!("{api}/applications"),
        Some(&session),
        json!({ "name": format!("smoke-{nonce}"), "organization_id": organization_id }),
    )?;
    let application_id = string(&application, "application_id", "creating an application")?;

    let secret = post(
        "creating an application secret",
        &format!("{api}/application_secrets"),
        Some(&session),
        json!({ "application_id": application_id, "name": "live smoke" }),
    )?;
    let token = string(&secret, "token", "creating an application secret")?;

    let event_type = post(
        "creating an event type",
        &format!("{api}/event_types"),
        Some(&session),
        json!({
            "application_id": application_id,
            "service": "smoke",
            "resource_type": "event",
            "verb": "sent",
        }),
    )?;
    let event_type = string(&event_type, "event_type_name", "creating an event type")?;

    let subscription = post(
        "creating a subscription",
        &format!("{api}/subscriptions"),
        Some(&session),
        json!({
            "application_id": application_id,
            "is_enabled": true,
            "description": "where the live smoke catches what the instance signs",
            "event_types": [event_type],
            "labels": { "smoke": nonce },
            "metadata": {},
            "target": { "type": "http", "method": "POST", "url": receiver, "headers": {} },
        }),
    )?;
    let subscription_secret = string(&subscription, "secret", "creating a subscription")?;

    println!(
        "== provision: application {application_id}, event type {event_type}, subscription pointing at {receiver}"
    );

    Ok(Provisioned {
        application_id,
        token,
        event_type,
        subscription_secret,
    })
}

/// Sends one event, so that the instance signs and delivers a webhook there is something to catch.
///
/// This is the harness's own send rather than a client's: what is being obtained here is a
/// signature the server produced, and taking it from whichever client happened to run first would
/// make eleven of the twelve verify a delivery the twelfth caused.
pub fn emit(api: &str, provisioned: &Provisioned, nonce: &str) -> Result<(), Error> {
    post(
        "sending the event the delivery is caught from",
        &format!("{api}/event"),
        Some(&provisioned.token),
        json!({
            "application_id": provisioned.application_id,
            "event_type": provisioned.event_type,
            "payload": format!("{{\"smoke\":\"{nonce}\"}}"),
            "payload_content_type": "application/json",
            "labels": { "smoke": nonce },
            "occurred_at": "2026-01-01T00:00:00Z",
        }),
    )?;
    Ok(())
}
