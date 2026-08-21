#![cfg(feature = "producer")]

//! What a Hook0 API observes when a client sends it an event.
//!
//! Every case below talks to a real HTTP server bound on a loopback port, so what is asserted is
//! what actually went over the wire: how many requests the client issued, the identifier they
//! carried, and what the caller was told at the end. Nothing is stubbed.

use actix_web::dev::ServerHandle;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use hook0_client::{Event, Hook0Client, Hook0ClientError, RetryPolicy, generated};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

/// A retry schedule short enough that a case spends its time on requests rather than on waiting,
/// and whose budget is far above what its delays add up to, so the number of attempts a case
/// observes is the one its policy asked for.
fn prompt_retries(max_attempts: u32) -> RetryPolicy {
    RetryPolicy {
        max_attempts,
        initial_backoff: Duration::from_millis(5),
        max_backoff: Duration::from_millis(5),
        max_total_delay: Duration::from_secs(1),
    }
}

/// An answer the API is scripted to give, and how long it sits on it first.
#[derive(Debug, Clone)]
struct ScriptedResponse {
    status: u16,
    body: Value,
    held_for: Duration,
}

impl ScriptedResponse {
    fn new(status: u16, body: Value) -> Self {
        Self {
            status,
            body,
            held_for: Duration::ZERO,
        }
    }

    /// The same answer, withheld long enough for a client with a shorter timeout to give up on it.
    fn held_for(self, held_for: Duration) -> Self {
        Self { held_for, ..self }
    }
}

/// The answer a request draws when the case scripted none for it: an unscripted request is one the
/// client should not have issued, and a case detects it through the request count.
fn unscripted() -> ScriptedResponse {
    ScriptedResponse::new(500, json!({ "id": "InternalServerError" }))
}

fn ingested(event_id: &Uuid) -> ScriptedResponse {
    ScriptedResponse::new(
        201,
        json!({
            "application_id": Uuid::nil(),
            "event_id": event_id,
            "received_at": "2026-01-01T00:00:00Z",
        }),
    )
}

fn already_ingested() -> ScriptedResponse {
    ScriptedResponse::new(
        409,
        json!({
            "id": "EventAlreadyIngested",
            "title": "Event already Ingested",
            "detail": "This event was previously ingested and recorded inside Hook0 service.",
            "status": 409,
        }),
    )
}

#[derive(Clone)]
struct ApiState {
    /// The body of every request that reached the API, in the order it reached it.
    received: Arc<Mutex<Vec<Value>>>,
    /// What each of those requests carried beside its body, in the same order.
    carried: Arc<Mutex<Vec<HashMap<String, String>>>>,
    /// The answers still to be given, in order.
    scripted: Arc<Mutex<Vec<ScriptedResponse>>>,
}

async fn ingest(
    state: web::Data<ApiState>,
    request: HttpRequest,
    body: web::Json<Value>,
) -> HttpResponse {
    if let Ok(mut carried) = state.carried.lock() {
        carried.push(
            request
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|written| (name.as_str().to_owned(), written.to_owned()))
                })
                .collect(),
        );
    }

    let scripted = {
        match (state.received.lock(), state.scripted.lock()) {
            (Ok(mut received), Ok(mut scripted)) => {
                received.push(body.into_inner());
                if scripted.is_empty() {
                    unscripted()
                } else {
                    scripted.remove(0)
                }
            }
            _ => unscripted(),
        }
    };

    if !scripted.held_for.is_zero() {
        actix_web::rt::time::sleep(scripted.held_for).await;
    }

    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(scripted.status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    )
    .content_type("application/problem+json")
    .body(scripted.body.to_string())
}

/// A Hook0 API listening on a loopback port, for the lifetime of one test.
struct TestApi {
    base_url: Url,
    handle: ServerHandle,
    state: ApiState,
}

impl TestApi {
    /// Starts an API that answers `scripted`, in order, and anything beyond it as unscripted.
    fn start(scripted: Vec<ScriptedResponse>) -> Self {
        let state = ApiState {
            received: Arc::new(Mutex::new(Vec::new())),
            carried: Arc::new(Mutex::new(Vec::new())),
            scripted: Arc::new(Mutex::new(scripted)),
        };

        let listener =
            TcpListener::bind(("127.0.0.1", 0)).expect("a loopback port is available to bind");
        let address = listener
            .local_addr()
            .expect("a bound listener has a local address");

        let server_state = state.clone();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(server_state.clone()))
                .route("/event", web::post().to(ingest))
        })
        .listen(listener)
        .expect("the bound listener is usable as a server socket")
        .workers(1)
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        let base_url = Url::parse(&format!("http://{address}"))
            .expect("a loopback address makes a parsable base URL");

        Self {
            base_url,
            handle,
            state,
        }
    }

    fn client(&self) -> Hook0Client {
        Hook0Client::new(self.base_url.to_owned(), Uuid::nil(), "token")
            .expect("the client accepts a loopback API URL")
    }

    fn received(&self) -> Vec<Value> {
        self.state
            .received
            .lock()
            .expect("the recorded requests are readable")
            .clone()
    }

    fn carried(&self) -> Vec<HashMap<String, String>> {
        self.state
            .carried
            .lock()
            .expect("the recorded headers are readable")
            .clone()
    }

    /// The event ID carried by request number `index`, as the API read it.
    fn event_id_of(&self, index: usize) -> String {
        let received = self.received();
        let body = received.get(index).unwrap_or_else(|| {
            panic!("expected at least {} requests, got {received:?}", index + 1)
        });
        body.get("event_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("request {index} carries no event_id: {body}"))
            .to_owned()
    }

    async fn stop(self) {
        self.handle.stop(true).await;
    }
}

fn an_event() -> Event<'static> {
    Event {
        event_id: None,
        event_type: "service.resource.verb",
        payload: Cow::Borrowed(r#"{"hello":"world"}"#),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![("environment".to_owned(), "test".to_owned())],
    }
}

#[actix_web::test]
async fn a_send_the_api_accepts_at_once_issues_a_single_request() {
    let id = Uuid::now_v7();
    let api = TestApi::start(vec![ingested(&id)]);

    let sent = api
        .client()
        .send_event(&an_event())
        .await
        .expect("the API accepts the event");

    assert_eq!(sent, id);
    assert_eq!(
        api.received().len(),
        1,
        "expected exactly one request, got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn an_event_without_an_id_is_sent_under_one_the_client_generated() {
    let id = Uuid::now_v7();
    let api = TestApi::start(vec![ingested(&id)]);

    api.client()
        .send_event(&an_event())
        .await
        .expect("the API accepts the event");

    let sent_id = api.event_id_of(0);
    assert!(
        Uuid::parse_str(&sent_id).is_ok(),
        "the client sent {sent_id}, which is not a UUID"
    );

    api.stop().await;
}

#[actix_web::test]
async fn an_event_the_caller_gave_an_id_is_sent_under_that_id() {
    let chosen = Uuid::now_v7();
    let api = TestApi::start(vec![ingested(&chosen)]);

    let event = Event {
        event_id: Some(&chosen),
        ..an_event()
    };
    api.client()
        .send_event(&event)
        .await
        .expect("the API accepts the event");

    assert_eq!(api.event_id_of(0), chosen.to_string());

    api.stop().await;
}

#[actix_web::test]
async fn an_attempt_that_runs_out_of_time_is_retried_under_the_same_event_id() {
    let id = Uuid::now_v7();
    let api = TestApi::start(vec![
        ingested(&id).held_for(Duration::from_millis(500)),
        ingested(&id),
    ]);

    let sent = api
        .client()
        .with_retry_policy(prompt_retries(3))
        .with_request_timeout(Duration::from_millis(100))
        .send_event(&an_event())
        .await
        .expect("the second attempt is answered in time");

    assert_eq!(sent, id);
    assert_eq!(
        api.received().len(),
        2,
        "expected the timed-out attempt and its retry, got {:?}",
        api.received()
    );
    assert_eq!(
        api.event_id_of(0),
        api.event_id_of(1),
        "the retry must carry the ID of the attempt it repeats, or Hook0 ingests the event twice"
    );

    api.stop().await;
}

#[actix_web::test]
async fn repeated_server_errors_stop_at_the_configured_number_of_attempts() {
    const ATTEMPTS: u32 = 3;
    let api = TestApi::start(vec![unscripted(), unscripted(), unscripted(), unscripted()]);

    let result = api
        .client()
        .with_retry_policy(prompt_retries(ATTEMPTS))
        .send_event(&an_event())
        .await;

    let error = result.expect_err("every attempt was answered with a server error");
    let message = error.to_string();
    assert!(
        message.contains("gave up after 3 attempts"),
        "the error must name the exhaustion, it says: {message}"
    );
    assert_eq!(
        api.received().len(),
        ATTEMPTS as usize,
        "expected exactly {ATTEMPTS} attempts, got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn an_answer_the_api_would_repeat_is_not_retried() {
    let api = TestApi::start(vec![ScriptedResponse::new(
        429,
        json!({ "id": "TooManyEventsToday", "status": 429 }),
    )]);

    let result = api
        .client()
        .with_retry_policy(prompt_retries(4))
        .send_event(&an_event())
        .await;

    assert!(
        matches!(result, Err(Hook0ClientError::EventSending { .. })),
        "expected the refusal to be reported, got {result:?}"
    );
    assert_eq!(
        api.received().len(),
        1,
        "a quota that is exhausted for the day cannot clear itself between two attempts, got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn a_conflict_on_a_retry_reports_the_event_an_earlier_attempt_ingested() {
    let api = TestApi::start(vec![unscripted(), already_ingested()]);

    let sent = api
        .client()
        .with_retry_policy(prompt_retries(3))
        .send_event(&an_event())
        .await
        .expect("the ID was already ingested by the attempt this one repeats");

    assert_eq!(
        sent.to_string(),
        api.event_id_of(0),
        "the ID reported to the caller is the one the send used throughout"
    );
    assert_eq!(
        api.received().len(),
        2,
        "expected the failed attempt and its retry, got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn a_conflict_on_a_first_attempt_reports_the_conflict() {
    let api = TestApi::start(vec![already_ingested()]);

    let result = api
        .client()
        .with_retry_policy(prompt_retries(3))
        .send_event(&an_event())
        .await;

    let error = result.expect_err("nothing this send did can explain the conflict");
    assert!(
        error.to_string().contains("EventAlreadyIngested"),
        "the caller must hear what Hook0 refused, the error says: {error}"
    );
    assert_eq!(
        api.received().len(),
        1,
        "expected exactly one request, got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn a_client_that_does_not_retry_issues_a_single_request() {
    let api = TestApi::start(vec![unscripted(), unscripted(), unscripted()]);

    let result = api
        .client()
        .with_retry_policy(RetryPolicy::disabled())
        .send_event(&an_event())
        .await;

    assert!(result.is_err(), "the API answered a server error");
    assert_eq!(
        api.received().len(),
        1,
        "expected exactly one request, got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn a_payload_above_the_maximum_is_refused_before_any_request_is_issued() {
    const MAXIMUM: usize = 16;
    let api = TestApi::start(vec![ingested(&Uuid::now_v7())]);

    let event = Event {
        payload: Cow::Owned("x".repeat(MAXIMUM + 1)),
        ..an_event()
    };
    let result = api
        .client()
        .with_max_payload_bytes(MAXIMUM)
        .send_event(&event)
        .await;

    let error = result.expect_err("the payload is larger than the client accepts");
    let message = error.to_string();
    assert!(
        message.contains(&format!("{} bytes this client sends at most", MAXIMUM)),
        "the error must name the bound it refused against, it says: {message}"
    );
    assert!(
        api.received().is_empty(),
        "nothing must reach the API, it got {:?}",
        api.received()
    );

    api.stop().await;
}

#[actix_web::test]
async fn the_delays_of_one_send_stay_inside_the_configured_budget() {
    /// Delays alone would add up to nine draws of up to 300 ms; the budget below lets through 300 ms
    /// of them in total.
    const ATTEMPTS: u32 = 10;
    const BUDGET: Duration = Duration::from_millis(300);
    /// What ten requests to a loopback socket, and the work around them, are allowed to cost on top
    /// of the delays.
    const ROUND_TRIP_ALLOWANCE: Duration = Duration::from_millis(300);

    let api = TestApi::start(Vec::new());
    let client = api.client().with_retry_policy(RetryPolicy {
        max_attempts: ATTEMPTS,
        initial_backoff: BUDGET,
        max_backoff: BUDGET,
        max_total_delay: BUDGET,
    });

    let started = Instant::now();
    let result = client.send_event(&an_event()).await;
    let elapsed = started.elapsed();

    assert!(result.is_err(), "every attempt was answered with an error");
    assert!(
        elapsed < BUDGET + ROUND_TRIP_ALLOWANCE,
        "the send took {elapsed:?}, more than its {BUDGET:?} of delay plus {ROUND_TRIP_ALLOWANCE:?} of round trips"
    );
    let attempts = api.received().len();
    assert!(
        (2..=ATTEMPTS as usize).contains(&attempts),
        "expected the send to retry and to stop at {ATTEMPTS} attempts, it made {attempts}"
    );

    api.stop().await;
}

/// An event read back out of the API and sent on again, with its labels carried across.
///
/// This is the shape every forwarder, replayer and migration script has: read an event, build the
/// next one from it. The document describes a label differently on each side, so this is also what
/// that difference costs a caller. Reading gives a free-form object, because a stored row is only
/// held to being an object and older rows were written when a value could be any JSON at all;
/// sending takes a map of string to string, because that is what the ingestion endpoint has
/// accepted since it was tightened. A forwarder therefore converts, and the conversion is the
/// thing that can fail: an event whose labels the ingestion endpoint did not write does not
/// survive the trip, and it fails here rather than being silently reshaped.
#[actix_web::test]
async fn labels_read_off_an_event_are_carried_into_the_next_one() {
    let read: generated::Event = serde_json::from_value(json!({
        "event_id": Uuid::nil(),
        "event_type_name": "service.resource.verb",
        "ip": "127.0.0.1",
        "labels": {"environment": "test", "tenant": "acme"},
        "occurred_at": "2026-01-01T00:00:00Z",
        "payload_content_type": "application/json",
        "received_at": "2026-01-01T00:00:00Z",
    }))
    .expect("an event as the API serializes it parses into the model generated from the document");

    let carried: std::collections::HashMap<String, String> =
        serde_json::from_value(read.labels.clone())
            .expect("the labels of an event the ingestion endpoint wrote are strings");

    let posted = generated::EventPost {
        application_id: Uuid::nil(),
        event_id: None,
        event_type: read.event_type_name.clone(),
        labels: carried.clone(),
        metadata: None,
        occurred_at: read.occurred_at,
        payload: r#"{"hello":"world"}"#.to_owned(),
        payload_content_type: read.payload_content_type.clone(),
    };
    assert_eq!(
        serde_json::to_value(&posted.labels).expect("a map of strings serializes"),
        read.labels,
        "the labels of the event posted back are the ones read off the first"
    );

    let id = Uuid::now_v7();
    let api = TestApi::start(vec![ingested(&id)]);

    api.client()
        .send_event(&Event {
            event_id: None,
            event_type: &posted.event_type,
            payload: Cow::Borrowed(posted.payload.as_str()),
            payload_content_type: &posted.payload_content_type,
            metadata: None,
            occurred_at: None,
            labels: carried.into_iter().collect(),
        })
        .await
        .expect("the API accepts the event built from the one that was read");

    let received = api.received();
    let body = received
        .first()
        .unwrap_or_else(|| panic!("expected one request, got {received:?}"));
    assert_eq!(
        body.get("labels"),
        Some(&json!({"environment": "test", "tenant": "acme"})),
        "the labels reached the API as they were read, in {body}"
    );

    api.stop().await;
}

#[actix_web::test]
async fn a_policy_at_the_edges_of_its_type_still_states_four_integers() {
    // A Duration reaches far past any delay a caller means, and a count of attempts reaches past
    // what this client will ever make. Neither is a policy anybody configures on purpose, and both
    // are what a header composed out of arithmetic gets wrong: what has to hold is that the value
    // stays four integers a reader can cut apart, rather than a panic or a word.
    let id = Uuid::now_v7();
    let api = TestApi::start(vec![ingested(&id)]);

    api.client()
        .with_retry_policy(RetryPolicy {
            max_attempts: u32::MAX,
            initial_backoff: Duration::MAX,
            max_backoff: Duration::MAX,
            max_total_delay: Duration::MAX,
        })
        .send_event(&an_event())
        .await
        .expect("the API accepts the event");

    let carried = api.carried();
    let stated = carried
        .first()
        .and_then(|headers| headers.get("hook0-client-options"))
        .unwrap_or_else(|| panic!("the send carried no options header, in {carried:?}"));

    for part in stated.split(',') {
        let (name, written) = part
            .split_once('=')
            .unwrap_or_else(|| panic!("`{part}` names nothing, in `{stated}`"));
        assert!(
            !written.is_empty() && written.bytes().all(|byte| byte.is_ascii_digit()),
            "`{name}` states `{written}`, which is no whole number of its own, in `{stated}`",
        );
    }

    api.stop().await;
}
