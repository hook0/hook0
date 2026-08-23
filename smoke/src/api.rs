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

use std::cell::RefCell;
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

/// How often the instance is asked whether something it does on its own has happened yet.
const HOUSEKEEPING_EVERY: Duration = Duration::from_millis(500);

/// How long a session the API mints stays valid.
///
/// `USER_ACCESS_TOKEN_EXPIRATION` in `api/src/iam.rs`. Restated here because the harness reaches
/// the instance over HTTP and has no way to read a constant out of it; `smoke/tests/cadence.rs`
/// holds the two to the same value.
const SESSION_LASTS: Duration = Duration::from_secs(60 * 5);

/// How much of a session is left unspent, so a call cannot land after the expiry of a session that
/// was still valid when it was picked up.
const SESSION_MARGIN: Duration = Duration::from_secs(60);

/// How long the per-day counts are waited for.
///
/// They come out of a materialized view the API refreshes on a cycle of its own — sixty seconds by
/// default — so an event ingested a moment ago is not in them yet. Three cycles, so one missed
/// refresh is a slow run rather than a failed one.
const COUNTED_WITHIN: Duration = Duration::from_secs(180);

/// The account every application of a run belongs to, and the session that creates them.
///
/// A session lasts five minutes and a run lasts longer than that: twelve languages are provisioned
/// one after another, and each one needs three calls only a user session can make. Nothing a smoke
/// is handed can stand in — an application secret is scoped to one application, and an
/// organization-scoped service token is refused `applications.create` — so the account keeps what
/// it registered with and logs back in when the session in hand is close to its expiry.
#[derive(Debug, Clone)]
pub struct Account {
    pub organization_id: String,
    /// What the account was registered with, kept so a session can be minted again.
    email: String,
    password: String,
    /// What an organization-scoped request is made with. It never reaches a smoke: an SDK
    /// authenticates with an application secret, and handing the clients more than they are meant
    /// to carry would prove the wrong thing.
    session: RefCell<Held>,
}

/// A session and when it was minted, which is what says whether it is still worth using.
#[derive(Debug, Clone)]
struct Held {
    token: String,
    minted: Instant,
}

impl Account {
    /// The session an organization-scoped request is made with, minted again when the one in hand
    /// is close enough to its expiry that the call about to be made could land after it.
    fn session(&self, api: &str) -> Result<String, Error> {
        {
            let held = self.session.borrow();
            if held.minted.elapsed() + SESSION_MARGIN < SESSION_LASTS {
                return Ok(held.token.clone());
            }
        }

        let logged = post(
            "logging the account back in",
            &format!("{api}/auth/login"),
            None,
            json!({ "email": self.email, "password": self.password }),
        )?;
        let token = string(&logged, "access_token", "logging the account back in")?;

        *self.session.borrow_mut() = Held {
            token: token.clone(),
            minted: Instant::now(),
        };
        Ok(token)
    }
}

/// One application, everything a client needs to talk about it, and nothing that belongs to
/// another.
///
/// There is one of these per language rather than one for the run. Deleting an application, an
/// event type and a subscription are operations a client declares, so a smoke has to be able to
/// drive them — and a single shared application would mean the first language to delete it took
/// the eleven behind it with it.
#[derive(Debug, Clone)]
pub struct Application {
    pub application_id: String,
    /// An application secret, which is what an SDK authenticates with.
    pub token: String,
    pub event_type: String,
}

/// One delivery the output worker has finished with, as the ids a client can read it back by.
///
/// Provisioned here rather than left to the smokes because getting one means waiting on the worker,
/// and a wait written twelve times is twelve chances to write it wrong. It is what makes
/// `response.get` an operation a client can be *answered* by: a response row exists only once an
/// attempt has been made, so without this every language could do no better than ask for one that
/// is not there — and the type the API answers it with would never be decoded by anybody.
#[derive(Debug, Clone)]
pub struct Attempted {
    pub request_attempt_id: String,
    pub response_id: String,
}

/// What the delivery every client verifies was produced with.
#[derive(Debug, Clone)]
pub struct Provisioned {
    pub application: Application,
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

/// Reads a document with no credential, or refuses naming the status and the body.
fn get(what: &str, url: &str) -> Result<Value, Error> {
    get_with(what, url, None)
}

/// Reads a document, or refuses naming the status and the body.
fn get_with(what: &str, url: &str, token: Option<&str>) -> Result<Value, Error> {
    let agent = agent();
    let mut request = agent.get(url);
    if let Some(token) = token {
        request = request.header("authorization", &format!("Bearer {token}"));
    }
    let mut answer = request.call().map_err(|cause| Error::Http {
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

/// The body, or a refusal naming the ceiling this harness sets for itself.
///
/// One byte beyond the ceiling is read so that crossing it can be seen. Cutting the body there
/// instead and handing what was read on would make every caller below report that the answer is not
/// JSON — a wrong diagnosis for a bound that was crossed, and one that sends whoever reads it
/// looking at the API rather than at the ceiling.
fn read_body(answer: &mut ureq::http::Response<ureq::Body>, what: &str) -> Result<String, Error> {
    let mut read: Vec<u8> = Vec::new();
    answer
        .body_mut()
        .as_reader()
        .take(MAX_ANSWER_BYTES + 1)
        .read_to_end(&mut read)
        .map_err(|cause| Error::Http {
            what: what.to_owned(),
            cause: format!("the answer could not be read: {cause}"),
        })?;

    if read.len() as u64 > MAX_ANSWER_BYTES {
        return Err(Error::Answer {
            what: what.to_owned(),
            detail: format!("the answer is longer than the {MAX_ANSWER_BYTES} bytes read at most"),
        });
    }

    String::from_utf8(read).map_err(|cause| Error::Http {
        what: what.to_owned(),
        cause: format!("the answer could not be read: {cause}"),
    })
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
pub fn account(api: &str, mailpit: &str, nonce: &str) -> Result<Account, Error> {
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

    Ok(Account {
        organization_id,
        email: address,
        password,
        session: RefCell::new(Held {
            token: access_token,
            minted: Instant::now(),
        }),
    })
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

        // Skipping past the ceiling rather than refusing at it, unlike the body read above: Mailpit
        // was already asked for at most this many and answers newest first, this is a search for one
        // address no other run uses, and it is retried until `EMAIL_WITHIN` runs out. A run that
        // shares a Mailpit with another must not fail because a fifty-first message arrived; a
        // verification email that never turns up is already refused, by name, below the loop.
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

/// One application of its own for one language, created through the API the way a user creates one.
///
/// The event type is named the same way in every application, so the name a smoke is handed is the
/// same string whichever application it was created in.
pub fn application(api: &str, account: &Account, name: &str) -> Result<Application, Error> {
    let held = account.session(api)?;
    let session = Some(held.as_str());

    let application = post(
        "creating an application",
        &format!("{api}/applications"),
        session,
        json!({ "name": name, "organization_id": account.organization_id }),
    )?;
    let application_id = string(&application, "application_id", "creating an application")?;

    let secret = post(
        "creating an application secret",
        &format!("{api}/application_secrets"),
        session,
        json!({ "application_id": application_id, "name": "live smoke" }),
    )?;
    let token = string(&secret, "token", "creating an application secret")?;

    let event_type = post(
        "creating an event type",
        &format!("{api}/event_types"),
        session,
        json!({
            "application_id": application_id,
            "service": "smoke",
            "resource_type": "event",
            "verb": "sent",
        }),
    )?;
    let event_type = string(&event_type, "event_type_name", "creating an event type")?;

    println!("== provision: application {application_id} ({name}), event type {event_type}");

    Ok(Application {
        application_id,
        token,
        event_type,
    })
}

/// An organization-scoped credential, minted through the API the way a user mints one.
///
/// Every SDK authenticates with a bearer token and the API takes two kinds: an application secret,
/// scoped to one application, and this, scoped to the organization. Several operations the document
/// declares are the organization's — listing applications, everything about service tokens, the
/// per-organization event counts — and no application secret can perform them. A smoke holding only
/// one of the two could report them, but only ever as refusals, and the types they answer would
/// never be decoded.
pub fn service_token(api: &str, account: &Account, name: &str) -> Result<String, Error> {
    let held = account.session(api)?;
    let minted = post(
        "creating a service token",
        &format!("{api}/service_token"),
        Some(&held),
        json!({ "name": name, "organization_id": account.organization_id }),
    )?;
    let biscuit = string(&minted, "biscuit", "creating a service token")?;

    println!("== provision: an organization-scoped service token for the smokes to hold as well");
    Ok(biscuit)
}

/// One finished delivery attempt of that application, waited for under the worker's own cadence.
///
/// Asked once, of the application the shared delivery was caught from, because that application has
/// already had a webhook delivered by the time this is called — the run waited for it. Every
/// language then reads the same attempt and the same response back with the organization credential
/// it is handed, so no smoke has to wait on the worker itself.
pub fn attempted(api: &str, account: &Account, application_id: &str) -> Result<Attempted, Error> {
    let deadline = Instant::now() + crate::worker::DELIVERS_WITHIN;
    while Instant::now() < deadline {
        let held = account.session(api)?;
        let listed = get_with(
            "listing the attempts of an application",
            &format!("{api}/request_attempts?application_id={application_id}"),
            Some(&held),
        )?;

        let finished = listed
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .find_map(|attempt| {
                let response = attempt.get("response_id").and_then(Value::as_str)?;
                let id = attempt.get("request_attempt_id").and_then(Value::as_str)?;
                Some(Attempted {
                    request_attempt_id: id.to_owned(),
                    response_id: response.to_owned(),
                })
            });

        if let Some(attempted) = finished {
            println!(
                "== provision: the worker finished attempt {}, so there is a response every client \
                 can read back",
                attempted.request_attempt_id
            );
            return Ok(attempted);
        }
        std::thread::sleep(HOUSEKEEPING_EVERY);
    }

    Err(Error::NoAttempt {
        application: application_id.to_owned(),
        seconds: crate::worker::DELIVERS_WITHIN.as_secs(),
        expectation: crate::worker::expectation(),
    })
}

/// Waits until the organization's per-day event counts are no longer empty.
///
/// Waited for rather than assumed, and waited for once. The counts are a materialized view the API
/// refreshes on a cycle of its own, so an application created a moment ago has none — which would
/// leave every language reporting the operation that reads them and no language ever decoding the
/// type it answers. What is being waited for is the *instance's own housekeeping*, not anything
/// this harness or a client does.
pub fn counted(api: &str, account: &Account) -> Result<(), Error> {
    let deadline = Instant::now() + COUNTED_WITHIN;
    while Instant::now() < deadline {
        let held = account.session(api)?;
        let listed = get_with(
            "listing an organization's events per day",
            &format!(
                "{api}/events_per_day/organization?organization_id={}",
                account.organization_id
            ),
            Some(&held),
        )?;

        if listed.as_array().is_some_and(|counted| !counted.is_empty()) {
            println!(
                "== provision: the instance has refreshed its per-day counts, so what they are \
                 answered with is a type a client can decode"
            );
            return Ok(());
        }
        std::thread::sleep(HOUSEKEEPING_EVERY);
    }

    Err(Error::NoCounts {
        organization: account.organization_id.clone(),
        seconds: COUNTED_WITHIN.as_secs(),
    })
}

/// The application the one shared delivery is caught from, with a subscription pointing at the
/// harness's own socket.
///
/// `receiver` is where the output worker is told to deliver, which is what makes the signature the
/// clients verify one the instance produced rather than one this harness computed.
pub fn provision(
    api: &str,
    account: &Account,
    nonce: &str,
    receiver: &str,
) -> Result<Provisioned, Error> {
    let application = application(api, account, &format!("smoke-{nonce}"))?;

    let held = account.session(api)?;
    let subscription = post(
        "creating a subscription",
        &format!("{api}/subscriptions"),
        Some(&held),
        json!({
            "application_id": application.application_id,
            "is_enabled": true,
            "description": "where the live smoke catches what the instance signs",
            "event_types": [application.event_type],
            "labels": { "smoke": nonce },
            "metadata": {},
            "target": { "type": "http", "method": "POST", "url": receiver, "headers": {} },
        }),
    )?;
    let subscription_secret = string(&subscription, "secret", "creating a subscription")?;

    println!("== provision: subscription pointing at {receiver}");

    Ok(Provisioned {
        application,
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
        Some(&provisioned.application.token),
        json!({
            "application_id": provisioned.application.application_id,
            "event_type": provisioned.application.event_type,
            "payload": format!("{{\"smoke\":\"{nonce}\"}}"),
            "payload_content_type": "application/json",
            "labels": { "smoke": nonce },
            "occurred_at": "2026-01-01T00:00:00Z",
        }),
    )?;
    Ok(())
}
