//! The Rust client against a Hook0 that is really running.
//!
//! Two things happen here, and the second is the reason the first is worth having.
//!
//! The control: whether an application secret the API minted is accepted, whether a second send
//! under an identifier already ingested comes back as the conflict it is, and whether a signature
//! the *output worker* computed verifies. Those are the three questions no loopback suite can ask
//! itself, because a suite that signs and verifies with the same module only proves the module
//! agrees with itself.
//!
//! The surface: every operation the API document declares, driven through the generated layer
//! against the same instance, one line of report each. `clients/rust/tests/generated.rs` already
//! drives all of them — against an API the suite itself writes, out of the same document the
//! client was generated from. That proves the client matches the document. It cannot prove the
//! document matches Hook0, and a field the API really answers under another name passes there and
//! fails on a consumer's first call.
//!
//! Each operation is reported, and so is each generated model type this client decodes out of a
//! real answer. Operations alone would not be enough: every one of them could come back refused,
//! and a client that can decode nothing at all would satisfy that bijection while never having
//! parsed a model out of what Hook0 sent.
//!
//! The calls below are written out because Rust cannot be asked what a module declares. The sets
//! they are held to are not: the harness reads the API document through the generator and refuses
//! this smoke unless the operations reported and the operations declared are the same set, and
//! likewise for the models. An operation or a type the API grows fails here until somebody drives
//! it.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use hook0_client::generated::{
    ApplicationPost, ApplicationSecretPost, ApplicationSecretsApi, ApplicationsApi, ErrorsApi,
    EventPost, EventTypePost, EventTypesApi, EventsApi, EventsPerDayApi, InstanceApi,
    PayloadContentTypesApi, ProblemError, QuotasApi, ReplayEvent, RequestAttemptsApi, RequestError,
    ResponseApi, ServiceTokenApi, ServiceTokenPost, SubscriptionPost, SubscriptionPostTarget,
    SubscriptionsApi, Transport,
};
use hook0_client::{Event, Hook0Client, verify_webhook_signature};
use url::Url;
use uuid::Uuid;

/// The conflict the API answers a duplicated ingestion with.
const ALREADY_INGESTED: &str = "EventAlreadyIngested";

/// What this smoke labels everything it creates with, so that the subscription it makes and the
/// event it sends find each other.
const LANGUAGE: &str = "rust";

/// Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
/// delivery proves is proved once, by the webhook the harness catches and every language verifies,
/// and a subscription pointing at something real would make this flow wait for a worker.
const NOWHERE: &str = "http://127.0.0.1:1/";

/// How long one request to the instance is given.
const REQUEST_WITHIN: Duration = Duration::from_secs(30);

/// The most times one request is sent again after the instance answered that it is arriving too
/// fast. Past this, what the instance says is what the caller is told.
const PACED_AGAIN: u32 = 8;

/// The shortest and the longest this waits before sending a paced request again.
const SHORTEST_PAUSE: Duration = Duration::from_millis(200);
const LONGEST_PAUSE: Duration = Duration::from_secs(10);

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let settings = Settings::read()?;

    control(&settings).await?;
    surface(&settings).await?;

    // Last, and on purpose: it needs no instance at all, so it still answers after the flow above
    // has deleted the application it was run against.
    verify(&settings.delivery)?;
    println!("the signature the instance produced verifies");
    Ok(())
}

/// Everything the harness hands this smoke.
struct Settings {
    /// Where the API answers, path included, which is what [`Hook0Client`] is built with.
    api_url: Url,
    /// The same instance without that path, which is what the generated layer is issued against:
    /// every path it builds already carries the one the document's own server URL leaves out.
    origin: String,
    application_id: Uuid,
    organization_id: Uuid,
    /// An application secret, which is what an SDK authenticates with.
    token: String,
    /// An organization-scoped credential, for what an application secret is not allowed to reach.
    service_token: String,
    /// An event type the application already declares.
    event_type: String,
    /// An application of the same organization that the instance has already delivered a webhook
    /// for, and refreshed its counts of. Reading it needs the organization credential.
    seeded_application_id: Uuid,
    /// A delivery attempt of that application, and the response the worker wrote for it. Neither
    /// exists until the worker has run, which is why the harness waits for them once rather than
    /// every language waiting for its own.
    request_attempt_id: Uuid,
    response_id: Uuid,
    /// A directory holding the webhook the instance really delivered.
    delivery: PathBuf,
}

impl Settings {
    fn read() -> Result<Self, String> {
        let api_url = url(&setting("HOOK0_API_URL")?)?;

        // The instance without the path the hand-written half is built with. The generated half
        // composes paths that already carry `/api/v1`, since the API document's own server URL is
        // the bare origin. Every SDK's own transport resolves a path against its base and would
        // reach the same request either way; `Reaching` below is this smoke's own and joins the two
        // by concatenating, so for it the origin is the only form that works.
        let mut origin = api_url.clone();
        origin.set_path("");
        origin.set_query(None);

        Ok(Self {
            origin: origin.as_str().trim_end_matches('/').to_owned(),
            api_url,
            application_id: identifier(&setting("HOOK0_APPLICATION_ID")?)?,
            organization_id: identifier(&setting("HOOK0_ORGANIZATION_ID")?)?,
            token: setting("HOOK0_TOKEN")?,
            service_token: setting("HOOK0_SERVICE_TOKEN")?,
            event_type: setting("HOOK0_EVENT_TYPE")?,
            seeded_application_id: identifier(&setting("HOOK0_SEEDED_APPLICATION_ID")?)?,
            request_attempt_id: identifier(&setting("HOOK0_REQUEST_ATTEMPT_ID")?)?,
            response_id: identifier(&setting("HOOK0_RESPONSE_ID")?)?,
            delivery: PathBuf::from(setting("HOOK0_DELIVERY")?),
        })
    }
}

/// The three questions the loopback suite cannot ask, against the instance that can answer them.
async fn control(settings: &Settings) -> Result<(), String> {
    let client = Hook0Client::new(
        settings.api_url.clone(),
        settings.application_id,
        &settings.token,
    )
    .map_err(|cause| format!("building the client: {cause}"))?;

    let sent = client
        .send_event(&event(&settings.event_type, None))
        .await
        .map_err(|cause| format!("the instance refused the first send: {cause}"))?;
    println!("ingested {sent}");

    let refused = client
        .send_event(&event(&settings.event_type, Some(&sent)))
        .await
        .err()
        .ok_or("sending the same event twice was accepted twice")?;
    let said = format!("{refused}");
    if !said.contains(ALREADY_INGESTED) {
        return Err(format!(
            "the second send failed without naming {ALREADY_INGESTED}: {said}"
        ));
    }
    println!("the second send reported {ALREADY_INGESTED}");

    Ok(())
}

/// Every operation the API document declares, driven against the instance in the order a consumer
/// would: what it needs is created, read and listed, updated, and destroyed last.
///
/// Two credentials, because the API takes two and one of them cannot do everything. An application
/// secret is scoped to the application it belongs to; what belongs to the *organization* — listing
/// its applications, everything about service tokens, its per-day counts — needs the
/// organization-scoped token beside it. A flow holding only the first could report those operations
/// but would only ever be refused by them, and the types they answer would go undecoded.
async fn surface(settings: &Settings) -> Result<(), String> {
    let held = Reaching::new(&settings.origin, &settings.token)?;
    let organization_wide = Reaching::new(&settings.origin, &settings.service_token)?;

    let applications = ApplicationsApi::new(held.clone());
    let secrets = ApplicationSecretsApi::new(held.clone());
    let event_types = EventTypesApi::new(held.clone());
    let subscriptions = SubscriptionsApi::new(held.clone());
    let events = EventsApi::new(held.clone());
    let instance = InstanceApi::new(held.clone());
    let quotas = QuotasApi::new(held.clone());
    let payload_content_types = PayloadContentTypesApi::new(held.clone());
    let errors = ErrorsApi::new(held.clone());
    let events_per_day = EventsPerDayApi::new(held);

    let organization_applications = ApplicationsApi::new(organization_wide.clone());
    let organization_events_per_day = EventsPerDayApi::new(organization_wide.clone());
    let request_attempts = RequestAttemptsApi::new(organization_wide.clone());
    let responses = ResponseApi::new(organization_wide.clone());
    let service_tokens = ServiceTokenApi::new(organization_wide);

    let application = settings.application_id.to_string();
    let organization = settings.organization_id.to_string();
    let seeded = settings.seeded_application_id.to_string();

    // What the instance says about itself, which is what an application asks before it has
    // anything of its own: how it is configured, what it will let this account do, what a payload
    // may be, and every problem it can report.
    let configured = read("instance.get", instance.get().await)?;
    decoded("InstanceConfig", &configured);

    let allowed = read("quotas.get", quotas.get().await)?;
    decoded("QuotasResponseLimits", &allowed.limits);
    decoded("QuotasResponse", &allowed);

    exercised(
        "payload_content_types.list",
        payload_content_types.list().await,
    )?;

    let catalogue = read("errors.list", errors.list().await)?;
    let problem = catalogue
        .first()
        .ok_or("the instance published an empty catalogue of the problems it can report")?;
    decoded("ProblemId", &problem.id);
    decoded("Problem", problem);

    // The application this smoke owns. One per language, so that the three deletions at the end of
    // this flow are real deletions rather than something eleven other smokes have to live with.
    let info = read("applications.get", applications.get(&application).await)?;
    decoded("ApplicationInfoConsumption", &info.consumption);
    decoded("ApplicationInfoQuotas", &info.quotas);
    decoded(
        "ApplicationInfoOnboardingStepsEvent",
        &info.onboarding_steps.event,
    );
    decoded(
        "ApplicationInfoOnboardingStepsEventType",
        &info.onboarding_steps.event_type,
    );
    decoded(
        "ApplicationInfoOnboardingStepsSubscription",
        &info.onboarding_steps.subscription,
    );
    decoded("ApplicationInfoOnboardingSteps", &info.onboarding_steps);
    decoded("ApplicationInfo", &info);

    let renamed = read(
        "applications.update",
        applications
            .update(
                &application,
                ApplicationPost {
                    name: "the application the rust smoke drives".to_owned(),
                    organization_id: settings.organization_id,
                },
            )
            .await,
    )?;
    decoded("Application", &renamed);

    // The organization's, so the organization credential. Listing what an account has is the first
    // thing a console does.
    exercised(
        "applications.list",
        organization_applications.list(&organization).await,
    )?;

    // This one is driven with the *application* secret on purpose, and it is the flow's one
    // refusal. Creating an application is the organization's business and an application secret is
    // not the organization's, so the instance answers a problem document and this client reads it
    // — which is the half of the client that nothing else here would exercise. A flow whose every
    // answer is a success never finds out whether it can read a failure.
    exercised(
        "applications.create",
        applications
            .create(ApplicationPost {
                name: "an application the rust smoke's application secret may not create"
                    .to_owned(),
                organization_id: settings.organization_id,
            })
            .await,
    )?;

    // A second secret, so that the one this smoke is authenticating with is never the one it
    // revokes. Deleting that one succeeds and then locks the flow out of everything below.
    let minted = read(
        "applicationSecrets.create",
        secrets
            .create(ApplicationSecretPost {
                application_id: settings.application_id,
                name: Some("a secret the rust smoke minted".to_owned()),
            })
            .await,
    )?;
    decoded("ApplicationSecret", &minted);
    let minted_token = minted.token.to_string();

    exercised("applicationSecrets.read", secrets.read(&application).await)?;
    exercised(
        "applicationSecrets.update",
        secrets
            .update(
                &minted_token,
                ApplicationSecretPost {
                    application_id: settings.application_id,
                    name: Some("a secret the rust smoke renamed".to_owned()),
                },
            )
            .await,
    )?;
    exercised(
        "applicationSecrets.delete",
        secrets.delete(&minted_token, &application).await,
    )?;

    // An event type of this smoke's own, rather than the one the harness declared: what is created
    // here is what is subscribed to, sent, replayed and deleted below.
    let declared = read(
        "eventTypes.create",
        event_types
            .create(EventTypePost {
                application_id: settings.application_id,
                service: LANGUAGE.to_owned(),
                resource_type: "smoke".to_owned(),
                verb: "ran".to_owned(),
            })
            .await,
    )?;
    decoded("EventType", &declared);

    exercised(
        "eventTypes.get",
        event_types
            .get(&declared.event_type_name, &application)
            .await,
    )?;
    exercised("eventTypes.list", event_types.list(&application).await)?;

    let labels = HashMap::from([("language".to_owned(), LANGUAGE.to_owned())]);
    let subscription = read(
        "subscriptions.create",
        subscriptions
            .create(SubscriptionPost {
                application_id: settings.application_id,
                dedicated_workers: None,
                description: Some(
                    "what the rust smoke subscribes to its own events with".to_owned(),
                ),
                event_types: vec![declared.event_type_name.clone()],
                is_enabled: true,
                label_key: None,
                label_value: None,
                labels: Some(labels.clone()),
                metadata: None,
                target: SubscriptionPostTarget {
                    headers: serde_json::json!({}),
                    method: "POST".to_owned(),
                    type_: "http".to_owned(),
                    url: url(NOWHERE)?,
                },
            })
            .await,
    )?;
    decoded("SubscriptionTarget", &subscription.target);
    decoded("Subscription", &subscription);
    let subscribed = subscription.subscription_id.to_string();

    exercised("subscriptions.get", subscriptions.get(&subscribed).await)?;
    exercised("subscriptions.list", subscriptions.list(&application).await)?;
    exercised(
        "subscriptions.update",
        subscriptions
            .update(
                &subscribed,
                SubscriptionPost {
                    application_id: settings.application_id,
                    dedicated_workers: None,
                    description: Some("what the rust smoke renamed it to".to_owned()),
                    event_types: vec![declared.event_type_name.clone()],
                    is_enabled: true,
                    label_key: None,
                    label_value: None,
                    labels: Some(labels.clone()),
                    metadata: None,
                    target: SubscriptionPostTarget {
                        headers: serde_json::json!({}),
                        method: "POST".to_owned(),
                        type_: "http".to_owned(),
                        url: url(NOWHERE)?,
                    },
                },
            )
            .await,
    )?;

    // The event the subscription above selects, sent through the generated layer rather than
    // through `send_event`: the hand-written half has its own three questions above, and this is
    // the operation the document declares.
    let ingested = read(
        "events.ingest",
        events
            .ingest(EventPost {
                application_id: settings.application_id,
                event_id: Some(Uuid::now_v7()),
                event_type: declared.event_type_name.clone(),
                labels,
                metadata: None,
                occurred_at: Utc::now(),
                payload: r#"{"from":"the rust smoke"}"#.to_owned(),
                payload_content_type: "application/json".to_owned(),
            })
            .await,
    )?;
    decoded("IngestedEvent", &ingested);
    let sent = ingested.event_id.to_string();

    let whole = read("events.get", events.get(&sent, &application).await)?;
    decoded("EventWithPayload", &whole);

    let listed = read("events.list", events.list(&application).await)?;
    let event = listed
        .first()
        .ok_or("the instance ingested an event and then listed none")?;
    decoded("Event", event);

    exercised(
        "events.replay",
        events
            .replay(
                &sent,
                ReplayEvent {
                    application_id: settings.application_id,
                },
            )
            .await,
    )?;

    // This application was created a moment ago and the counts come out of a view the instance
    // refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
    // answer, and one a client has to be able to read.
    exercised(
        "events_per_day.list_for_application",
        events_per_day
            .list_for_application(&application, None, None)
            .await,
    )?;

    // The organization's counts do have something in them: the harness waited for the instance to
    // refresh them before running any of this, precisely so that the type they are answered with is
    // one a client decodes rather than one nothing ever produces.
    let per_day = read(
        "events_per_day.list_for_organization",
        organization_events_per_day
            .list_for_organization(&organization, None, None)
            .await,
    )?;
    let day = per_day
        .first()
        .ok_or("the organization has ingested events and its per-day counts are empty")?;
    decoded("EventsPerDayEntry", day);

    // An attempt and a response exist only once the output worker has finished a delivery. The
    // harness waited for one, in the application it caught the shared delivery from, and handed the
    // ids on — so this reads them back with the organization credential rather than waiting again.
    exercised(
        "requestAttempts.read",
        request_attempts
            .read(&seeded, None, None, None, None, None, None)
            .await,
    )?;
    let attempted = read(
        "requestAttempts.get",
        request_attempts
            .get(&settings.request_attempt_id.to_string(), &seeded)
            .await,
    )?;
    decoded("RequestAttemptEvent", &attempted.event);
    decoded("RequestAttemptSubscription", &attempted.subscription);
    decoded("RequestAttemptStatusType", &attempted.status.type_);
    decoded("RequestAttemptStatus", &attempted.status);
    decoded("RequestAttempt", &attempted);

    let answered = read(
        "response.get",
        responses
            .get(&settings.response_id.to_string(), &seeded)
            .await,
    )?;
    decoded("Response", &answered);

    // Service tokens belong to the organization, so they are minted, read and revoked with the
    // organization credential. The one revoked below is the one minted here — never the one this
    // half of the flow is authenticating with.
    let token = read(
        "serviceToken.create",
        service_tokens
            .create(ServiceTokenPost {
                name: "a token the rust smoke minted".to_owned(),
                organization_id: settings.organization_id,
            })
            .await,
    )?;
    decoded("ServiceToken", &token);
    let minted_id = token.token_id.to_string();

    exercised(
        "serviceToken.list",
        service_tokens.list(&organization).await,
    )?;
    exercised(
        "serviceToken.get",
        service_tokens.get(&minted_id, &organization).await,
    )?;
    exercised(
        "serviceToken.edit",
        service_tokens
            .edit(
                &minted_id,
                ServiceTokenPost {
                    name: "a token the rust smoke renamed".to_owned(),
                    organization_id: settings.organization_id,
                },
            )
            .await,
    )?;
    exercised(
        "serviceToken.delete",
        service_tokens.delete(&minted_id, &organization).await,
    )?;

    // Destroyed in the order the instance can accept: the subscription that references the event
    // type, then the event type, then the application — which is last because the secret this
    // whole flow authenticates with stops authenticating the moment its application is gone.
    exercised(
        "subscriptions.delete",
        subscriptions.delete(&subscribed, &application).await,
    )?;
    exercised(
        "eventTypes.delete",
        event_types
            .delete(&declared.event_type_name, &application)
            .await,
    )?;
    exercised(
        "applications.delete",
        applications.delete(&application).await,
    )?;

    Ok(())
}

/// Reports one generated model type as decoded out of a real answer.
///
/// The value is taken rather than only named, so the line cannot outlive what it is about: a type
/// that stops being part of an answer stops compiling here rather than going on being reported.
fn decoded<T>(model: &str, _value: &T) {
    println!("decoded {model}");
}

/// One operation the flow goes on to use the answer of, which therefore has to be a success.
fn read<T>(operation: &str, answered: Result<T, RequestError>) -> Result<T, String> {
    match answered {
        Ok(value) => {
            println!("exercised {operation} accepted");
            Ok(value)
        }
        Err(refused) => Err(format!(
            "{operation}: the flow needs what it answers, and it answered {refused}"
        )),
    }
}

/// One operation driven for its own sake, reported whichever way the instance answered it.
///
/// A success and a problem are both complete round trips through the generated layer: the request
/// was composed, the instance answered, and this client read the answer. What is neither — the API
/// not reached, a body this client cannot read, a problem it does not know — stops the smoke,
/// because none of those say the client and the instance agree on anything.
fn exercised<T>(operation: &str, answered: Result<T, RequestError>) -> Result<(), String> {
    match answered {
        Ok(_) => {
            println!("exercised {operation} accepted");
            Ok(())
        }
        Err(RequestError::Api(problem)) => {
            println!(
                "exercised {operation} refused:{}",
                named(operation, &problem)?
            );
            Ok(())
        }
        Err(refused) => Err(format!("{operation}: {refused}")),
    }
}

/// The problem the instance named, or a refusal saying it named none this client knows.
fn named(operation: &str, problem: &ProblemError) -> Result<String, String> {
    match problem.kind {
        Some(kind) => Ok(kind.to_string()),
        None => Err(format!(
            "{operation}: the instance answered {} and this client read no problem it knows out of \
             it: {}",
            problem.status, problem.detail
        )),
    }
}

/// What every generated method is issued through: one HTTP client, pointed at the instance,
/// carrying the application secret.
///
/// The generated half is handed a transport and nothing else — reaching the network is the
/// application's business — so this is the piece a consumer writes, and writing it here is what
/// makes this smoke the same shape as their code.
#[derive(Clone)]
struct Reaching {
    origin: String,
    token: String,
    client: reqwest::Client,
}

impl Reaching {
    /// One transport, pointed at an instance and carrying one credential.
    ///
    /// Per credential rather than per client: which of the two a request needs is the operation's
    /// business, and a transport that switched tokens depending on the path would be this smoke
    /// deciding what the API's authorization rules are.
    fn new(origin: &str, token: &str) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_WITHIN)
            .build()
            .map_err(|cause| format!("building the transport: {cause}"))?;

        Ok(Self {
            origin: origin.to_owned(),
            token: token.to_owned(),
            client,
        })
    }
}

impl Transport for Reaching {
    type Error = Unreached;

    fn request(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, String)],
        body: Option<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<(u16, Vec<u8>), Self::Error>> + Send {
        // The path is written whole by the generated method, escaping included, so it is carried
        // as it was built rather than parsed apart and put back together.
        let target = format!("{}{path}", self.origin);
        let method = method.to_owned();
        let token = self.token.clone();
        let query: Vec<(String, String)> = query
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        let client = self.client.clone();

        async move {
            let verb = reqwest::Method::from_bytes(method.as_bytes())
                .map_err(|_| Unreached::Method(method))?;

            let mut sent = 0;
            loop {
                let mut issued = client
                    .request(verb.clone(), target.clone())
                    .header("Authorization", format!("Bearer {token}"))
                    .header("Accept", "application/json")
                    .query(&query);
                if let Some(body) = body.clone() {
                    issued = issued.header("Content-Type", "application/json").body(body);
                }

                let answer = issued.send().await.map_err(Unreached::Sending)?;
                sent += 1;

                // Hook0 paces callers per credential, and a flow driving three dozen operations
                // one after another is exactly what that is for. The answer says the request was
                // not processed and is safe to send again after the delay it names, so this waits
                // and sends it again rather than handing the caller a problem that says nothing
                // about the operation it was asking about. Bounded both ways: an instance that is
                // still refusing after this many tries is one whose answer the caller should see.
                if answer.status() == reqwest::StatusCode::TOO_MANY_REQUESTS && sent <= PACED_AGAIN
                {
                    tokio::time::sleep(after(&answer)).await;
                    continue;
                }

                let status = answer.status().as_u16();
                let payload = answer.bytes().await.map_err(Unreached::Reading)?;
                return Ok((status, payload.to_vec()));
            }
        }
    }
}

/// How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
///
/// The floor is there because the header counts in whole seconds and the delay being waited out is
/// a fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
/// again immediately, forever. The ceiling is there because a header is written by a server this
/// smoke does not control.
fn after(answer: &reqwest::Response) -> Duration {
    answer
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|seconds| seconds.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(SHORTEST_PAUSE)
        .clamp(SHORTEST_PAUSE, LONGEST_PAUSE)
}

/// What this transport reports when the instance was not reached, or when it was asked for
/// something it could not ask for.
#[derive(Debug)]
enum Unreached {
    /// A request line the generated layer named and this transport cannot write.
    Method(String),
    /// The request never left.
    Sending(reqwest::Error),
    /// It left and the answer never came back whole.
    Reading(reqwest::Error),
}

impl fmt::Display for Unreached {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Method(named) => write!(formatter, "`{named}` is not a request method"),
            Self::Sending(cause) => write!(formatter, "the request was not sent: {cause}"),
            Self::Reading(cause) => write!(formatter, "the answer was not read: {cause}"),
        }
    }
}

impl std::error::Error for Unreached {}

/// The event both control sends carry, under the identifier the caller names.
fn event<'a>(event_type: &'a str, event_id: Option<&'a Uuid>) -> Event<'a> {
    Event {
        event_id,
        event_type,
        payload: r#"{"from":"the rust smoke"}"#.into(),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![("language".to_owned(), LANGUAGE.to_owned())],
    }
}

/// Verifies what the output worker really delivered, with this client's own verification.
fn verify(delivery: &Path) -> Result<(), String> {
    let signature = part(delivery, "signature")?;
    let secret = part(delivery, "secret")?;
    let tolerance: u64 = part(delivery, "tolerance")?
        .trim()
        .parse()
        .map_err(|cause| format!("the tolerance is not a number of seconds: {cause}"))?;
    let body = fs::read(delivery.join("body"))
        .map_err(|cause| format!("reading the delivered body: {cause}"))?;

    let delivered = part(delivery, "headers")?;
    let headers: Vec<(&str, &str)> = delivered
        .lines()
        .filter_map(|line| line.split_once(": "))
        .collect();

    verify_webhook_signature(
        signature.trim(),
        &body,
        &headers,
        secret.trim(),
        Duration::from_secs(tolerance),
    )
    .map_err(|refused| format!("the signature the instance produced was refused: {refused}"))
}

/// One part of the delivery, as the harness wrote it down.
fn part(delivery: &Path, name: &str) -> Result<String, String> {
    fs::read_to_string(delivery.join(name))
        .map_err(|cause| format!("reading the delivered {name}: {cause}"))
}

/// A setting the harness passes, or a refusal naming it: a smoke that ran without one would report
/// a failure of the client for something the harness never handed it.
fn setting(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{name} is not set"))
}

fn url(written: &str) -> Result<Url, String> {
    written
        .parse()
        .map_err(|cause| format!("{written} is not a URL: {cause}"))
}

fn identifier(written: &str) -> Result<Uuid, String> {
    written
        .parse()
        .map_err(|cause| format!("{written} is not an identifier: {cause}"))
}
