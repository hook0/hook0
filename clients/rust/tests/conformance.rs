//! The cases the shared conformance corpus dictates, run against this client.
//!
//! The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every
//! SDK. Nothing below writes down a verdict, a bound or a signature of its own: they are read out of
//! the committed documents and this client is driven against them over a real socket. A case added
//! to the corpus is therefore exercised here without this file being touched, and a verdict changed
//! there fails here until this client agrees with it again.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Where the shared contract sits, from the crate this suite tests.
const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conformance");

/// Largest document of the corpus read back. The corpus is committed, so one above this is one that
/// grew out of shape rather than one somebody meant.
const MAX_CORPUS_BYTES: u64 = 512 * 1024;

/// One document of the shared contract, bounded before it is parsed.
fn corpus(document: &str) -> Value {
    let path = PathBuf::from(CORPUS).join(document);
    let about = fs::metadata(&path).unwrap_or_else(|e| {
        panic!(
            "the shared contract at {} is unreadable: {e}",
            path.display()
        )
    });
    assert!(
        about.len() <= MAX_CORPUS_BYTES,
        "{} is {} bytes long, above the {MAX_CORPUS_BYTES} read back",
        path.display(),
        about.len(),
    );

    let written = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "the shared contract at {} is unreadable: {e}",
            path.display()
        )
    });
    serde_json::from_str(&written).unwrap_or_else(|e| {
        panic!(
            "{} does not read as the contract it is: {e}",
            path.display()
        )
    })
}

/// The entries the corpus carries at `at`, which no case may find empty: a document that classifies
/// nothing would let every case below pass without exercising anything.
fn entries(document: &Value, at: &str) -> Vec<Value> {
    let found = document
        .pointer(at)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("the shared contract carries nothing at `{at}`"));
    assert!(!found.is_empty(), "the shared contract is empty at `{at}`");
    found.to_owned()
}

/// What the corpus wrote at `field` of one entry, as text. A field named with a leading `/` is read
/// as a path through the document rather than as a field of it.
fn text<'a>(entry: &'a Value, field: &str) -> &'a str {
    at(entry, field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("`{field}` is not the text the contract declares"))
}

/// What the corpus wrote at `field` of one entry, as a whole number.
#[cfg(feature = "producer")]
fn number(entry: &Value, field: &str) -> i64 {
    at(entry, field)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("`{field}` is not the number the contract declares"))
}

/// What the corpus wrote at `field` of one entry, as a verdict.
#[cfg(feature = "producer")]
fn flag(entry: &Value, field: &str) -> bool {
    at(entry, field)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("`{field}` is not the verdict the contract declares"))
}

/// Where in one entry a field is read from: a name is a field of it, a path is followed through it.
fn at<'a>(entry: &'a Value, field: &str) -> Option<&'a Value> {
    if field.starts_with('/') {
        entry.pointer(field)
    } else {
        entry.get(field)
    }
}

#[cfg(feature = "producer")]
mod sending {
    use super::{corpus, entries, flag, number, text};
    use actix_web::dev::ServerHandle;
    use actix_web::http::StatusCode;
    use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
    use hook0_client::{
        DEFAULT_MAX_PAYLOAD_BYTES, DEFAULT_MAX_RESPONSE_BYTES, DEFAULT_REQUEST_TIMEOUT, Event,
        Hook0Client, MAX_ATTEMPTS_CAP, MAX_HEAD_BYTES, MAX_HEADER_BYTES, MAX_RESPONSE_HEADERS,
        RetryPolicy,
    };
    use serde_json::{Value, json};
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use url::Url;
    use uuid::Uuid;

    /// The credential every client below is built with, and the one the request document expects to
    /// find on the wire.
    const TOKEN: &str = "token-xyz";

    /// The ID the API answers with once it takes the event.
    const INGESTED_ID: &str = "a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0100";

    /// The schedule a case that is not about waiting spends between attempts.
    const PROMPT_BACKOFF: Duration = Duration::from_millis(5);

    /// The budget the delay cases share. A delay the API names above it is expected to be cut down
    /// to it, so it also bounds what those cases cost.
    const DELAY_BUDGET: Duration = Duration::from_millis(1100);

    /// What a wait may overshoot by before it is read as more than what was asked for: a loopback
    /// round trip, a timer and a scheduler all sit inside it.
    const DELAY_SLACK: Duration = Duration::from_millis(500);

    /// An answer the API is scripted to give: what it says, what it carries beside it, and how long
    /// it sits on it first.
    #[derive(Debug, Clone)]
    struct ScriptedResponse {
        status: u16,
        body: String,
        headers: Vec<(String, String)>,
        held_for: Duration,
    }

    impl ScriptedResponse {
        fn new(status: u16, body: &Value) -> Self {
            Self {
                status,
                body: body.to_string(),
                headers: Vec::new(),
                held_for: Duration::ZERO,
            }
        }

        fn carrying(self, name: &str, value: &str) -> Self {
            let mut headers = self.headers;
            headers.push((name.to_owned(), value.to_owned()));
            Self { headers, ..self }
        }

        /// The same answer, withheld long enough for a client with a shorter timeout to give up.
        fn held_for(self, held_for: Duration) -> Self {
            Self { held_for, ..self }
        }
    }

    /// What the API says when it refuses a request, in the shape every Hook0 failure takes.
    fn refusal(status: u16, problem: &str) -> ScriptedResponse {
        ScriptedResponse::new(
            status,
            &json!({
                "id": problem,
                "status": status,
                "title": "refused",
                "detail": "what the corpus scripted",
                "type": format!("https://hook0.com/documentation/errors/{problem}"),
            }),
        )
    }

    fn ingested() -> ScriptedResponse {
        ScriptedResponse::new(
            201,
            &json!({
                "application_id": Uuid::nil(),
                "event_id": INGESTED_ID,
                "received_at": "2026-01-01T00:00:00Z",
            }),
        )
    }

    /// A request the API received, as it read it off the wire.
    #[derive(Debug, Clone)]
    struct ReceivedRequest {
        headers: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct ApiState {
        received: Arc<Mutex<Vec<ReceivedRequest>>>,
        scripted: Arc<Mutex<Vec<ScriptedResponse>>>,
    }

    async fn ingest(state: web::Data<ApiState>, request: HttpRequest) -> HttpResponse {
        let scripted = match (state.received.lock(), state.scripted.lock()) {
            (Ok(mut received), Ok(mut scripted)) => {
                received.push(ReceivedRequest {
                    headers: request
                        .headers()
                        .iter()
                        .map(|(name, value)| {
                            (
                                name.as_str().to_lowercase(),
                                String::from_utf8_lossy(value.as_bytes()).into_owned(),
                            )
                        })
                        .collect(),
                });
                if scripted.is_empty() {
                    // An unscripted request is one the client should not have issued, and a case
                    // detects it through the request count.
                    refusal(500, "InternalServerError")
                } else {
                    scripted.remove(0)
                }
            }
            _ => refusal(500, "InternalServerError"),
        };

        if !scripted.held_for.is_zero() {
            actix_web::rt::time::sleep(scripted.held_for).await;
        }

        let mut answer = HttpResponse::build(
            StatusCode::from_u16(scripted.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );
        answer.content_type("application/problem+json");
        for (name, value) in &scripted.headers {
            answer.insert_header((name.as_str(), value.as_str()));
        }
        answer.body(scripted.body)
    }

    /// A Hook0 API listening on a loopback port, for the lifetime of one case.
    struct TestApi {
        base_url: Url,
        handle: ServerHandle,
        state: ApiState,
    }

    impl TestApi {
        fn start(scripted: Vec<ScriptedResponse>) -> Self {
            let state = ApiState {
                received: Arc::new(Mutex::new(Vec::new())),
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
            Hook0Client::new(self.base_url.to_owned(), Uuid::nil(), TOKEN)
                .expect("the client accepts a loopback API URL")
                .with_retry_policy(prompt_retries(4))
        }

        fn received(&self) -> Vec<ReceivedRequest> {
            self.state
                .received
                .lock()
                .expect("the recorded requests are readable")
                .clone()
        }

        async fn stop(self) {
            self.handle.stop(true).await;
        }
    }

    /// A retry schedule short enough that a case spends its time on requests rather than on waiting,
    /// and whose budget is far above what its delays add up to.
    fn prompt_retries(max_attempts: u32) -> RetryPolicy {
        RetryPolicy {
            max_attempts,
            initial_backoff: PROMPT_BACKOFF,
            max_backoff: PROMPT_BACKOFF,
            max_total_delay: Duration::from_secs(1),
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

    /// How many attempts a send made, whether it ended up ingesting the event, and what it said.
    ///
    /// A send that reached a server is counted by what that server received. One that never reached
    /// anything — an API URL nothing can be sent to is the corpus's own example — is counted by what
    /// the client says it did, which is also the message a caller is left holding: a misconfiguration
    /// retried four times reads as a network that would not answer.
    async fn issued_by(api: &TestApi, client: Hook0Client) -> (usize, bool, String) {
        match client.send_event(&an_event()).await {
            Ok(_) => (api.received().len(), true, String::new()),
            Err(refused) => {
                let said = refused.to_string();
                (api.received().len().max(attempts_of(&said)), false, said)
            }
        }
    }

    /// How many attempts a client says it made, read out of what it told its caller. A send that
    /// gave up after more than one attempt names the number; one that stopped at its first says
    /// nothing, and made one.
    fn attempts_of(said: &str) -> usize {
        const NAMED: &str = "gave up after ";
        match said.split_once(NAMED) {
            Some((_, rest)) => rest
                .split_whitespace()
                .next()
                .and_then(|attempts| attempts.parse::<usize>().ok())
                .unwrap_or(1),
            None => 1,
        }
    }

    /// How many requests a send made when the API answered that way and then took the event.
    async fn issued_for(scripted: ScriptedResponse) -> (usize, bool) {
        let api = TestApi::start(vec![scripted, ingested()]);
        let (issued, ingested, _) = issued_by(&api, api.client()).await;
        api.stop().await;
        (issued, ingested)
    }

    /// A problem the corpus says is worth repeating and that shares its status with one it says is
    /// not. That is the answer the API names a delay beside, and the one a status alone cannot
    /// classify.
    fn paced_problem() -> Value {
        let contract = corpus("retry.json");
        let problems = entries(&contract, "/problems");

        problems
            .iter()
            .find(|rule| {
                flag(rule, "retryable")
                    && problems.iter().any(|other| {
                        number(other, "status") == number(rule, "status")
                            && !flag(other, "retryable")
                    })
            })
            .cloned()
            .expect("the corpus classifies no problem the API names a delay beside")
    }

    #[actix_web::test]
    async fn every_problem_the_corpus_classifies_is_repeated_as_it_says() {
        // The status is not what decides: the corpus carries problems answering the same status
        // with opposite verdicts, and a client reading the status alone fails half of them.
        let contract = corpus("retry.json");

        for rule in entries(&contract, "/problems") {
            let problem = text(&rule, "problem");
            let status = number(&rule, "status") as u16;
            let (issued, ingested) = issued_for(refusal(status, problem)).await;

            let expected = if flag(&rule, "retryable") { 2 } else { 1 };
            assert_eq!(
                issued,
                expected,
                "`{problem}` under {status} issued {issued} requests where the corpus expects \
                 {expected}: {}",
                text(&rule, "reason"),
            );
            assert_eq!(
                ingested,
                flag(&rule, "retryable"),
                "a send the API refused with `{problem}` ended the wrong way",
            );
        }
    }

    #[actix_web::test]
    async fn every_status_the_corpus_rules_on_is_repeated_as_it_says() {
        // A body naming no problem this client could read is also what an older client meets when
        // the API names a problem it has never heard of.
        let contract = corpus("retry.json");

        for rule in entries(&contract, "/statuses") {
            let status = number(&rule, "status") as u16;
            let (issued, _) =
                issued_for(refusal(status, "AProblemThisClientHasNeverHeardOf")).await;

            let expected = if flag(&rule, "retryable") { 2 } else { 1 };
            assert_eq!(
                issued,
                expected,
                "a status of {status} issued {issued} requests where the corpus expects {expected}: {}",
                text(&rule, "reason"),
            );
        }
    }

    /// Makes this client meet one of the causes the corpus names, for real.
    ///
    /// The API takes the event on its second answer, so a cause the corpus calls retryable is one
    /// the send comes back from, and one it does not is a send that stops at its first attempt.
    async fn provoked(cause: &str) -> (bool, usize) {
        let (api, client) = match cause {
            "no_answer" => {
                let api = TestApi::start(vec![
                    ingested().held_for(Duration::from_millis(500)),
                    ingested(),
                ]);
                let client = api
                    .client()
                    .with_request_timeout(Duration::from_millis(100));
                (api, client)
            }
            "answer_above_a_bound" => {
                let api = TestApi::start(vec![
                    ScriptedResponse::new(200, &json!({ "padding": "x".repeat(1024) })),
                    ingested(),
                ]);
                let client = api.client().with_max_response_bytes(64);
                (api, client)
            }
            "unusable_api_url" => {
                // Nothing listens, and nothing is sent: the URL names no scheme a request could
                // travel on.
                let api = TestApi::start(vec![ingested()]);
                let unreachable = Url::parse("gopher://nowhere.invalid")
                    .expect("a scheme no request travels on is still a URL");
                let client = Hook0Client::new(unreachable, Uuid::nil(), TOKEN)
                    .expect("the client accepts any API URL it is given")
                    .with_retry_policy(prompt_retries(4));
                (api, client)
            }
            _ => panic!(
                "the corpus names the transport cause `{cause}`, which this suite cannot provoke"
            ),
        };

        let (issued, survived, _) = issued_by(&api, client).await;
        api.stop().await;
        (survived, issued)
    }

    #[actix_web::test]
    async fn every_transport_cause_the_corpus_names_ends_a_send_as_it_says() {
        // They arrive as one type in this client as in most runtimes, and only one of them could
        // end differently: a client deciding by the type spends four attempts on a mistyped API URL
        // and then hands its caller a message that accuses the network.
        let contract = corpus("retry.json");

        for rule in entries(&contract, "/transport/causes") {
            let cause = text(&rule, "cause");
            let (survived, attempts) = provoked(cause).await;
            let reason = text(&rule, "reason");

            if flag(&rule, "retryable") {
                assert!(
                    survived,
                    "a send that met `{cause}` gave up after {attempts} attempts where the corpus \
                     says it is retryable: {reason}",
                );
                continue;
            }

            assert!(
                !survived,
                "a send that met `{cause}` reported success: {reason}"
            );
            assert_eq!(
                attempts, 1,
                "`{cause}` was met {attempts} times where the corpus says repeating it changes \
                 nothing: {reason}",
            );
        }
    }

    #[actix_web::test]
    async fn a_head_above_the_ceilings_the_corpus_names_is_refused() {
        // The head is written by the other end, so a client that bounds the body and not the head
        // has only moved where a broken or hostile server spends its caller's memory. Every ceiling
        // is crossed on its own and well over, and the head that is read is well under: the band
        // around a ceiling is where the runtime of the day answers rather than the client, so
        // nothing here is built in it.
        let contract = corpus("bounds.json");
        let bounds = contract
            .get("bounds")
            .expect("the shared contract carries no bounds");
        let lines = number(bounds, "max_response_headers") as usize;
        let per_line = number(bounds, "max_header_bytes") as usize;
        let whole = number(bounds, "max_head_bytes") as usize;

        // Half of every ceiling: a head this size is one a client reads without a word.
        let mut well_under = ingested();
        for index in 0..lines / 4 {
            well_under =
                well_under.carrying(&format!("x-filler-{index}"), &"v".repeat(whole / lines));
        }

        // Above the count this client holds a head to, and below the hundred lines the HTTP stack
        // under it holds one to: past that, the runtime refuses the head before the client sees it,
        // and the case would be reading the runtime rather than the client.
        let mut too_many = ScriptedResponse::new(200, &json!({}));
        for index in 0..lines + 8 {
            too_many = too_many.carrying(&format!("x-filler-{index}"), "filler");
        }
        let too_long =
            ScriptedResponse::new(200, &json!({})).carrying("x-filler", &"v".repeat(per_line * 2));
        // Lines that are neither too many nor too long on their own — an eighth of the whole-head
        // ceiling each, one short of the count — and eight times too much head together.
        let mut too_much = ScriptedResponse::new(200, &json!({}));
        let filling = whole / 8;
        for index in 0..lines - 1 {
            too_much = too_much.carrying(&format!("x-filler-{index}"), &"v".repeat(filling));
        }

        let api = TestApi::start(vec![well_under]);
        let (_, survived, said) = issued_by(&api, api.client()).await;
        assert!(
            survived,
            "an answer whose head is well under every ceiling was refused ({said})",
        );
        api.stop().await;

        for (head, scripted) in [
            ("more header lines than are read", too_many),
            ("a header line longer than is read", too_long),
            ("a whole head above what is read", too_much),
        ] {
            let api = TestApi::start(vec![scripted, ingested()]);
            let (issued, survived, said) = issued_by(&api, api.client()).await;

            assert!(!survived, "an answer whose head carries {head} was read");
            assert_eq!(
                issued, 1,
                "an answer this client will not read was drawn {issued} times ({said})",
            );
            api.stop().await;
        }
    }

    #[actix_web::test]
    async fn every_request_carries_what_the_corpus_says_it_does() {
        // A representation a client forgets to ask for costs nothing until the API serves a second
        // one, at which point it costs everything, which is exactly the kind of divergence nobody
        // notices by hand.
        let contract = corpus("request.json");
        let api = TestApi::start(vec![ingested()]);

        api.client()
            .send_event(&an_event())
            .await
            .expect("the API accepts the event");

        // A send carries a body, so every occasion the corpus declares applies to this one request.
        let received = api.received();
        let carried = &received.first().expect("the send reached the API").headers;

        for header in entries(&contract, "/headers") {
            let name = text(&header, "name").to_lowercase();
            let expected = text(&header, "value").replace("${token}", TOKEN);
            let written = carried.get(&name).map(String::as_str).unwrap_or("");
            assert_eq!(
                written,
                expected,
                "the request carried `{name}: {written}` where the shared contract says \
                 `{expected}`: {}",
                text(&header, "reason"),
            );
        }

        api.stop().await;
    }

    #[actix_web::test]
    async fn the_delay_the_api_names_is_honoured_and_bounded() {
        // The header is written by the other end, so honouring it whole would hand a stranger the
        // length of this client's send. What the corpus asks for is that a delay be waited out when
        // the budget can afford it and cut down to what is left of the budget when it cannot.
        let contract = corpus("retry.json");
        let named = text(&contract, "/retry_after/header");
        let paced = paced_problem();

        for delay in entries(&contract, "/retry_after/cases") {
            let written = text(&delay, "header");
            let api = TestApi::start(vec![
                refusal(number(&paced, "status") as u16, text(&paced, "problem"))
                    .carrying(named, written),
                ingested(),
            ]);
            let patient = api.client().with_retry_policy(RetryPolicy {
                max_attempts: 4,
                initial_backoff: PROMPT_BACKOFF,
                max_backoff: PROMPT_BACKOFF,
                max_total_delay: DELAY_BUDGET,
            });

            let started = Instant::now();
            let sent = patient.send_event(&an_event()).await;
            let waited = started.elapsed();

            assert!(
                sent.is_ok(),
                "the send did not survive a paced answer: {sent:?}"
            );
            assert_eq!(
                api.received().len(),
                2,
                "a paced answer was not retried exactly once",
            );

            let asked_for = if flag(&delay, "honoured") {
                Duration::from_secs(number(&delay, "seconds") as u64).min(DELAY_BUDGET)
            } else {
                Duration::ZERO
            };
            assert!(
                waited >= asked_for,
                "`{named}: {written}` was retried after {waited:?}, sooner than the {asked_for:?} \
                 it asked for",
            );
            assert!(
                waited <= asked_for + DELAY_SLACK,
                "`{named}: {written}` held the send for {waited:?}, above the {asked_for:?} it is \
                 bounded to",
            );

            api.stop().await;
        }
    }

    #[test]
    fn the_bounds_are_the_ones_the_corpus_names() {
        // This client's defaults, held against the one place the numbers are written down. What is
        // asserted is read from the corpus rather than listed here, so a bound added there and left
        // unapplied fails instead of passing unnoticed.
        let contract = corpus("bounds.json");
        let policy = RetryPolicy::default();
        let applied: HashMap<&str, i64> = HashMap::from([
            ("max_attempts", i64::from(policy.max_attempts)),
            ("max_attempts_cap", i64::from(MAX_ATTEMPTS_CAP)),
            (
                "initial_backoff_ms",
                policy.initial_backoff.as_millis() as i64,
            ),
            ("max_backoff_ms", policy.max_backoff.as_millis() as i64),
            (
                "max_total_delay_ms",
                policy.max_total_delay.as_millis() as i64,
            ),
            (
                "request_timeout_ms",
                DEFAULT_REQUEST_TIMEOUT.as_millis() as i64,
            ),
            ("max_payload_bytes", DEFAULT_MAX_PAYLOAD_BYTES as i64),
            ("max_response_bytes", DEFAULT_MAX_RESPONSE_BYTES as i64),
            ("max_head_bytes", MAX_HEAD_BYTES as i64),
            ("max_response_headers", MAX_RESPONSE_HEADERS as i64),
            ("max_header_bytes", MAX_HEADER_BYTES as i64),
        ]);

        let bounds = contract
            .get("bounds")
            .and_then(Value::as_object)
            .expect("the shared contract carries no bounds");
        for (bound, named) in bounds {
            let carried = applied.get(bound.as_str()).unwrap_or_else(|| {
                panic!("the corpus names `{bound}`, which this client applies nowhere")
            });
            assert_eq!(
                Some(*carried),
                named.as_i64(),
                "`{bound}` is {carried} here and {named} in the shared contract",
            );
        }
    }
}

#[cfg(feature = "consumer")]
mod verifying {
    use super::{corpus, entries, text};
    use chrono::DateTime;
    use hook0_client::{Hook0ClientError, verify_webhook_signature_with_current_time};
    use serde_json::Value;
    use std::time::Duration;

    /// How a refusal the corpus names reads in this client's own vocabulary. Every name the corpus
    /// declares is looked up here, so one added there stops this suite until it is mapped rather
    /// than passing under whatever the client happened to answer.
    ///
    /// A code that is not hexadecimal and a code that does not match are both refused as
    /// [`Hook0ClientError::InvalidSignature`]: this client tells its caller that the signature is
    /// invalid without saying which of the two it was, and the variants that would say so are never
    /// returned by the entry point.
    fn refused_as(refusal: &str, error: &Hook0ClientError) -> bool {
        match refusal {
            "code_not_hexadecimal" | "code_mismatch" => {
                matches!(error, Hook0ClientError::InvalidSignature)
            }
            "header_not_delivered" => matches!(error, Hook0ClientError::MissingHeader(_)),
            "outside_tolerance" => matches!(error, Hook0ClientError::ExpiredWebhook { .. }),
            _ => false,
        }
    }

    /// The refusals this suite knows how to read, which every one the corpus declares has to be
    /// among.
    const MAPPED: [&str; 4] = [
        "code_not_hexadecimal",
        "code_mismatch",
        "header_not_delivered",
        "outside_tolerance",
    ];

    #[test]
    fn every_refusal_the_corpus_declares_reads_as_one_of_this_client_s() {
        let contract = corpus("signature.json");

        for refusal in entries(&contract, "/refusals") {
            let refusal = refusal
                .as_str()
                .expect("a refusal the corpus declares is a name")
                .to_owned();
            assert!(
                MAPPED.contains(&refusal.as_str()),
                "the corpus declares the refusal `{refusal}`, which this suite maps to nothing here",
            );
        }
    }

    #[test]
    fn every_delivery_of_the_corpus_is_verified_as_it_says() {
        // A refused delivery has to be refused for the reason the corpus names: a client that
        // computed a code over a header that never arrived and reported a mismatch would otherwise
        // look right.
        let contract = corpus("signature.json");

        for vector in entries(&contract, "/vectors") {
            let name = text(&vector, "name");
            let delivered = vector
                .get("headers")
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("`{name}` carries no headers to deliver"))
                .iter()
                .map(|pair| {
                    let pair = pair
                        .as_array()
                        .filter(|pair| pair.len() == 2)
                        .unwrap_or_else(|| {
                            panic!("a header of `{name}` is not a name and a value")
                        });
                    (
                        pair[0].as_str().unwrap_or_default().to_owned(),
                        pair[1].as_str().unwrap_or_default().to_owned(),
                    )
                })
                .collect::<Vec<_>>();
            let current_time = vector
                .get("current_time")
                .and_then(Value::as_i64)
                .and_then(|moment| DateTime::from_timestamp(moment, 0))
                .unwrap_or_else(|| panic!("`{name}` is held against no readable moment"));
            let tolerance = Duration::from_secs(
                vector
                    .get("tolerance_seconds")
                    .and_then(Value::as_u64)
                    .unwrap_or_else(|| panic!("`{name}` is held within no readable tolerance")),
            );

            let verdict = verify_webhook_signature_with_current_time(
                text(&vector, "signature"),
                text(&vector, "payload").as_bytes(),
                &delivered,
                text(&vector, "secret"),
                tolerance,
                current_time,
            );
            let reason = text(&vector, "reason");

            if text(&vector, "verdict") == "accepted" {
                assert!(
                    verdict.is_ok(),
                    "a delivery the corpus accepts was refused as {verdict:?}: {reason}",
                );
                continue;
            }

            let refusal = text(&vector, "refusal");
            let error = verdict.expect_err(name);
            assert!(
                refused_as(refusal, &error),
                "a delivery the corpus refuses as `{refusal}` was answered `{error}`: {reason}",
            );
        }
    }
}
