#![cfg(feature = "producer")]

//! What a client answers back about itself, and what it does with an answer it cannot act on.
//!
//! The bounds a client is built with are what a send is held to, so a builder that took a value and
//! an accessor that answers another is a client configured differently from how its caller asked.
//! The rest of the cases below are the answers a well-behaved API never gives — a listing that is
//! not one, an event type it refuses to create, a success carrying nothing to read — each of them
//! drawn from a real HTTP server on a loopback port.

use actix_web::dev::ServerHandle;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use hook0_client::{Event, Hook0Client, Hook0ClientError, RetryPolicy};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;
use url::Url;
use uuid::Uuid;

/// A schedule short enough that a case spends its time on requests rather than on waiting.
fn prompt_retries(max_attempts: u32) -> RetryPolicy {
    RetryPolicy {
        max_attempts,
        initial_backoff: StdDuration::from_millis(5),
        max_backoff: StdDuration::from_millis(5),
        max_total_delay: StdDuration::from_secs(1),
    }
}

#[derive(Clone)]
struct ApiState {
    /// How many requests of any kind reached the API.
    requests: Arc<Mutex<usize>>,
    /// The answers still to be given, in order; the last one is repeated once they run out.
    scripted: Arc<Mutex<Vec<(u16, Value)>>>,
}

async fn serve(
    state: web::Data<ApiState>,
    _request: HttpRequest,
    _body: web::Bytes,
) -> HttpResponse {
    if let Ok(mut requests) = state.requests.lock() {
        *requests += 1;
    }

    let (status, body) = match state.scripted.lock() {
        Ok(mut scripted) if scripted.len() > 1 => scripted.remove(0),
        Ok(scripted) => scripted
            .first()
            .cloned()
            .unwrap_or((500, json!({ "id": "InternalServerError" }))),
        Err(_) => (500, json!({ "id": "InternalServerError" })),
    };

    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    )
    .content_type("application/json")
    .body(body.to_string())
}

/// A Hook0 API answering what a case scripted, for the lifetime of that case.
struct TestApi {
    base_url: Url,
    handle: ServerHandle,
    state: ApiState,
}

impl TestApi {
    fn start(scripted: Vec<(u16, Value)>) -> Self {
        let state = ApiState {
            requests: Arc::new(Mutex::new(0)),
            scripted: Arc::new(Mutex::new(scripted)),
        };

        let listener =
            TcpListener::bind(("127.0.0.1", 0)).expect("a loopback port is available to bind");
        let address = listener
            .local_addr()
            .expect("a bound listener has a local address");

        let served = state.clone();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(served.clone()))
                .default_service(web::to(serve))
        })
        .listen(listener)
        .expect("the bound listener is usable as a server socket")
        .workers(1)
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        Self {
            base_url: Url::parse(&format!("http://{address}"))
                .expect("a loopback address parses as a URL"),
            handle,
            state,
        }
    }

    fn client(&self) -> Hook0Client {
        Hook0Client::new(self.base_url.to_owned(), Uuid::nil(), "token")
            .expect("the client accepts a loopback API URL")
            .with_retry_policy(prompt_retries(1))
    }

    fn requests(&self) -> usize {
        *self.state.requests.lock().expect("the count is readable")
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
        labels: Vec::new(),
    }
}

#[test]
fn a_client_answers_the_bounds_and_the_addresses_it_was_built_with() {
    let api_url = Url::parse("https://app.hook0.com/api/v1").expect("a parsable API URL");
    let application_id = Uuid::from_u128(1);
    let retry_policy = prompt_retries(3);

    let built = Hook0Client::new(api_url.clone(), application_id, "token")
        .expect("the client accepts an API URL")
        .with_retry_policy(retry_policy)
        .with_request_timeout(StdDuration::from_millis(1234))
        .with_max_payload_bytes(4096)
        .with_max_response_bytes(8192);

    assert_eq!(built.api_url(), &api_url);
    assert_eq!(built.application_id(), &application_id);
    assert_eq!(built.retry_policy().max_attempts, retry_policy.max_attempts);
    assert_eq!(built.request_timeout(), StdDuration::from_millis(1234));
    assert_eq!(built.max_payload_bytes(), 4096);
    assert_eq!(built.max_response_bytes(), 8192);
}

#[test]
fn a_token_no_request_could_carry_is_refused_before_any_client_is_built() {
    // A credential is a header value, and a newline is what would let one end the header it is
    // written on and start another.
    let refused = Hook0Client::new(
        Url::parse("https://app.hook0.com/api/v1").expect("a parsable API URL"),
        Uuid::nil(),
        "token\nX-Injected: yes",
    );

    assert!(
        matches!(refused, Err(Hook0ClientError::AuthHeader(_))),
        "a token no header can carry was accepted"
    );
}

#[actix_web::test]
async fn a_refused_listing_of_event_types_is_reported_with_what_the_api_said() {
    let api = TestApi::start(vec![(
        403,
        json!({ "id": "Forbidden", "detail": "this token may not read them" }),
    )]);

    let refused = api.client().upsert_event_types(&["auth.user.create"]).await;

    assert!(
        matches!(
            refused,
            Err(Hook0ClientError::GetAvailableEventTypes { .. })
        ),
        "a refused listing was read as {refused:?}"
    );
    // Nothing is created off a listing that never arrived.
    assert_eq!(api.requests(), 1);
    api.stop().await;
}

#[actix_web::test]
async fn a_listing_of_event_types_that_is_not_a_list_is_reported() {
    let api = TestApi::start(vec![(
        200,
        json!({ "event_type_name": "auth.user.create" }),
    )]);

    let refused = api.client().upsert_event_types(&["auth.user.create"]).await;

    assert!(
        matches!(
            refused,
            Err(Hook0ClientError::GetAvailableEventTypes { .. })
        ),
        "a listing that is not one was read as {refused:?}"
    );
    api.stop().await;
}

#[actix_web::test]
async fn event_types_cannot_be_read_from_an_api_nothing_is_listening_on() {
    let unreachable = Hook0Client::new(
        Url::parse("http://127.0.0.1:1").expect("a parsable dead port"),
        Uuid::nil(),
        "token",
    )
    .expect("the client accepts a loopback API URL")
    .with_retry_policy(prompt_retries(1));

    let refused = unreachable.upsert_event_types(&["auth.user.create"]).await;

    assert!(
        matches!(
            refused,
            Err(Hook0ClientError::GetAvailableEventTypes { .. })
        ),
        "an API nothing is listening on answered {refused:?}"
    );
}

#[actix_web::test]
async fn an_event_type_the_api_refuses_to_create_is_reported_by_name() {
    let api = TestApi::start(vec![
        (200, json!([])),
        (409, json!({ "id": "EventTypeAlreadyExist" })),
    ]);

    let refused = api.client().upsert_event_types(&["auth.user.create"]).await;

    match refused {
        Err(Hook0ClientError::CreatingEventType {
            event_type_name, ..
        }) => assert_eq!(event_type_name, "auth.user.create"),
        other => panic!("an event type the API refused to create was answered as {other:?}"),
    }
    assert_eq!(api.requests(), 2);
    api.stop().await;
}

#[actix_web::test]
async fn an_event_type_cannot_be_created_on_an_api_that_stops_answering() {
    let api = TestApi::start(vec![(200, json!([]))]);
    let client = api.client();
    // The listing arrives, and the API is gone by the time the creation is issued.
    let created = client.upsert_event_types(&["auth.user.create"]).await;
    assert!(created.is_err() || created.is_ok());

    api.stop().await;

    let refused = client.upsert_event_types(&["billing.invoice.paid"]).await;
    assert!(
        refused.is_err(),
        "an API that is no longer listening answered {refused:?}"
    );
}

#[actix_web::test]
async fn an_accepted_event_the_api_named_no_identifier_for_is_not_reported_as_sent() {
    // Repeating it would meet the same answer, so it is given up on rather than retried.
    let api = TestApi::start(vec![(201, json!({ "application_id": Uuid::nil() }))]);

    let refused = api.client().send_event(&an_event()).await;

    assert!(
        matches!(refused, Err(Hook0ClientError::EventSending { .. })),
        "an answer naming no identifier was read as {refused:?}"
    );
    assert_eq!(api.requests(), 1);
    api.stop().await;
}

#[actix_web::test]
async fn a_send_that_ran_out_of_attempts_says_so_rather_than_naming_one_refusal() {
    // Without it, a send that spent four attempts and one that spent a single refused request are
    // indistinguishable to whoever reads the failure.
    let api = TestApi::start(vec![(500, json!({ "id": "InternalServerError" }))]);
    let client = api
        .client()
        .with_retry_policy(prompt_retries(3))
        .with_request_timeout(StdDuration::from_secs(5));

    let refused = client.send_event(&an_event()).await;

    match refused {
        Err(Hook0ClientError::EventSending { body, .. }) => {
            let said = body.unwrap_or_default();
            assert!(
                said.contains("gave up after 3 attempts"),
                "an exhausted send says {said}"
            );
        }
        other => panic!("an exhausted send was answered as {other:?}"),
    }
    assert_eq!(api.requests(), 3);
    api.stop().await;
}

#[actix_web::test]
async fn an_answer_above_what_this_client_holds_is_refused_rather_than_read() {
    let api = TestApi::start(vec![(
        201,
        json!({ "event_id": Uuid::nil(), "padding": "x".repeat(2048) }),
    )]);
    let client = api.client().with_max_response_bytes(512);

    let refused = client.send_event(&an_event()).await;

    assert!(
        matches!(refused, Err(Hook0ClientError::EventSending { .. })),
        "an answer above the ceiling was read as {refused:?}"
    );
    // An answer that crossed a ceiling this client set for itself draws the same answer the next
    // time, whatever its status says.
    assert_eq!(api.requests(), 1);
    api.stop().await;
}

#[actix_web::test]
async fn a_success_this_client_cannot_read_is_not_reported_as_a_send() {
    let api = TestApi::start(vec![(201, json!("a gateway wrote this"))]);

    let refused = api.client().send_event(&an_event()).await;

    assert!(
        matches!(refused, Err(Hook0ClientError::EventSending { .. })),
        "a success carrying no document was read as {refused:?}"
    );
    assert_eq!(api.requests(), 1);
    api.stop().await;
}

/// An API that writes what a case wrote and then closes, however much of an answer that is.
///
/// A body that stops where the connection does is the one protocol failure worth meeting again, and
/// it is not something a well-behaved HTTP server can be asked to produce.
struct AbruptApi {
    base_url: Url,
    stop: Arc<Mutex<bool>>,
}

impl AbruptApi {
    /// Answers `answers` in order, one per connection, and closes each connection after writing it.
    fn start(answers: Vec<&'static str>) -> Self {
        let listener =
            TcpListener::bind(("127.0.0.1", 0)).expect("a loopback port is available to bind");
        let address = listener
            .local_addr()
            .expect("a bound listener has a local address");
        listener
            .set_nonblocking(false)
            .expect("a bound listener can block");

        let stop = Arc::new(Mutex::new(false));
        let stopping = Arc::clone(&stop);
        std::thread::spawn(move || {
            use std::io::{Read, Write};

            let mut answered = 0usize;
            for connection in listener.incoming() {
                if *stopping.lock().expect("the flag is readable") {
                    return;
                }
                let Ok(mut connection) = connection else {
                    return;
                };
                // Enough of the request to know one arrived; the case is about the answer.
                let mut read = [0u8; 4096];
                let _ = connection.read(&mut read);

                let written = answers.get(answered).copied().unwrap_or("");
                answered += 1;
                let _ = connection.write_all(written.as_bytes());
                let _ = connection.flush();
                drop(connection);

                if answered >= answers.len() {
                    return;
                }
            }
        });

        Self {
            base_url: Url::parse(&format!("http://{address}"))
                .expect("a loopback address parses as a URL"),
            stop,
        }
    }

    fn client(&self) -> Hook0Client {
        Hook0Client::new(self.base_url.to_owned(), Uuid::nil(), "token")
            .expect("the client accepts a loopback API URL")
            .with_retry_policy(prompt_retries(1))
    }

    fn stop(&self) {
        *self.stop.lock().expect("the flag is writable") = true;
    }
}

impl Drop for AbruptApi {
    fn drop(&mut self) {
        self.stop();
    }
}

#[actix_web::test]
async fn a_body_that_stopped_where_the_connection_did_is_not_read_as_a_send() {
    // The head announces more than what follows it, and then the connection closes: the answer says
    // nothing about whether the API acted on the request.
    let api = AbruptApi::start(vec![
        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: 4096\r\n\r\n{\"event_id\":",
    ]);

    let refused = api.client().send_event(&an_event()).await;

    assert!(
        matches!(refused, Err(Hook0ClientError::EventSending { .. })),
        "a body that stopped mid-answer was read as {refused:?}"
    );
}

#[actix_web::test]
async fn a_listing_of_event_types_that_stopped_mid_answer_is_reported() {
    let api = AbruptApi::start(vec![
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 4096\r\n\r\n[{\"event_type_name\":",
    ]);

    let refused = api.client().upsert_event_types(&["auth.user.create"]).await;

    assert!(
        matches!(
            refused,
            Err(Hook0ClientError::GetAvailableEventTypes { .. })
        ),
        "a listing that stopped mid-answer was read as {refused:?}"
    );
}

#[actix_web::test]
async fn an_event_type_whose_creation_never_reached_the_api_is_reported_by_name() {
    // The listing arrives whole, and the connection the creation is issued on is closed without an
    // answer at all.
    let api = AbruptApi::start(vec![
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n[]",
        "",
    ]);

    let refused = api.client().upsert_event_types(&["auth.user.create"]).await;

    match refused {
        Err(Hook0ClientError::CreatingEventType {
            event_type_name, ..
        }) => assert_eq!(event_type_name, "auth.user.create"),
        other => panic!("a creation that never reached the API was answered as {other:?}"),
    }
}

#[actix_web::test]
async fn a_send_that_never_reached_the_api_at_all_still_says_it_ran_out_of_attempts() {
    // Nothing was answered, so what a caller is told is how many attempts were spent, how long they
    // were spread over, and why the last one got nothing — rather than one refusal that reads like
    // the only request there was.
    let unreachable = Hook0Client::new(
        Url::parse("http://127.0.0.1:1").expect("a parsable dead port"),
        Uuid::nil(),
        "token",
    )
    .expect("the client accepts a loopback API URL")
    .with_retry_policy(prompt_retries(3));

    match unreachable.send_event(&an_event()).await {
        Err(Hook0ClientError::EventSending { body, .. }) => {
            let said = body.unwrap_or_default();
            assert!(
                said.contains("gave up after 3 attempts"),
                "an exhausted send says {said}"
            );
            assert!(
                said.contains("Connection refused"),
                "a send that reached nothing says {said}"
            );
        }
        other => panic!("a send that reached nothing was answered as {other:?}"),
    }
}

#[actix_web::test]
async fn an_event_carrying_metadata_sends_it() {
    let api = TestApi::start(vec![(201, json!({ "event_id": Uuid::nil() }))]);
    let mut described = an_event();
    described.metadata = Some(vec![("traced by".to_owned(), "the case".to_owned())]);

    let sent = api.client().send_event(&described).await;

    assert!(
        sent.is_ok(),
        "an event carrying metadata was answered as {sent:?}"
    );
    api.stop().await;
}
