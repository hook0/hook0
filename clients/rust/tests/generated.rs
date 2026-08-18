#![cfg(feature = "producer")]

//! What the generated request layer puts on the wire, and what it does with what comes back.
//!
//! The generated half is handed a transport and nothing else, so these cases hand it a real one —
//! `reqwest`, against an API listening on a loopback port — and watch a real API answer: the path it
//! interpolated, the query it assembled, the body it sent, the value it read back, and the error it
//! answered when the answer was a problem, when it was unreadable, and when nothing was listening.
//!
//! Rust has no way to be asked what a module declares, so the calls below are written out. What
//! they are held to is not: every one of them is looked up in the API document the generator was
//! run against, the request is compared against what that document declares, the answer the API
//! gives is built out of the schema the operation declares it answers, and the last case says that
//! every operation the document declares was reached. An operation the API grows therefore fails
//! this suite until it is driven here, rather than going unnoticed.

use actix_web::dev::ServerHandle;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use hook0_client::generated;
use hook0_client::generated::{RequestError, Transport};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use url::Url;

/// What every string-shaped argument is given. It carries the two characters a path segment may not
/// leave as they are, so a value reaching a path proves it was escaped rather than pasted.
const A_STRING: &str = "a value/with a space";

/// No repository nests this crate deeper than this below its root; the bound turns a checkout that
/// is missing the API document into a failure rather than a walk up to the root of the filesystem.
const MAX_ANCESTORS: usize = 8;

/// No schema the API declares nests anywhere near this deep. The bound turns a document that
/// describes itself into a failure instead of a recursion that never returns.
const MAX_DEPTH: usize = 8;

/// The tag that marks an operation as part of the surface an SDK exposes. A document that marks none
/// of its operations with it declares the whole of itself public, which is the rule the generator
/// applies and therefore the rule this suite holds it to.
const PUBLIC_TAG: &str = "public";

/// The methods a request line can carry, which is what tells an operation apart from the rest of
/// what a path item holds.
const VERBS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// What the API answers under, and what the answer is held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// A success carrying the document the operation declares it answers, with every member the
    /// document lets the API leave out either written or left out.
    ReadsBack { optionals: bool },
    /// A problem the API reports, which every operation answers as one rather than as a value.
    Refuses,
    /// A success carrying something the operation does not declare, which is not read as if it did.
    Unreadable,
    /// Nothing listening at all, which no operation answers a value out of.
    Unreachable,
}

/// One operation the API document declares, as a request has to look to be it.
#[derive(Debug, Clone)]
struct Declared {
    method: String,
    /// The path with its parameters still written the way the document writes them, `{like_this}`.
    template: String,
    required_query: BTreeSet<String>,
    optional_query: BTreeSet<String>,
    /// The schema a success carries, absent when the operation answers no content at all.
    answers: Option<Value>,
    /// The schema the body of the request carries, absent when the operation reads no body.
    reads: Option<Value>,
}

impl Declared {
    /// Whether a request line landed on this operation.
    fn matches(&self, path: &str) -> bool {
        let wanted: Vec<&str> = self.template.split('/').collect();
        let got: Vec<&str> = path.split('/').collect();
        if wanted.len() != got.len() {
            return false;
        }
        wanted.iter().zip(got.iter()).all(|(declared, sent)| {
            if declared.starts_with('{') && declared.ends_with('}') {
                // A parameter stands for a segment that is there; an empty one is the trailing
                // slash of another path rather than a value.
                !sent.is_empty()
            } else {
                declared == sent
            }
        })
    }

    fn query(&self, optionals: bool) -> BTreeSet<String> {
        let mut wanted = self.required_query.clone();
        if optionals {
            wanted.extend(self.optional_query.iter().cloned());
        }
        wanted
    }
}

/// A request the API received.
#[derive(Debug, Clone)]
struct Received {
    method: String,
    path: String,
    query: String,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Clone)]
struct ApiState {
    received: Arc<Mutex<Vec<Received>>>,
    answer: Arc<Mutex<(u16, Value)>>,
}

async fn serve(state: web::Data<ApiState>, request: HttpRequest, body: web::Bytes) -> HttpResponse {
    if let Ok(mut received) = state.received.lock() {
        received.push(Received {
            method: request.method().as_str().to_owned(),
            path: request.path().to_owned(),
            query: request.query_string().to_owned(),
            body: String::from_utf8_lossy(&body).into_owned(),
            headers: request
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|written| (name.as_str().to_owned(), written.to_owned()))
                })
                .collect(),
        });
    }

    let (status, answered) = match state.answer.lock() {
        Ok(answer) => answer.clone(),
        Err(_) => (500, json!({ "id": "InternalServerError" })),
    };

    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    )
    .content_type("application/json")
    .body(answered.to_string())
}

/// The API document the generator was run against, out of the repository holding it.
fn api_document() -> Value {
    let mut at = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..MAX_ANCESTORS {
        let candidate = at.join("api").join("openapi.snapshot.json");
        if candidate.is_file() {
            let read = std::fs::read(&candidate).expect("the API document is readable");
            return serde_json::from_slice(&read).expect("the API document is JSON");
        }
        at = at.join("..");
    }
    panic!("no `api/openapi.snapshot.json` within {MAX_ANCESTORS} directories of this crate");
}

/// Every operation an SDK is built out of, which is what the document marks as public.
fn declared_operations(document: &Value) -> BTreeMap<String, Declared> {
    let paths = document
        .get("paths")
        .and_then(Value::as_object)
        .expect("the API document declares paths");

    let mut all: BTreeMap<String, Declared> = BTreeMap::new();
    let mut public: BTreeMap<String, Declared> = BTreeMap::new();

    for (template, item) in paths {
        for verb in VERBS {
            let Some(operation) = item.get(verb) else {
                continue;
            };
            let Some(id) = operation.get("operationId").and_then(Value::as_str) else {
                continue;
            };

            let mut required_query = BTreeSet::new();
            let mut optional_query = BTreeSet::new();
            if let Some(parameters) = operation.get("parameters").and_then(Value::as_array) {
                for parameter in parameters {
                    if parameter.get("in").and_then(Value::as_str) != Some("query") {
                        continue;
                    }
                    let Some(name) = parameter.get("name").and_then(Value::as_str) else {
                        continue;
                    };
                    if parameter.get("required") == Some(&Value::Bool(true)) {
                        required_query.insert(name.to_owned());
                    } else {
                        optional_query.insert(name.to_owned());
                    }
                }
            }

            let answers = operation
                .get("responses")
                .and_then(Value::as_object)
                .and_then(|answers| {
                    answers
                        .iter()
                        .filter(|(status, _)| status.starts_with('2'))
                        .find_map(|(_, answer)| {
                            answer.pointer("/content/application~1json/schema").cloned()
                        })
                });

            let reads = operation
                .pointer("/requestBody/content/application~1json/schema")
                .cloned();

            let declared = Declared {
                method: verb.to_uppercase(),
                template: (*template).to_owned(),
                required_query,
                optional_query,
                answers,
                reads,
            };

            if operation
                .get("tags")
                .and_then(Value::as_array)
                .is_some_and(|tags| tags.iter().any(|tag| tag.as_str() == Some(PUBLIC_TAG)))
            {
                public.insert(id.to_owned(), declared.clone());
            }
            all.insert(id.to_owned(), declared);
        }
    }

    assert!(!all.is_empty(), "the API document declares no operation");
    if public.is_empty() { all } else { public }
}

/// One document of the shape a schema describes, with every member it does not require either
/// written or left out.
fn document_for(document: &Value, schema: &Value, optionals: bool, depth: usize) -> Value {
    assert!(
        depth <= MAX_DEPTH,
        "{schema} nests more than {MAX_DEPTH} deep"
    );

    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let named = reference
            .rsplit('/')
            .next()
            .expect("a reference names something");
        let resolved = document
            .pointer(&format!("/components/schemas/{named}"))
            .unwrap_or_else(|| panic!("the document declares no schema called `{named}`"));
        return document_for(document, resolved, optionals, depth + 1);
    }

    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        return values
            .first()
            .cloned()
            .expect("a closed list declares a value");
    }

    match schema.get("type").and_then(Value::as_str) {
        Some("array") => {
            let items = schema.get("items").cloned().unwrap_or(json!({}));
            json!([document_for(document, &items, optionals, depth + 1)])
        }
        Some("object") | None => {
            let required: BTreeSet<&str> = schema
                .get("required")
                .and_then(Value::as_array)
                .map(|names| names.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();

            let mut written = serde_json::Map::new();
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                for (name, property) in properties {
                    if !optionals && !required.contains(name.as_str()) {
                        continue;
                    }
                    written.insert(
                        name.clone(),
                        document_for(document, property, optionals, depth + 1),
                    );
                }
            }
            if let Some(values) = schema.get("additionalProperties")
                && values.is_object()
            {
                written.insert(
                    "a key".to_owned(),
                    document_for(document, values, optionals, depth + 1),
                );
            }
            Value::Object(written)
        }
        Some("string") => match schema.get("format").and_then(Value::as_str) {
            Some("uuid") => json!("3f2504e0-4f89-41d3-9a0c-0305e82c3301"),
            Some("date-time") => json!("2026-01-02T03:04:05Z"),
            Some("date") => json!("2026-01-02"),
            Some("url") => json!("https://example.invalid/where-a-webhook-lands"),
            _ => json!("a value the API answered"),
        },
        Some("integer") => json!(12),
        Some("number") => json!(1.5),
        Some("boolean") => json!(true),
        Some(other) => panic!("the document declares a `{other}` nothing here knows how to build"),
    }
}

/// A `reqwest` transport reaching whatever it is pointed at, which is what an application writes.
#[derive(Clone)]
struct Reqwest {
    base: Arc<Mutex<Url>>,
    client: reqwest::Client,
}

impl Transport for Reqwest {
    type Error = reqwest::Error;

    fn request(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, String)],
        body: Option<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<(u16, Vec<u8>), Self::Error>> + Send {
        let base = self
            .base
            .lock()
            .map(|held| held.clone())
            .unwrap_or_else(|_| Url::parse("http://127.0.0.1:1").expect("a parsable fallback"));
        // The path is already written whole by the generated method, escaping included, so it is
        // carried as it was built rather than parsed apart and put back together.
        let target = format!("{}{path}", base.as_str().trim_end_matches('/'));
        let method = method.to_owned();
        let query: Vec<(String, String)> = query
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        let client = self.client.clone();

        async move {
            let verb =
                reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);
            let mut issued = client
                .request(verb, target)
                .header("Authorization", "Bearer token-xyz")
                .header("Accept", "application/json")
                .query(&query);
            if let Some(body) = body {
                issued = issued.header("Content-Type", "application/json").body(body);
            }

            let answer = issued.send().await?;
            let status = answer.status().as_u16();
            let payload = answer.bytes().await?;
            Ok((status, payload.to_vec()))
        }
    }
}

/// Everything one pass over the generated layer needs: the API, the document, and what the answers
/// are held to.
struct Walk {
    document: Value,
    operations: BTreeMap<String, Declared>,
    state: ApiState,
    base: Arc<Mutex<Url>>,
    listening: Url,
    handle: ServerHandle,
    mode: Mutex<Mode>,
    reached: Mutex<BTreeSet<String>>,
}

impl Walk {
    fn start() -> Self {
        let state = ApiState {
            received: Arc::new(Mutex::new(Vec::new())),
            answer: Arc::new(Mutex::new((200, json!({})))),
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

        let listening =
            Url::parse(&format!("http://{address}")).expect("a loopback address parses as a URL");
        let document = api_document();
        let operations = declared_operations(&document);

        Self {
            document,
            operations,
            state,
            base: Arc::new(Mutex::new(listening.clone())),
            listening,
            handle,
            mode: Mutex::new(Mode::ReadsBack { optionals: true }),
            reached: Mutex::new(BTreeSet::new()),
        }
    }

    fn transport(&self) -> Reqwest {
        Reqwest {
            base: Arc::clone(&self.base),
            client: reqwest::Client::new(),
        }
    }

    fn declared(&self, id: &str) -> &Declared {
        self.operations
            .get(id)
            .unwrap_or_else(|| panic!("the API document declares no `{id}`"))
    }

    /// The body one operation reads, built out of the schema the document declares it reads.
    fn body<T: DeserializeOwned>(&self, id: &str) -> T {
        let optionals = matches!(self.mode(), Mode::ReadsBack { optionals: true });
        let schema = self
            .declared(id)
            .reads
            .clone()
            .unwrap_or_else(|| panic!("`{id}` reads no body the document describes"));
        let written = document_for(&self.document, &schema, optionals, 0);
        serde_json::from_value(written.clone()).unwrap_or_else(|e| {
            panic!("`{id}` cannot read the body its own schema describes: {e} in {written}")
        })
    }

    fn mode(&self) -> Mode {
        *self.mode.lock().expect("the mode is readable")
    }

    /// Points the walk at what the next pass is about, and at whatever is answering it.
    fn under(&self, mode: Mode) {
        *self.mode.lock().expect("the mode is writable") = mode;
        let mut base = self.base.lock().expect("the base URL is writable");
        *base = match mode {
            // A port nothing is bound to, which is a request that gets no answer at all.
            Mode::Unreachable => Url::parse("http://127.0.0.1:1").expect("a parsable dead port"),
            _ => self.listening.clone(),
        };
    }

    /// Queues what the API answers the operation about to be asked.
    fn will_answer(&self, id: &str) {
        let declared = self.declared(id);
        let answered = match self.mode() {
            Mode::ReadsBack { optionals } => match declared.answers.as_ref() {
                Some(schema) => (200, document_for(&self.document, schema, optionals, 0)),
                None => (204, json!({})),
            },
            Mode::Refuses => (
                404,
                json!({
                    "id": "NotFound",
                    "title": "Not found",
                    "detail": "what the case scripted",
                    "status": 404,
                    "type": "https://documentation.hook0.com/problems",
                }),
            ),
            Mode::Unreadable => (200, json!("a gateway wrote this")),
            Mode::Unreachable => (200, json!({})),
        };
        *self.state.answer.lock().expect("the answer is writable") = answered;
    }

    fn received(&self) -> Vec<Received> {
        self.state
            .received
            .lock()
            .expect("the recorded requests are readable")
            .clone()
    }

    /// What one operation answered, held to what the document declares of it.
    fn answered<T: Serialize + std::fmt::Debug>(
        &self,
        id: &str,
        answered: Result<T, RequestError>,
    ) {
        let declared = self.declared(id).clone();
        let mode = self.mode();

        if mode == Mode::Unreachable {
            match answered {
                Err(RequestError::Transport(_)) => {}
                other => panic!("`{id}` answered {other:?} although nothing was listening"),
            }
            self.reached
                .lock()
                .expect("the reached operations are writable")
                .insert(id.to_owned());
            return;
        }

        let received = self.received();
        let request = received
            .last()
            .unwrap_or_else(|| panic!("`{id}` issued no request at all"));

        assert_eq!(
            request.method, declared.method,
            "`{id}` was issued with {}",
            request.method
        );
        assert!(
            declared.matches(&request.path),
            "`{id}` reached `{}`, which is not `{}`",
            request.path,
            declared.template
        );
        assert_eq!(
            request.headers.get("authorization").map(String::as_str),
            Some("Bearer token-xyz"),
            "`{id}` carried no credential"
        );

        // The value lands in the path escaped, so that nothing in it can name a segment the
        // operation never had.
        let escaped: String = A_STRING
            .bytes()
            .map(|byte| {
                if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                    char::from(byte).to_string()
                } else {
                    format!("%{byte:02X}")
                }
            })
            .collect();
        for (wanted, sent) in declared
            .template
            .split('/')
            .zip(request.path.split('/'))
            .filter(|(wanted, _)| wanted.starts_with('{'))
        {
            assert_eq!(sent, escaped, "`{id}` left `{wanted}` unescaped");
        }

        let optionals = matches!(mode, Mode::ReadsBack { optionals: true });
        let carried: BTreeSet<String> = url::form_urlencoded::parse(request.query.as_bytes())
            .map(|(name, value)| {
                assert_eq!(value, A_STRING, "`{id}` carried `{name}` altered");
                name.into_owned()
            })
            .collect();
        assert_eq!(
            carried,
            declared.query(optionals),
            "`{id}` assembled a query the document does not declare"
        );

        if let Some(schema) = declared.reads.as_ref() {
            let sent: Value = serde_json::from_str(&request.body)
                .unwrap_or_else(|e| panic!("`{id}` sent a body that is not JSON: {e}"));
            assert_eq!(
                sent,
                document_for(&self.document, schema, optionals, 0),
                "`{id}` sent a body the API cannot read back"
            );
        } else {
            assert!(
                request.body.is_empty(),
                "`{id}` sent a body the document does not declare: {}",
                request.body
            );
        }

        match mode {
            Mode::ReadsBack { optionals } => {
                let read = answered.unwrap_or_else(|e| {
                    panic!("`{id}` failed on what the document declares: {e:?}")
                });
                let written = serde_json::to_value(&read)
                    .unwrap_or_else(|e| panic!("`{id}` read back what cannot be written: {e}"));
                match declared.answers.as_ref() {
                    Some(schema) => assert_eq!(
                        written,
                        document_for(&self.document, schema, optionals, 0),
                        "`{id}` does not write back the document the API answered"
                    ),
                    None => assert_eq!(
                        written,
                        Value::Null,
                        "`{id}` read a value out of an answer the document says carries none"
                    ),
                }
            }
            Mode::Refuses => match answered {
                Err(RequestError::Api(reported)) => {
                    assert_eq!(
                        reported.status, 404,
                        "`{id}` says the API answered {}",
                        reported.status
                    );
                    assert_eq!(
                        reported.kind,
                        Some(generated::ProblemId::NotFound),
                        "`{id}` answered a problem the API did not name"
                    );
                    assert_eq!(
                        reported
                            .problem
                            .as_ref()
                            .map(|problem| problem.detail.as_str()),
                        Some("what the case scripted"),
                        "`{id}` does not carry the document the API answered"
                    );
                }
                other => panic!("`{id}` answered {other:?} where the API reported a problem"),
            },
            Mode::Unreadable => match (answered, declared.answers.as_ref()) {
                // An operation the document says answers nothing reads nothing, so there is
                // nothing about the body it could refuse.
                (Ok(_), None) => {}
                (Err(RequestError::Unreadable { status, .. }), Some(_)) => {
                    assert_eq!(status, 200, "`{id}` says the API answered {status}");
                }
                (other, _) => panic!("`{id}` answered {other:?} to a body it cannot read"),
            },
            Mode::Unreachable => unreachable!("answered before the request was compared"),
        }

        self.reached
            .lock()
            .expect("the reached operations are writable")
            .insert(id.to_owned());
    }

    async fn stop(self) {
        self.handle.stop(true).await;
    }
}

/// Drives one operation and holds what it did to what the document declares of it.
macro_rules! reach {
    ($walk:expr, $id:literal, $call:expr) => {{
        $walk.will_answer($id);
        let answered = $call.await;
        $walk.answered($id, answered);
    }};
}

#[actix_web::test]
async fn every_operation_the_document_declares_is_reached_the_way_it_declares_it() {
    let walk = Walk::start();
    let transport = walk.transport();

    let application_secrets = generated::ApplicationSecretsApi::new(transport.clone());
    let applications = generated::ApplicationsApi::new(transport.clone());
    let errors = generated::ErrorsApi::new(transport.clone());
    let event_types = generated::EventTypesApi::new(transport.clone());
    let events = generated::EventsApi::new(transport.clone());
    let events_per_day = generated::EventsPerDayApi::new(transport.clone());
    let instance = generated::InstanceApi::new(transport.clone());
    let payload_content_types = generated::PayloadContentTypesApi::new(transport.clone());
    let quotas = generated::QuotasApi::new(transport.clone());
    let request_attempts = generated::RequestAttemptsApi::new(transport.clone());
    let response = generated::ResponseApi::new(transport.clone());
    let service_token = generated::ServiceTokenApi::new(transport.clone());
    let subscriptions = generated::SubscriptionsApi::new(transport);

    for mode in [
        Mode::ReadsBack { optionals: true },
        Mode::ReadsBack { optionals: false },
        Mode::Refuses,
        Mode::Unreadable,
        Mode::Unreachable,
    ] {
        walk.under(mode);

        reach!(
            walk,
            "applicationSecrets.create",
            application_secrets.create(walk.body("applicationSecrets.create"))
        );
        reach!(
            walk,
            "applicationSecrets.delete",
            application_secrets.delete(A_STRING, A_STRING)
        );
        reach!(
            walk,
            "applicationSecrets.list",
            application_secrets.list(A_STRING)
        );
        reach!(
            walk,
            "applicationSecrets.update",
            application_secrets.update(A_STRING, walk.body("applicationSecrets.update"))
        );

        reach!(
            walk,
            "applications.create",
            applications.create(walk.body("applications.create"))
        );
        reach!(walk, "applications.delete", applications.delete(A_STRING));
        reach!(walk, "applications.get", applications.get(A_STRING));
        reach!(walk, "applications.list", applications.list(A_STRING));
        reach!(
            walk,
            "applications.update",
            applications.update(A_STRING, walk.body("applications.update"))
        );

        reach!(walk, "errors.list", errors.list());

        reach!(
            walk,
            "eventTypes.create",
            event_types.create(walk.body("eventTypes.create"))
        );
        reach!(
            walk,
            "eventTypes.delete",
            event_types.delete(A_STRING, A_STRING)
        );
        reach!(walk, "eventTypes.get", event_types.get(A_STRING, A_STRING));
        reach!(walk, "eventTypes.list", event_types.list(A_STRING));

        reach!(walk, "events.get", events.get(A_STRING, A_STRING));
        reach!(
            walk,
            "events.ingest",
            events.ingest(walk.body("events.ingest"))
        );
        reach!(walk, "events.list", events.list(A_STRING));
        reach!(
            walk,
            "events.replay",
            events.replay(A_STRING, walk.body("events.replay"))
        );

        let optionals = matches!(mode, Mode::ReadsBack { optionals: true });
        let named = if optionals { Some(A_STRING) } else { None };

        reach!(
            walk,
            "events_per_day.list_for_application",
            events_per_day.list_for_application(A_STRING, named, named)
        );
        reach!(
            walk,
            "events_per_day.list_for_organization",
            events_per_day.list_for_organization(A_STRING, named, named)
        );

        reach!(walk, "instance.get", instance.get());
        reach!(
            walk,
            "payload_content_types.list",
            payload_content_types.list()
        );
        reach!(walk, "quotas.get", quotas.get());

        reach!(
            walk,
            "requestAttempts.get",
            request_attempts.get(A_STRING, A_STRING)
        );
        reach!(
            walk,
            "requestAttempts.list",
            request_attempts.list(A_STRING, named, named, named, named, named, named)
        );

        reach!(walk, "response.get", response.get(A_STRING, A_STRING));

        reach!(
            walk,
            "serviceToken.create",
            service_token.create(walk.body("serviceToken.create"))
        );
        reach!(
            walk,
            "serviceToken.delete",
            service_token.delete(A_STRING, A_STRING)
        );
        reach!(
            walk,
            "serviceToken.get",
            service_token.get(A_STRING, A_STRING)
        );
        reach!(walk, "serviceToken.list", service_token.list(A_STRING));
        reach!(
            walk,
            "serviceToken.update",
            service_token.update(A_STRING, walk.body("serviceToken.update"))
        );

        reach!(
            walk,
            "subscriptions.create",
            subscriptions.create(walk.body("subscriptions.create"))
        );
        reach!(
            walk,
            "subscriptions.delete",
            subscriptions.delete(A_STRING, A_STRING)
        );
        reach!(walk, "subscriptions.get", subscriptions.get(A_STRING));
        reach!(walk, "subscriptions.list", subscriptions.list(A_STRING));
        reach!(
            walk,
            "subscriptions.update",
            subscriptions.update(A_STRING, walk.body("subscriptions.update"))
        );
    }

    let reached = walk
        .reached
        .lock()
        .expect("the reached operations are readable")
        .clone();
    let declared: BTreeSet<String> = walk.operations.keys().cloned().collect();
    assert_eq!(
        reached, declared,
        "the calls above reach fewer operations than the API document declares"
    );

    walk.stop().await;
}

#[actix_web::test]
async fn every_problem_the_document_names_is_answered_as_its_own_kind() {
    let walk = Walk::start();
    let applications = generated::ApplicationsApi::new(walk.transport());

    let named: Vec<String> = walk
        .document
        .pointer("/components/schemas/Problem/properties/id/enum")
        .and_then(Value::as_array)
        .expect("the document names the problems the API reports")
        .iter()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect();
    assert!(!named.is_empty(), "the document names no problem at all");

    for problem in &named {
        *walk.state.answer.lock().expect("the answer is writable") = (
            400,
            json!({
                "id": problem,
                "status": 400,
                "title": "refused",
                "detail": "what the case scripted",
                "type": format!("https://hook0.com/documentation/errors/{problem}"),
            }),
        );

        let answered = applications.get(A_STRING).await;

        match answered {
            Err(RequestError::Api(reported)) => {
                let kind = reported
                    .kind
                    .unwrap_or_else(|| panic!("`{problem}` was answered carrying no kind"));
                assert_eq!(
                    kind.as_str(),
                    problem.as_str(),
                    "`{problem}` was answered as `{kind}`"
                );
                // What a value prints as is the text it travels as, so a message naming a problem
                // names the one an instance would recognise.
                assert_eq!(kind.to_string(), *problem);
                assert_eq!(reported.status, 400);
                assert!(
                    !reported.to_string().is_empty(),
                    "`{problem}` says nothing about itself"
                );
            }
            other => panic!("`{problem}` was answered as {other:?}"),
        }
    }

    walk.stop().await;
}

#[actix_web::test]
async fn a_problem_this_crate_has_never_heard_of_is_still_answered_as_a_failure() {
    let walk = Walk::start();
    let applications = generated::ApplicationsApi::new(walk.transport());
    *walk.state.answer.lock().expect("the answer is writable") = (
        400,
        json!({ "id": "AProblemThisCrateHasNeverHeardOf", "status": 400 }),
    );

    match applications.get(A_STRING).await {
        Err(RequestError::Api(reported)) => {
            assert_eq!(reported.status, 400);
            assert_eq!(
                reported.kind, None,
                "a problem nobody declared was given a kind"
            );
            assert!(reported.problem.is_none());
        }
        other => panic!("a problem nobody declared was answered as {other:?}"),
    }

    walk.stop().await;
}

#[actix_web::test]
async fn a_failure_that_is_not_a_problem_document_is_still_answered_as_one() {
    let walk = Walk::start();
    let applications = generated::ApplicationsApi::new(walk.transport());
    *walk.state.answer.lock().expect("the answer is writable") = (
        502,
        json!("a gateway wrote this, and it is not a problem document"),
    );

    match applications.get(A_STRING).await {
        Err(RequestError::Api(reported)) => {
            assert_eq!(reported.status, 502);
            assert_eq!(reported.kind, None);
            assert!(
                reported.detail.contains("502"),
                "the failure says {} about a gateway that answered 502",
                reported.detail
            );
        }
        other => panic!("a gateway failure was answered as {other:?}"),
    }

    walk.stop().await;
}

#[actix_web::test]
async fn a_body_above_what_a_message_quotes_is_cut_rather_than_echoed_whole() {
    let walk = Walk::start();
    let applications = generated::ApplicationsApi::new(walk.transport());
    let long = "x".repeat(4096);
    *walk.state.answer.lock().expect("the answer is writable") = (500, json!(long));

    match applications.get(A_STRING).await {
        Err(RequestError::Api(reported)) => {
            assert!(
                reported.detail.len() < long.len(),
                "a body of {} characters reached a message whole",
                long.len()
            );
            assert!(reported.detail.contains('…'));
        }
        other => panic!("a long refusal was answered as {other:?}"),
    }

    walk.stop().await;
}

/// Every closed list of strings the document declares, by the name the generator writes it under,
/// with the values the document says it carries.
///
/// The generator names a list after the schema it was found in and every member walked through to
/// reach it, so `ApplicationInfo.onboarding_steps.event` becomes
/// `ApplicationInfoOnboardingStepsEvent`. That rule is applied here rather than assumed: it is what
/// makes the set below comparable against the types written out further down.
fn closed_lists_of(document: &Value) -> BTreeMap<String, Vec<String>> {
    let mut found = BTreeMap::new();
    let schemas = document
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .expect("the document declares schemas");
    for (named, schema) in schemas {
        walk_closed_lists(schema, named, 0, &mut found);
    }
    found
}

/// One schema, and everything nested under it, looking for the closed lists it carries.
fn walk_closed_lists(
    node: &Value,
    named: &str,
    depth: usize,
    found: &mut BTreeMap<String, Vec<String>>,
) {
    if depth > MAX_DEPTH {
        return;
    }

    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        let carried: Vec<String> = values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        if !carried.is_empty() {
            found.insert(named.to_owned(), carried);
        }
        return;
    }

    if let Some(properties) = node.get("properties").and_then(Value::as_object) {
        for (member, property) in properties {
            let reached = format!("{named}{}", pascal(member));
            walk_closed_lists(property, &reached, depth + 1, found);
        }
    }
    // A list carried by the values of an array or of an open-keyed object is named after what holds
    // it rather than after a member of its own, since there is no member to name it after.
    if let Some(items) = node.get("items") {
        walk_closed_lists(items, named, depth + 1, found);
    }
    if let Some(values) = node.get("additionalProperties")
        && values.is_object()
    {
        walk_closed_lists(values, named, depth + 1, found);
    }
}

/// A member of the document as the generator spells it in a type name.
fn pascal(member: &str) -> String {
    member
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut letters = part.chars();
            match letters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + letters.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// A closed list of strings the generator wrote.
///
/// `as_str` is an inherent method on each of them rather than a trait method, so there is no way to
/// walk them all without one. This is that trait, and the macro below is the only thing that
/// implements it.
trait ClosedList: DeserializeOwned + Serialize + std::fmt::Display {
    /// The text this value travels as.
    fn text(&self) -> &'static str;
}

macro_rules! closed_lists {
    ($($named:ty),+ $(,)?) => {
        $(
            impl ClosedList for $named {
                fn text(&self) -> &'static str {
                    self.as_str()
                }
            }
        )+

        /// Drives every list named below, and answers the names it drove.
        fn every_closed_list_driven(declared: &BTreeMap<String, Vec<String>>) -> BTreeSet<String> {
            let mut driven = BTreeSet::new();
            $(
                driven.insert(drives_one::<$named>(declared));
            )+
            driven
        }
    };
}

// Rust cannot be asked what a module declares, so the types are written out. What they are held to
// is not: each is looked up under the name `closed_lists_of` builds for it out of the document, by
// the generator's own naming rule, so a type spelled anything else fails to find its values — and
// the case below says every list the document declares was driven. A list the API grows therefore
// fails this suite until it is named here, rather than going unnoticed.
closed_lists!(
    generated::ApplicationInfoOnboardingStepsEvent,
    generated::ApplicationInfoOnboardingStepsEventType,
    generated::ApplicationInfoOnboardingStepsSubscription,
    generated::OrganizationInfoOnboardingStepsApplication,
    generated::OrganizationInfoOnboardingStepsEvent,
    generated::OrganizationInfoOnboardingStepsEventType,
    generated::OrganizationInfoOnboardingStepsSubscription,
    generated::ProblemId,
    generated::RequestAttemptStatusType,
);

/// One closed list, driven against every value the document declares it carries.
fn drives_one<T: ClosedList>(declared: &BTreeMap<String, Vec<String>>) -> String {
    let named = std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .expect("a type is named")
        .to_owned();
    let values = declared.get(&named).unwrap_or_else(|| {
        panic!(
            "`{named}` is driven here, but the document declares no closed list the generator \
             would have written under that name"
        )
    });

    for value in values {
        let read: T = serde_json::from_value(json!(value)).unwrap_or_else(|refused| {
            panic!("`{named}` refuses `{value}`, which the document says it carries: {refused}")
        });
        assert_eq!(
            read.text(),
            value,
            "`{named}` read `{value}` back as a value that says it travels as `{}`",
            read.text()
        );
        assert_eq!(
            read.to_string(),
            read.text(),
            "`{named}` writes `{value}` as `{read}` when displayed and as `{}` when asked for its \
             text, which are the same thing said twice",
            read.text()
        );
        let written = serde_json::to_value(&read).expect("a closed list writes back");
        assert_eq!(
            written,
            json!(value),
            "`{named}` read `{value}` and wrote it back as `{written}`"
        );
    }

    named
}

#[test]
fn every_closed_list_the_document_declares_carries_the_text_it_declares() {
    // A closed list is the one place the generated types spell text out themselves rather than
    // handing a value to serde: `as_str` writes each value, and `Display` is written on top of it.
    // Both are what a caller logging a value gets, so both are held to the document rather than to
    // each other alone — every value read back, the two spellings agreeing, and what was read
    // written back as what it came from.
    let document = api_document();
    let declared = closed_lists_of(&document);
    assert!(
        !declared.is_empty(),
        "the document names no closed list at all"
    );

    // Driving a list the document does not declare fails inside this call, under the name that was
    // driven, so only the other direction is left to say here.
    let driven = every_closed_list_driven(&declared);
    for named in declared.keys() {
        assert!(
            driven.contains(named),
            "`{named}` is a closed list the document declares and nothing here drives"
        );
    }
}

#[actix_web::test]
async fn every_failure_an_operation_answers_says_what_it_was() {
    // A caller that logs a failure gets a line saying which of the four things went wrong. A
    // message that named none of them would be a client nobody can debug from what it printed.
    let walk = Walk::start();
    let applications = generated::ApplicationsApi::new(walk.transport());

    *walk.state.answer.lock().expect("the answer is writable") = (
        404,
        json!({ "id": "NotFound", "status": 404, "detail": "what the case scripted" }),
    );
    let reported = applications
        .get(A_STRING)
        .await
        .expect_err("a problem the API reported");
    assert!(
        reported.to_string().contains("what the case scripted"),
        "a problem reads as {reported}"
    );

    *walk.state.answer.lock().expect("the answer is writable") =
        (200, json!("a gateway wrote this"));
    let unreadable = applications
        .get(A_STRING)
        .await
        .expect_err("a body the document does not describe");
    assert!(
        unreadable.to_string().contains("200"),
        "an unreadable answer reads as {unreadable}"
    );

    walk.under(Mode::Unreachable);
    let unreached = applications
        .get(A_STRING)
        .await
        .expect_err("an API nothing is listening on");
    assert!(
        unreached.to_string().contains("was not reached"),
        "a request that reached nothing reads as {unreached}"
    );

    walk.stop().await;
}
