#![cfg(feature = "producer")]

//! What a Hook0 application observes when a client upserts its event types.
//!
//! The structured form an event type is split into is internal to the crate; what is observable is
//! the request the client sends for a type the application does not have yet, the names it reports
//! as created, and its refusal of a type whose syntax is wrong. Each case below reads one of those
//! from a real HTTP server bound on a loopback port, so the client performs genuine HTTP.

use actix_web::dev::ServerHandle;
use actix_web::{App, HttpResponse, HttpServer, web};
use hook0_client::{Hook0Client, Hook0ClientError};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use url::Url;
use uuid::Uuid;

/// An event type as the Hook0 API lists it.
#[derive(Debug, Serialize)]
struct ListedEventType {
    event_type_name: String,
}

/// An event type as the client asks the Hook0 API to create it.
#[derive(Debug, Clone, Deserialize)]
struct CreatedEventType {
    service: String,
    resource_type: String,
    verb: String,
}

#[derive(Clone)]
struct ApiState {
    /// The event types the application already has.
    existing: Vec<String>,
    /// Every creation the client asked for, in the order it asked.
    created: Arc<Mutex<Vec<CreatedEventType>>>,
    /// How many requests of any kind reached the API.
    requests: Arc<Mutex<usize>>,
}

async fn list_event_types(state: web::Data<ApiState>) -> HttpResponse {
    state.count_request();

    let body = state
        .existing
        .iter()
        .map(|name| ListedEventType {
            event_type_name: name.to_owned(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(body)
}

async fn create_event_type(
    state: web::Data<ApiState>,
    event_type: web::Json<CreatedEventType>,
) -> HttpResponse {
    state.count_request();

    let event_type = event_type.into_inner();
    let name = format!(
        "{}.{}.{}",
        event_type.service, event_type.resource_type, event_type.verb
    );

    match state.created.lock() {
        Ok(mut created) => created.push(event_type),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    HttpResponse::Created().json(ListedEventType {
        event_type_name: name,
    })
}

impl ApiState {
    fn count_request(&self) {
        if let Ok(mut requests) = self.requests.lock() {
            *requests += 1;
        }
    }
}

/// A Hook0 API listening on a loopback port, for the lifetime of one test.
struct TestApi {
    base_url: Url,
    handle: ServerHandle,
    state: ApiState,
}

impl TestApi {
    /// Starts an API whose application already has `existing` event types.
    fn start(existing: &[&str]) -> Self {
        let state = ApiState {
            existing: existing.iter().map(|name| (*name).to_owned()).collect(),
            created: Arc::new(Mutex::new(Vec::new())),
            requests: Arc::new(Mutex::new(0)),
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
                .route("/event_types", web::get().to(list_event_types))
                .route("/event_types", web::post().to(create_event_type))
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

    fn created(&self) -> Vec<CreatedEventType> {
        self.state
            .created
            .lock()
            .expect("the recorded creations are readable")
            .clone()
    }

    fn request_count(&self) -> usize {
        *self
            .state
            .requests
            .lock()
            .expect("the request count is readable")
    }

    async fn stop(self) {
        self.handle.stop(true).await;
    }
}

#[actix_web::test]
async fn an_event_type_the_application_does_not_have_yet_is_created_from_its_three_parts() {
    let api = TestApi::start(&[]);

    let created = api
        .client()
        .upsert_event_types(&["service.resource.verb"])
        .await
        .expect("the API accepts the creation");

    assert_eq!(created, vec!["service.resource.verb".to_owned()]);

    let received = api.created();
    assert_eq!(
        received.len(),
        1,
        "expected exactly one creation, got {received:?}"
    );
    assert_eq!(received[0].service, "service");
    assert_eq!(received[0].resource_type, "resource");
    assert_eq!(received[0].verb, "verb");

    api.stop().await;
}

#[actix_web::test]
async fn an_event_type_the_application_already_has_is_left_alone() {
    // The name the client compares against the ones the API lists is the three parts joined back
    // with dots, so a type listed as `service.resource.verb` is recognised as the one asked for.
    let api = TestApi::start(&["service.resource.verb"]);

    let created = api
        .client()
        .upsert_event_types(&["service.resource.verb"])
        .await
        .expect("the API answers the listing");

    assert!(
        created.is_empty(),
        "expected no event type to be created, got {created:?}"
    );
    assert!(
        api.created().is_empty(),
        "expected no creation to reach the API, got {:?}",
        api.created()
    );

    api.stop().await;
}

#[actix_web::test]
async fn an_event_type_whose_syntax_is_wrong_is_refused_before_anything_is_sent() {
    let api = TestApi::start(&[]);

    let result = api.client().upsert_event_types(&["test.test"]).await;

    assert!(
        matches!(&result, Err(Hook0ClientError::InvalidEventType(name)) if name == "test.test"),
        "expected an InvalidEventType error naming test.test, got {result:?}"
    );
    assert_eq!(
        api.request_count(),
        0,
        "an event type that cannot be parsed must not reach the API"
    );

    api.stop().await;
}
