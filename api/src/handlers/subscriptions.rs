use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use paperclip::v2::models::{DataType, DataTypeFormat, DefaultSchemaRaw};
use paperclip::v2::schema::Apiv2Schema;
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{query, query_as, query_scalar};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use tracing::error;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::hook0_client::{
    EventSubscriptionCreated, EventSubscriptionRemoved, EventSubscriptionUpdated, Hook0ClientEvent,
};
use crate::iam::{Action, authorize_for_application, get_owner_organization};
use crate::openapi::OaBiscuit;
use crate::opentelemetry::report_cancelled_request_attempts;
use crate::problems::Hook0Problem;
use crate::validators::{
    subscription_target_http_method, subscription_target_http_method_headers,
    subscription_target_http_url,
};

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Subscription {
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub is_enabled: bool,
    pub event_types: Vec<String>,
    pub description: Option<String>,
    /// HMAC signing secret shared with the receiver. Present only for
    /// configuration-capable callers (Editor, service or master token); omitted
    /// entirely for a read-only Viewer. Absent field, never `null`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<Uuid>,
    pub metadata: HashMap<String, String>,
    /// _Kept for backward compatibility, you should use `labels`_
    pub label_key: String,
    /// _Kept for backward compatibility, you should use `labels`_
    pub label_value: String,
    pub labels: HashMap<String, String>,
    pub target: Target,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dedicated_workers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Target {
    Http {
        method: String,
        #[serde(deserialize_with = "deserialize_http_url")]
        url: HttpUrl,
        headers: HeaderMap,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, derive_more::Into)]
#[into(owned, ref)]
pub struct HeaderMap(#[serde(with = "http_serde::header_map")] pub reqwest::header::HeaderMap);

// This implementation is manual because paperclip could not handle the Target enum automatically and exposed it as a string in the generated OpenAPI document
impl Apiv2Schema for Target {
    fn raw_schema() -> DefaultSchemaRaw {
        let type_ = DefaultSchemaRaw {
            data_type: Some(DataType::String),
            example: Some(Value::String("http".to_owned())),
            ..Default::default()
        };
        let method = DefaultSchemaRaw {
            data_type: Some(DataType::String),
            ..Default::default()
        };
        let url = DefaultSchemaRaw {
            data_type: Some(DataType::String),
            format: Some(DataTypeFormat::Url),
            ..Default::default()
        };
        let headers = DefaultSchemaRaw {
            data_type: Some(DataType::Object),
            ..Default::default()
        };

        DefaultSchemaRaw {
            data_type: Some(DataType::Object),
            properties: BTreeMap::from_iter([
                ("type".to_owned(), Box::new(type_)),
                ("method".to_owned(), Box::new(method)),
                ("url".to_owned(), Box::new(url)),
                ("headers".to_owned(), Box::new(headers)),
            ]),
            required: BTreeSet::from_iter([
                "type".to_owned(),
                "method".to_owned(),
                "url".to_owned(),
                "headers".to_owned(),
            ]),
            ..Default::default()
        }
    }
}

impl Validate for Target {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Target::Http {
                method,
                url,
                headers,
            } => {
                let mut errors = ValidationErrors::new();

                let method_validation = subscription_target_http_method(method);
                if let Err(e) = method_validation {
                    errors.add("method", e)
                }

                let url_validation = subscription_target_http_url(url.as_str());
                if let Err(e) = url_validation {
                    errors.add("url", e)
                }

                let headers_validation = subscription_target_http_method_headers(headers.into());
                if let Err(e) = headers_validation {
                    errors.add("headers", e)
                }

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpUrl(Url);

impl Deref for HttpUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Url> for HttpUrl {
    fn as_ref(&self) -> &Url {
        &self.0
    }
}

fn deserialize_http_url<'de, D>(deserializer: D) -> Result<HttpUrl, D::Error>
where
    D: Deserializer<'de>,
{
    const ALLOWED_SCHEMES: &[&str] = &["http", "https"];
    let url = Url::deserialize(deserializer)?;

    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        Err(serde::de::Error::custom(format!(
            "'{}' URLs are not allowed; use one of the following schemes: {}",
            url.scheme(),
            ALLOWED_SCHEMES.join(", ")
        )))
    } else if !url.has_host() {
        Err(serde::de::Error::custom(
            "URL must contain a host (domain or IP address)",
        ))
    } else {
        Ok(HttpUrl(url))
    }
}

#[derive(Debug, Deserialize, Serialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
}

#[api_v2_operation(
    summary = "List subscriptions",
    description = "Retrieves all active webhook subscriptions for an application. Each subscription defines which event types to listen for and where to deliver them (HTTP endpoint). Use application_id query parameter to filter by application.",
    operation_id = "subscriptions.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp", "sdk")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Subscription>>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionList {
            application_id: &qs.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    // Least-privilege gate on the HMAC signing secret: the read above already
    // passed (Viewer included), so this only decides whether the secret is
    // exposed. We reuse the *existing* write action `subscription:create` as the
    // probe rather than a dedicated action — the secret is a configuration
    // credential, so "may configure subscriptions here" is exactly who should
    // see it (Editor, plus service/master which pass any check). Probing a real
    // write action also honours biscuit attenuation: a token narrowed to
    // read-only fails it and gets the secret omitted, as intended.
    // A denial means "omit the secret", NOT "deny the request" — hence
    // `.is_ok()`, never `?`: propagating it would wrongly turn a Viewer's read
    // into a 403.
    let reveal_secret = authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionCreate {
            application_id: &qs.application_id,
            labels: &HashMap::new(),
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await
    .is_ok();

    #[allow(non_snake_case)]
    struct RawSubscription {
        subscription__id: Uuid,
        is_enabled: bool,
        event_types: Option<Vec<String>>,
        description: Option<String>,
        secret: Uuid,
        metadata: Value,
        labels: Value,
        target_json: Option<Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        dedicated_workers: Option<Vec<String>>,
    }

    let raw_subscriptions = query_as!(
        RawSubscription,
        r#"
            WITH subs AS (
                SELECT
                    s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.labels, s.target__id, s.created_at, s.updated_at,
                    CASE WHEN length((array_agg(set.event_type__name))[1]) > 0
                        THEN array_agg(set.event_type__name)
                        ELSE ARRAY[]::text[] END AS event_types,
                    CASE WHEN length((array_agg(w.name))[1]) > 0
                        THEN array_agg(w.name)
                        ELSE ARRAY[]::text[] END AS dedicated_workers
                FROM webhook.subscription AS s
                LEFT JOIN webhook.subscription__event_type AS set ON set.subscription__id = s.subscription__id
                LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                LEFT JOIN infrastructure.worker AS w ON w.worker__id = sw.worker__id
                WHERE s.application__id = $1 AND deleted_at IS NULL
                GROUP BY s.subscription__id
                ORDER BY s.created_at ASC
            ), targets AS (
                SELECT target__id, jsonb_build_object(
                    'type', replace(tableoid::regclass::text, 'webhook.target_', ''),
                    'method', method,
                    'url', url,
                    'headers', headers
                ) AS target_json FROM webhook.target_http
                WHERE target__id IN (SELECT target__id FROM subs)
            )
            SELECT subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.labels AS "labels!", subs.created_at AS "created_at!", subs.updated_at AS "updated_at!", subs.event_types, targets.target_json, subs.dedicated_workers
            FROM subs
            INNER JOIN targets ON subs.target__id = targets.target__id
        "#, // Column aliases ending with "!" are there because sqlx does not seem to infer correctly that these columns' types are not options
        &qs.application_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("{e}");
        Hook0Problem::InternalServerError
    })?;

    let subscriptions = raw_subscriptions
        .into_iter()
        .map(|s| {
            let labels: HashMap<String, String> =
                serde_json::from_value(s.labels).unwrap_or_else(|_| HashMap::new());
            let first_label = labels
                .iter()
                .next()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .unwrap_or_else(|| (String::new(), String::new()));

            Subscription {
                application_id: qs.application_id,
                subscription_id: s.subscription__id,
                is_enabled: s.is_enabled,
                event_types: s.event_types.unwrap_or_default(),
                description: s.description,
                secret: reveal_secret.then_some(s.secret),
                metadata: serde_json::from_value(s.metadata).unwrap_or_else(|_| HashMap::new()),
                label_key: first_label.0,
                label_value: first_label.1,
                labels,
                target: serde_json::from_value(s.target_json.unwrap())
                    .expect("Could not parse subscription target"),
                created_at: s.created_at,
                updated_at: s.updated_at,
                dedicated_workers: s.dedicated_workers.unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(subscriptions))
}

#[api_v2_operation(
    summary = "Get a subscription by its ID",
    description = "Retrieves full details of a specific webhook subscription including target URL, HTTP method, headers, event types, labels, and metadata. The subscription must exist and not be deleted.",
    operation_id = "subscriptions.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp", "sdk")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id: Path<Uuid>,
) -> Result<Json<Subscription>, Hook0Problem> {
    let subscription_id = subscription_id.into_inner();

    let application_id = query_scalar!(
        "SELECT application__id FROM webhook.subscription WHERE subscription__id = $1 AND deleted_at IS NULL LIMIT 1",
        &subscription_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("{}", &e);
        Hook0Problem::InternalServerError
    })?;

    if application_id.is_none() {
        return Err(Hook0Problem::NotFound);
    }

    let application_id = application_id.expect("Could not unwrap application_id");

    authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionGet {
            application_id: &application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    // See `list`: reuse an existing write action as the probe (here the precise
    // `subscription:edit` on this very subscription, since we have its id) so
    // only configuration-capable callers see the secret. `.is_ok()` maps a
    // denial to "omit the secret" rather than a 403 (the Viewer keeps its read).
    let reveal_secret = authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionEdit {
            application_id: &application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await
    .is_ok();

    #[allow(non_snake_case)]
    struct RawSubscription {
        application__id: Uuid,
        subscription__id: Uuid,
        is_enabled: bool,
        event_types: Option<Vec<String>>,
        description: Option<String>,
        secret: Uuid,
        metadata: Value,
        labels: Value,
        target_json: Option<Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        dedicated_workers: Option<Vec<String>>,
    }

    let raw_subscription = query_as!(
        RawSubscription,
        r#"
            WITH subs AS (
                SELECT
                    s.application__id, s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.labels, s.target__id, s.created_at, s.updated_at,
                    CASE WHEN length((array_agg(set.event_type__name))[1]) > 0
                        THEN array_agg(set.event_type__name)
                        ELSE ARRAY[]::text[] END AS event_types,
                    CASE WHEN length((array_agg(w.name))[1]) > 0
                        THEN array_agg(w.name)
                        ELSE ARRAY[]::text[] END AS dedicated_workers
                FROM webhook.subscription AS s
                LEFT JOIN webhook.subscription__event_type AS set ON set.subscription__id = s.subscription__id
                LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                LEFT JOIN infrastructure.worker AS w ON w.worker__id = sw.worker__id
                WHERE s.application__id = $1 AND s.subscription__id = $2
                GROUP BY s.subscription__id
                ORDER BY s.created_at ASC
            ), targets AS (
                SELECT target__id, jsonb_build_object(
                    'type', replace(tableoid::regclass::text, 'webhook.target_', ''),
                    'method', method,
                    'url', url,
                    'headers', headers
                ) AS target_json FROM webhook.target_http
                WHERE target__id IN (SELECT target__id FROM subs)
            )
            SELECT subs.application__id AS "application__id!", subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.labels AS "labels!", subs.created_at AS "created_at!", subs.updated_at AS "updated_at!", subs.event_types, targets.target_json, subs.dedicated_workers
            FROM subs
            INNER JOIN targets ON subs.target__id = targets.target__id
            LIMIT 1
        "#, // Column aliases ending with "!" are there because sqlx does not seem to infer correctly that these columns' types are not options
        &application_id,
        &subscription_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("{e}");
        Hook0Problem::InternalServerError
    })?;

    match raw_subscription {
        Some(s) => {
            let labels: HashMap<String, String> =
                serde_json::from_value(s.labels).unwrap_or_else(|_| HashMap::new());
            let first_label = labels
                .iter()
                .next()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .unwrap_or_else(|| (String::new(), String::new()));

            Ok(Json(Subscription {
                application_id: s.application__id,
                subscription_id: s.subscription__id,
                is_enabled: s.is_enabled,
                event_types: s.event_types.unwrap_or_default(),
                description: s.description,
                secret: reveal_secret.then_some(s.secret),
                metadata: serde_json::from_value(s.metadata).unwrap_or_else(|_| HashMap::new()),
                label_key: first_label.0,
                label_value: first_label.1,
                labels,
                target: serde_json::from_value(s.target_json.unwrap())
                    .expect("Could not parse subscription target"),
                created_at: s.created_at,
                updated_at: s.updated_at,
                dedicated_workers: s.dedicated_workers.unwrap_or_default(),
            }))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct SubscriptionPost {
    application_id: Uuid,
    is_enabled: bool,
    #[validate(custom(function = "crate::validators::event_types"))]
    event_types: Vec<String>,
    #[validate(length(min = 1, max = 100))]
    description: Option<String>,
    #[validate(custom(function = "crate::validators::metadata"))]
    metadata: Option<HashMap<String, String>>,
    /// _Kept for backward compatibility, you should use `labels`_
    #[validate(non_control_character, length(min = 1, max = 100))]
    label_key: Option<String>,
    /// _Kept for backward compatibility, you should use `labels`_
    #[validate(non_control_character, length(min = 1, max = 100))]
    label_value: Option<String>,
    #[validate(custom(function = "crate::validators::labels"))]
    labels: Option<HashMap<String, String>>,
    #[validate(nested)]
    target: Target,
    #[validate(length(min = 1, max = 20))]
    dedicated_workers: Option<Vec<String>>,
}

#[api_v2_operation(
    summary = "Create a new subscription",
    description = "Creates a webhook subscription that listens for specific event types and delivers them to an HTTP endpoint. Configure the target URL, HTTP method, headers, event type filters, labels for routing, and optional metadata.",
    operation_id = "subscriptions.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp", "sdk")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<SubscriptionPost>,
) -> Result<CreatedJson<Subscription>, Hook0Problem> {
    let labels = match (&body.labels, &body.label_key, &body.label_value) {
        (Some(l), _, _) => Ok(l.to_owned()),
        (None, Some(k), Some(v)) => Ok(HashMap::from_iter([(k.to_owned(), v.to_owned())])),
        _ => Err(Hook0Problem::LabelsAmbiguity),
    }?;

    authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionCreate {
            application_id: &body.application_id,
            labels: &labels,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let organization_id = get_owner_organization(&state.db, &body.application_id)
        .await
        .unwrap_or(Uuid::nil());

    let labels = serde_json::to_value(&labels).unwrap_or_else(|_| Value::Object(Map::new()));

    let metadata = match body.metadata.as_ref() {
        Some(m) => serde_json::to_value(m.clone())
            .expect("could not serialize subscription metadata into JSON"),
        None => json!({}),
    };

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    state
        .quotas
        .enforce_subscriptions_per_application(&mut tx, &body.application_id)
        .await?;

    #[allow(non_snake_case)]
    struct RawSubscription {
        subscription__id: Uuid,
        is_enabled: bool,
        description: Option<String>,
        secret: Uuid,
        metadata: Value,
        labels: Value,
        target__id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }
    let subscription = query_as!(
            RawSubscription,
            "
                INSERT INTO webhook.subscription (subscription__id, application__id, is_enabled, description, secret, metadata, labels, target__id, created_at, updated_at)
                VALUES (public.gen_random_uuid(), $1, $2, $3, public.gen_random_uuid(), $4, $5, public.gen_random_uuid(), statement_timestamp(), statement_timestamp())
                RETURNING subscription__id, is_enabled, description, secret, metadata, labels, target__id, created_at, updated_at
            ",
            &body.application_id,
            &body.is_enabled,
            body.description,
            metadata,
            labels,
        )
            .fetch_one(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

    match &body.target {
        Target::Http {
            method,
            url,
            headers,
        } => query!(
            "
                INSERT INTO webhook.target_http (target__id, method, url, headers)
                VALUES ($1, $2, $3, $4)
            ",
            &subscription.target__id,
            method.to_uppercase(),
            url.as_str(),
            serde_json::to_value(headers).expect("could not serialize target headers into JSON"),
        )
        .execute(&mut *tx)
        .await
        .map_err(Hook0Problem::from)?,
    };

    for event_type in &body.event_types {
        query!(
                "
                    INSERT INTO webhook.subscription__event_type (application__id, subscription__id, event_type__name)
                    VALUES ($1, $2, $3)
                ",
                &body.application_id,
                &subscription.subscription__id,
                &event_type,
            )
                .execute(&mut *tx).await
                .map_err(Hook0Problem::from)?;
    }

    #[allow(non_snake_case)]
    struct RawWorkerName {
        name: String,
    }
    let allowed_dedicated_workers = query_as!(
        RawWorkerName,
        r#"
            SELECT w.name AS "name!"
            FROM infrastructure.worker AS w
            LEFT JOIN iam.organization__worker AS aw ON aw.worker__id = w.worker__id
            WHERE aw.organization__id = $1 OR w.public
        "#,
        &organization_id,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?
    .iter()
    .map(|rw| rw.name.to_owned())
    .collect::<HashSet<_>>();

    let workers: HashSet<String> = HashSet::from_iter(
        body.dedicated_workers
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .cloned(),
    );
    if !workers.is_subset(&allowed_dedicated_workers) {
        let unauthorized_workers = workers
            .difference(&allowed_dedicated_workers)
            .cloned()
            .collect::<Vec<_>>();
        return Err(Hook0Problem::UnauthorizedWorkers(unauthorized_workers));
    }

    for worker in workers {
        query!(
            "
                INSERT INTO webhook.subscription__worker (subscription__id, worker__id)
                SELECT $1, infrastructure.worker.worker__id
                FROM infrastructure.worker
                WHERE infrastructure.worker.name = $2
            ",
            &subscription.subscription__id,
            &worker,
        )
        .execute(&mut *tx)
        .await
        .map_err(Hook0Problem::from)?;
    }

    tx.commit().await.map_err(Hook0Problem::from)?;

    let labels: HashMap<String, String> =
        serde_json::from_value(subscription.labels).unwrap_or_else(|_| HashMap::new());
    let first_label = labels
        .iter()
        .next()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .unwrap_or_else(|| (String::new(), String::new()));
    let subscription = Subscription {
        application_id: body.application_id,
        subscription_id: subscription.subscription__id,
        is_enabled: subscription.is_enabled,
        event_types: body.event_types.clone(),
        description: subscription.description,
        // create/edit are Editor-only, so the secret is always exposed here.
        secret: Some(subscription.secret),
        metadata: serde_json::from_value(subscription.metadata.clone())
            .unwrap_or_else(|_| HashMap::new()),
        label_key: first_label.0,
        label_value: first_label.1,
        labels,
        target: body.target.clone(),
        created_at: subscription.created_at,
        updated_at: subscription.updated_at,
        dedicated_workers: body.dedicated_workers.clone().unwrap_or_default(),
    };

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventSubscriptionCreated {
            organization_id,
            application_id: subscription.application_id,
            subscription_id: subscription.subscription_id,
            is_enabled: subscription.is_enabled,
            event_types: subscription.event_types.to_owned(),
            description: subscription.description.to_owned(),
            metadata: subscription.metadata.to_owned(),
            labels: subscription.labels.to_owned(),
            target: subscription.target.to_owned(),
            created_at: subscription.created_at,
            updated_at: subscription.updated_at,
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(subscription))
}

#[api_v2_operation(
    summary = "Update a subscription",
    description = "Modifies an existing webhook subscription. You can update the target URL, HTTP method, headers, event types, labels, metadata, and enabled status. Disabling a subscription marks all pending webhook deliveries as failed (they won't retry if re-enabled).",
    operation_id = "subscriptions.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp", "sdk")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id: Path<Uuid>,
    body: Json<SubscriptionPost>,
) -> Result<Json<Subscription>, Hook0Problem> {
    let labels = match (&body.labels, &body.label_key, &body.label_value) {
        (Some(l), _, _) => Ok(l.to_owned()),
        (None, Some(k), Some(v)) => Ok(HashMap::from_iter([(k.to_owned(), v.to_owned())])),
        _ => Err(Hook0Problem::LabelsAmbiguity),
    }?;

    authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionEdit {
            application_id: &body.application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let organization_id = get_owner_organization(&state.db, &body.application_id)
        .await
        .unwrap_or(Uuid::nil());

    let labels = serde_json::to_value(&labels).unwrap_or_else(|_| Value::Object(Map::new()));

    let metadata = match body.metadata.as_ref() {
        Some(m) => serde_json::to_value(m.clone())
            .expect("could not serialize subscription metadata into JSON"),
        None => json!({}),
    };

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    let subscription_id = subscription_id.into_inner();

    #[allow(non_snake_case)]
    struct RawSubscription {
        subscription__id: Uuid,
        is_enabled: bool,
        description: Option<String>,
        secret: Uuid,
        metadata: Value,
        labels: Value,
        target__id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    // Update all fields including is_enabled, description, metadata, labels
    let subscription = query_as!(
        RawSubscription,
        "
            UPDATE webhook.subscription
            SET is_enabled = $1, description = $2, metadata = $3, labels = $4, updated_at = statement_timestamp()
            WHERE subscription__id = $5 AND application__id = $6 AND deleted_at IS NULL
            RETURNING subscription__id, is_enabled, description, secret, metadata, labels, target__id, created_at, updated_at
        ",
        &body.is_enabled,
        body.description,
        metadata,
        labels,
        &subscription_id,
        &body.application_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    match subscription {
        Some(s) => {
            match &body.target {
                Target::Http {
                    method,
                    url,
                    headers,
                } => query!(
                    "
                        UPDATE webhook.target_http
                        SET method = $1, url = $2, headers = $3
                        WHERE target__id = $4
                    ",
                    method.to_uppercase(),
                    url.as_str(),
                    serde_json::to_value(headers)
                        .expect("could not serialize target headers into JSON"),
                    &s.target__id
                )
                .execute(&mut *tx)
                .await
                .map_err(Hook0Problem::from)?,
            };

            query!(
                "
                    DELETE FROM webhook.subscription__event_type
                    WHERE subscription__id = $1
                ",
                &s.subscription__id,
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

            for event_type in &body.event_types {
                query!(
                    "
                        INSERT INTO webhook.subscription__event_type (application__id, subscription__id, event_type__name)
                        VALUES ($1, $2, $3)
                    ",
                    &body.application_id,
                    &s.subscription__id,
                    &event_type,
                )
                .execute(&mut *tx).await
                .map_err(Hook0Problem::from)?;
            }

            query!(
                "
                    DELETE FROM webhook.subscription__worker
                    WHERE subscription__id = $1
                ",
                &s.subscription__id,
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

            #[allow(non_snake_case)]
            struct RawWorkerName {
                name: String,
            }
            let allowed_dedicated_workers = query_as!(
                RawWorkerName,
                r#"
                    SELECT w.name AS "name!"
                    FROM infrastructure.worker AS w
                    LEFT JOIN iam.organization__worker AS aw ON aw.worker__id = w.worker__id
                    WHERE aw.organization__id = $1 OR w.public
                "#,
                &organization_id,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?
            .iter()
            .map(|rw| rw.name.to_owned())
            .collect::<HashSet<_>>();

            let workers: HashSet<String> = HashSet::from_iter(
                body.dedicated_workers
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .cloned(),
            );
            if !workers.is_subset(&allowed_dedicated_workers) {
                let unauthorized_workers = workers
                    .difference(&allowed_dedicated_workers)
                    .cloned()
                    .collect::<Vec<_>>();
                return Err(Hook0Problem::UnauthorizedWorkers(unauthorized_workers));
            }

            query!(
                "
                    DELETE FROM webhook.subscription__worker
                    WHERE subscription__id = $1
                ",
                &s.subscription__id,
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

            for worker in workers {
                query!(
                    "
                        INSERT INTO webhook.subscription__worker (subscription__id, worker__id)
                        SELECT $1, infrastructure.worker.worker__id
                        FROM infrastructure.worker
                        WHERE infrastructure.worker.name = $2
                    ",
                    &s.subscription__id,
                    &worker,
                )
                .execute(&mut *tx)
                .await
                .map_err(Hook0Problem::from)?;
            }

            // Mark pending request attempts as failed if subscription is disabled
            // This is idempotent: if already marked as failed, nothing happens
            // Run twice: the first pass may block on an attempt locked by an in-flight worker, which can
            // insert a retry before committing; that retry is invisible to the first statement's snapshot
            // but visible to the second one (READ COMMITTED). The retry delay is much longer than
            // the gap between passes, so no worker can pick the retry and create yet another one in between.
            // `paused OR NOT paused` is always true, but it lets Postgres look the subscription up in the two
            // small partial indexes of pending request attempts (request_attempt_paused_idx and
            // request_attempt_subscription_waiting_idx) instead of reading its whole history; do not remove it.
            let cancelled_request_attempts = if !body.is_enabled {
                let mut cancelled_request_attempts = 0;
                for _ in 0..2 {
                    cancelled_request_attempts += query!(
                        "
                            UPDATE webhook.request_attempt
                            SET failed_at = statement_timestamp()
                            WHERE subscription__id = $1
                              AND failed_at IS NULL
                              AND succeeded_at IS NULL
                              AND (paused OR NOT paused)
                        ",
                        &s.subscription__id
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(Hook0Problem::from)?
                    .rows_affected();
                }
                cancelled_request_attempts
            } else {
                0
            };

            tx.commit().await.map_err(Hook0Problem::from)?;

            if cancelled_request_attempts > 0 {
                report_cancelled_request_attempts(cancelled_request_attempts);
            }

            let labels: HashMap<String, String> =
                serde_json::from_value(s.labels).unwrap_or_else(|_| HashMap::new());
            let first_label = labels
                .iter()
                .next()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .unwrap_or_else(|| (String::new(), String::new()));
            let subscription = Subscription {
                application_id: body.application_id,
                subscription_id: s.subscription__id,
                is_enabled: s.is_enabled,
                event_types: body.event_types.clone(),
                description: s.description,
                // edit is Editor-only, so the secret is always exposed here.
                secret: Some(s.secret),
                metadata: serde_json::from_value(s.metadata.clone())
                    .unwrap_or_else(|_| HashMap::new()),
                label_key: first_label.0,
                label_value: first_label.1,
                labels,
                target: body.target.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
                dedicated_workers: body.dedicated_workers.clone().unwrap_or_default(),
            };

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventSubscriptionUpdated {
                    organization_id,
                    application_id: subscription.application_id,
                    subscription_id: subscription.subscription_id,
                    is_enabled: subscription.is_enabled,
                    event_types: subscription.event_types.to_owned(),
                    description: subscription.description.to_owned(),
                    metadata: subscription.metadata.to_owned(),
                    labels: subscription.labels.to_owned(),
                    target: subscription.target.to_owned(),
                    created_at: subscription.created_at,
                    updated_at: subscription.updated_at,
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(Json(subscription))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "Delete a subscription",
    description = "Permanently deletes a webhook subscription. No more events will be delivered to this endpoint, and all pending deliveries are marked as failed. This action is irreversible.",
    operation_id = "subscriptions.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp", "sdk")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionDelete {
            application_id: &qs.application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    let application_id = qs.application_id;

    #[allow(non_snake_case)]
    struct SubscriptionId {
        subscription__id: Uuid,
    }
    let subscription = query_as!(
        SubscriptionId,
        "
            SELECT subscription__id
            FROM webhook.subscription
            WHERE application__id = $1 AND subscription__id = $2
        ",
        &application_id,
        &subscription_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match subscription {
        Some(s) => {
            let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

            query!(
                "
                    UPDATE webhook.subscription
                    SET deleted_at = statement_timestamp()
                    WHERE application__id = $1 AND subscription__id = $2
                ",
                &application_id,
                &s.subscription__id
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

            // Mark pending request attempts as failed
            // Run twice: the first pass may block on an attempt locked by an in-flight worker, which can
            // insert a retry before committing; that retry is invisible to the first statement's snapshot
            // but visible to the second one (READ COMMITTED). The retry delay is much longer than
            // the gap between passes, so no worker can pick the retry and create yet another one in between.
            // `paused OR NOT paused` is always true, but it lets Postgres look the subscription up in the two
            // small partial indexes of pending request attempts (request_attempt_paused_idx and
            // request_attempt_subscription_waiting_idx) instead of reading its whole history; do not remove it.
            let mut cancelled_request_attempts = 0;
            for _ in 0..2 {
                cancelled_request_attempts += query!(
                    "
                        UPDATE webhook.request_attempt
                        SET failed_at = statement_timestamp()
                        WHERE subscription__id = $1
                          AND failed_at IS NULL
                          AND succeeded_at IS NULL
                          AND (paused OR NOT paused)
                    ",
                    &s.subscription__id
                )
                .execute(&mut *tx)
                .await
                .map_err(Hook0Problem::from)?
                .rows_affected();
            }

            tx.commit().await.map_err(Hook0Problem::from)?;

            if cancelled_request_attempts > 0 {
                report_cancelled_request_attempts(cancelled_request_attempts);
            }

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventSubscriptionRemoved {
                    organization_id: get_owner_organization(&state.db, &qs.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: qs.application_id,
                    subscription_id: s.subscription__id,
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(NoContent)
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_value;

    use super::*;

    #[test]
    fn test_deserialize_http_target_valid() {
        let url = "https://www.hook0.com";
        let input = json!({
            "type": "http",
            "method": "GET",
            "headers": {},
            "url": url,
        });
        let expected = Target::Http {
            method: "GET".to_owned(),
            url: HttpUrl(Url::parse(url).unwrap()),
            headers: HeaderMap(reqwest::header::HeaderMap::new()),
        };
        assert_eq!(from_value::<Target>(input).unwrap(), expected);
    }

    #[test]
    fn test_deserialize_http_target_wrong_scheme() {
        let url = "ftp://www.hook0.com";
        let input = json!({
            "type": "http",
            "method": "GET",
            "headers": {},
            "url": url,
        });
        assert!(
            from_value::<Target>(input)
                .unwrap_err()
                .to_string()
                .contains("scheme")
        );
    }

    #[test]
    fn test_deserialize_http_target_no_host() {
        let url = "http://";
        let input = json!({
            "type": "http",
            "method": "GET",
            "headers": {},
            "url": url,
        });
        assert!(
            from_value::<Target>(input)
                .unwrap_err()
                .to_string()
                .contains("host")
        );
    }
}

#[cfg(test)]
mod quota_race_tests {
    use crate::google_ads::test_support::{
        issue_user_token, seed_membership, seed_org, seed_user, test_state,
    };
    use crate::quotas::{QuotaLimits, QuotaValue, Quotas};
    use actix_web::{App, test, web};
    use futures_util::future::join_all;
    use sqlx::PgPool;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    const CONCURRENT_CALLERS: usize = 8;
    const SUBSCRIPTIONS_ALLOWED: QuotaValue = 1;

    /// The subscriptions limit, held against callers arriving together.
    #[sqlx::test]
    async fn concurrent_creations_cannot_take_an_application_past_its_subscription_limit(
        pool: PgPool,
    ) {
        let options = (*pool.connect_options()).clone();
        let pool = PgPoolOptions::new()
            .max_connections(CONCURRENT_CALLERS as u32 + 2)
            .connect_with(options)
            .await
            .expect("open a wider pool on the test database");

        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        let user_token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let application = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, 'race')",
        )
        .bind(application)
        .bind(org)
        .execute(&pool)
        .await
        .expect("seed the application the subscriptions hang off");

        let mut state = test_state(pool.clone(), private_key.clone(), None).await;
        state.quotas = Quotas::new(
            true,
            QuotaLimits {
                global_free_organizations_per_user_limit: QuotaValue::MAX,
                global_subscriptions_per_application_limit: SUBSCRIPTIONS_ALLOWED,
                global_applications_per_organization_limit: QuotaValue::MAX,
                global_members_per_organization_limit: QuotaValue::MAX,
                global_events_per_day_limit: QuotaValue::MAX,
                global_days_of_events_retention_limit: QuotaValue::MAX,
                global_event_types_per_application_limit: QuotaValue::MAX,
            },
        );

        let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
            db: pool.clone(),
            biscuit_private_key: private_key.clone(),
            master_api_key: None,
            enable_application_secret_compatibility: true,
        };

        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/subscriptions")
                            .wrap(biscuit_auth.clone())
                            .route("", web::post().to(super::create)),
                    )
                    .service(
                        web::scope("/event_types")
                            .wrap(biscuit_auth)
                            .route("", web::post().to(crate::handlers::event_types::create)),
                    ),
            ),
        )
        .await;

        // Through the real endpoint rather than by hand: the event type spans
        // several tables tied by foreign keys, and seeding it directly means
        // reproducing that shape in the test and keeping it in step.
        let register_event_type = test::TestRequest::post()
            .uri("/api/v1/event_types")
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .set_json(serde_json::json!({
                "application_id": application,
                "service": "race",
                "resource_type": "thing",
                "verb": "created",
            }))
            .to_request();
        let registered = test::call_service(&app, register_event_type).await;
        assert!(
            registered.status().is_success(),
            "registering the event type the subscriptions listen to failed: {}",
            registered.status()
        );

        let app_ref = &app;
        let calls = (0..CONCURRENT_CALLERS).map(|i| {
            let request = test::TestRequest::post()
                .uri("/api/v1/subscriptions")
                .insert_header(("Authorization", format!("Bearer {user_token}")))
                .set_json(serde_json::json!({
                    "application_id": application,
                    "is_enabled": true,
                    "event_types": ["race.thing.created"],
                    "description": format!("race-{i}"),
                    "labels": {"all": "yes"},
                    "target": {
                        "type": "http",
                        "method": "POST",
                        "url": "https://example.test/hook",
                        "headers": {},
                    },
                }))
                .to_request();
            test::call_service(app_ref, request)
        });

        let accepted = join_all(calls)
            .await
            .iter()
            .filter(|response| response.status().is_success())
            .count();

        let stored: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM webhook.subscription WHERE application__id = $1 AND deleted_at IS NULL",
        )
        .bind(application)
        .fetch_one(&pool)
        .await
        .expect("count the subscriptions the application ended up with");

        assert_eq!(
            stored,
            i64::from(SUBSCRIPTIONS_ALLOWED),
            "the application is left holding more subscriptions than its plan allows"
        );
        assert_eq!(
            accepted, SUBSCRIPTIONS_ALLOWED as usize,
            "more callers were told their subscription was created than the plan allows"
        );
    }
}

/// Who gets to see a subscription's signing `secret` on `list`/`get`.
///
/// The secret is the HMAC key shared with the customer's own receiver, so
/// exposing it to any org member crosses no privilege boundary. Least-privilege
/// still says a read-only Viewer has no business reading a signing credential,
/// so `list`/`get` omit the field for a Viewer while keeping it for every
/// configuration-capable caller (Editor, service token, master). The omission
/// must stay an omission: a Viewer still gets HTTP 200, never a 403. These tests
/// drive the real endpoints behind the real biscuit middleware — no mocks.
#[cfg(test)]
mod secret_visibility_tests {
    use crate::google_ads::test_support::{
        issue_user_token, seed_membership, seed_org, seed_user, test_state,
    };
    use actix_web::{App, body::MessageBody, dev::ServiceResponse, test, web};
    use serde_json::Value;
    use sqlx::PgPool;
    use uuid::Uuid;

    /// Mint a service access token through the real `POST /service_token`
    /// endpoint (black box): this drives the production code path — biscuit
    /// minting *and* the `iam.token` insert — instead of reproducing it, so the
    /// test moves in lock-step with the handler rather than drifting from a
    /// copied SQL statement. `creator_token` must belong to an Editor of `org`
    /// (service-token creation is Editor-gated). Returns the serialized biscuit.
    async fn issue_service_token<S, B>(app: &S, creator_token: &str, org: Uuid) -> String
    where
        S: actix_web::dev::Service<
                actix_http::Request,
                Response = ServiceResponse<B>,
                Error = actix_web::Error,
            >,
        B: MessageBody,
    {
        let req = test::TestRequest::post()
            .uri("/api/v1/service_token")
            .insert_header(("Authorization", format!("Bearer {creator_token}")))
            .set_json(serde_json::json!({
                "organization_id": org,
                "name": "e2e",
            }))
            .to_request();
        let resp = test::call_service(app, req).await;
        assert!(
            resp.status().is_success(),
            "creating the service token failed: {}",
            resp.status()
        );
        let body: Value = test::read_body_json(resp).await;
        body.get("biscuit")
            .and_then(Value::as_str)
            .expect("service token creation returns the serialized biscuit")
            .to_owned()
    }

    /// The single subscription object an authenticated `GET /subscriptions/` and
    /// `GET /subscriptions/{id}` return for `token`, as raw JSON. Both shapes are
    /// read here (list returns an array of one) so each caller role is checked on
    /// both endpoints. Asserts HTTP 200 on the way through — that is the CA6
    /// guarantee (a Viewer keeps read access, it does not get a 403).
    async fn read_subscription_both_ways<S, B>(
        app: &S,
        token: &str,
        application: Uuid,
        subscription_id: Uuid,
    ) -> (Value, Value)
    where
        S: actix_web::dev::Service<
                actix_http::Request,
                Response = ServiceResponse<B>,
                Error = actix_web::Error,
            >,
        B: MessageBody,
    {
        let list_req = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/subscriptions?application_id={application}"
            ))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let list_resp = test::call_service(app, list_req).await;
        assert_eq!(
            list_resp.status().as_u16(),
            200,
            "list must stay readable (never 403) regardless of secret visibility"
        );
        let list_body: Value = test::read_body_json(list_resp).await;
        let from_list = list_body
            .as_array()
            .and_then(|subs| {
                subs.iter().find(|s| {
                    s.get("subscription_id").and_then(Value::as_str)
                        == Some(subscription_id.to_string().as_str())
                })
            })
            .expect("the created subscription is present in the list")
            .clone();

        let get_req = test::TestRequest::get()
            .uri(&format!("/api/v1/subscriptions/{subscription_id}"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let get_resp = test::call_service(app, get_req).await;
        assert_eq!(
            get_resp.status().as_u16(),
            200,
            "get must stay readable (never 403) regardless of secret visibility"
        );
        let from_get: Value = test::read_body_json(get_resp).await;

        (from_list, from_get)
    }

    /// Editor and service tokens keep the secret; the human Viewer loses it — on
    /// both `list` and `get`, with every other field untouched and no 403.
    #[sqlx::test]
    async fn secret_is_visible_to_editor_and_service_but_omitted_for_viewer(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        // One org with an Editor (its creator) and a separate Viewer, so both
        // roles exercise the very same subscription.
        let editor_user = seed_user(&pool).await;
        let org = seed_org(&pool, editor_user).await;
        seed_membership(&pool, editor_user, org, "editor").await;
        let viewer_user = seed_user(&pool).await;
        seed_membership(&pool, viewer_user, org, "viewer").await;

        let application = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, 'secret-visibility')",
        )
        .bind(application)
        .bind(org)
        .execute(&pool)
        .await
        .expect("seed the application the subscription hangs off");

        let state = test_state(pool.clone(), private_key.clone(), None).await;
        let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
            db: pool.clone(),
            biscuit_private_key: private_key.clone(),
            master_api_key: None,
            enable_application_secret_compatibility: true,
        };

        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/subscriptions")
                            .wrap(biscuit_auth.clone())
                            .route("", web::post().to(super::create))
                            .route("", web::get().to(super::list))
                            .route("/{subscription_id}", web::get().to(super::get)),
                    )
                    .service(
                        web::scope("/event_types")
                            .wrap(biscuit_auth.clone())
                            .route("", web::post().to(crate::handlers::event_types::create)),
                    )
                    .service(
                        web::scope("/service_token")
                            .wrap(biscuit_auth)
                            .route("", web::post().to(crate::handlers::service_token::create)),
                    ),
            ),
        )
        .await;

        let editor_token = issue_user_token(&pool, &private_key, editor_user, org, "editor").await;
        let viewer_token = issue_user_token(&pool, &private_key, viewer_user, org, "viewer").await;
        // Reuse the real endpoint (black box) rather than mirroring its insert.
        let service_token = issue_service_token(&app, &editor_token, org).await;

        // Seed through the real endpoints: the event type spans several tables
        // tied by foreign keys, so driving it avoids reproducing that shape.
        let register_event_type = test::TestRequest::post()
            .uri("/api/v1/event_types")
            .insert_header(("Authorization", format!("Bearer {editor_token}")))
            .set_json(serde_json::json!({
                "application_id": application,
                "service": "sec",
                "resource_type": "thing",
                "verb": "created",
            }))
            .to_request();
        let registered = test::call_service(&app, register_event_type).await;
        assert!(
            registered.status().is_success(),
            "registering the event type failed: {}",
            registered.status()
        );

        let create_req = test::TestRequest::post()
            .uri("/api/v1/subscriptions")
            .insert_header(("Authorization", format!("Bearer {editor_token}")))
            .set_json(serde_json::json!({
                "application_id": application,
                "is_enabled": true,
                "event_types": ["sec.thing.created"],
                "description": "secret-visibility",
                "labels": {"all": "yes"},
                "target": {
                    "type": "http",
                    "method": "POST",
                    "url": "https://example.test/hook",
                    "headers": {},
                },
            }))
            .to_request();
        let create_resp = test::call_service(&app, create_req).await;
        assert!(
            create_resp.status().is_success(),
            "creating the subscription failed: {}",
            create_resp.status()
        );
        let created: Value = test::read_body_json(create_resp).await;
        // CA5: create is Editor-only and still returns the secret.
        let created_secret = created
            .get("secret")
            .and_then(Value::as_str)
            .expect("create returns the secret to its Editor caller")
            .to_owned();
        let subscription_id: Uuid = created
            .get("subscription_id")
            .and_then(Value::as_str)
            .expect("create returns a subscription id")
            .parse()
            .expect("the subscription id is a uuid");

        // CA3: an Editor reads the full secret on both list and get.
        let (editor_list, editor_get) =
            read_subscription_both_ways(&app, &editor_token, application, subscription_id).await;
        assert_eq!(
            editor_list.get("secret").and_then(Value::as_str),
            Some(created_secret.as_str()),
            "an Editor must still see the secret on list"
        );
        assert_eq!(
            editor_get.get("secret").and_then(Value::as_str),
            Some(created_secret.as_str()),
            "an Editor must still see the secret on get"
        );

        // CA4: a service token keeps the secret (guards provisioning).
        let (service_list, service_get) =
            read_subscription_both_ways(&app, &service_token, application, subscription_id).await;
        assert_eq!(
            service_list.get("secret").and_then(Value::as_str),
            Some(created_secret.as_str()),
            "a service token must still see the secret on list"
        );
        assert_eq!(
            service_get.get("secret").and_then(Value::as_str),
            Some(created_secret.as_str()),
            "a service token must still see the secret on get"
        );

        // CA1/CA2/CA6: the Viewer reads both endpoints as 200 (asserted inside
        // the helper) but the `secret` field is absent entirely — not null, not
        // masked, just gone — while the rest of the payload is intact.
        let (viewer_list, viewer_get) =
            read_subscription_both_ways(&app, &viewer_token, application, subscription_id).await;
        assert!(
            viewer_list.get("secret").is_none(),
            "a Viewer must not receive the secret on list, got: {viewer_list}"
        );
        assert!(
            viewer_get.get("secret").is_none(),
            "a Viewer must not receive the secret on get, got: {viewer_get}"
        );
        // Everything else a Viewer can already read stays present.
        assert_eq!(
            viewer_get.get("subscription_id").and_then(Value::as_str),
            Some(subscription_id.to_string().as_str()),
            "omitting the secret must not drop the rest of the subscription"
        );
        assert!(
            viewer_get.get("target").is_some() && viewer_get.get("is_enabled").is_some(),
            "the Viewer still sees the non-secret fields"
        );
    }
}

#[cfg(test)]
mod cancel_pending_request_attempts_tests {
    use crate::google_ads::test_support::{
        issue_user_token, seed_event, seed_membership, seed_org, seed_subscription, seed_user,
        test_state,
    };
    use actix_web::{App, test, web};
    use sqlx::PgPool;

    /// Disabling a subscription fails all its pending request attempts, paused or not: the cancel query
    /// filters on `paused OR NOT paused` only so that Postgres uses the partial indexes.
    #[sqlx::test]
    async fn disabling_a_subscription_fails_its_paused_and_unpaused_pending_request_attempts(
        pool: PgPool,
    ) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        let user_token = issue_user_token(&pool, &private_key, user, org, "editor").await;
        let (application_id, event_id) = seed_event(&pool, org).await;
        let subscription_id = seed_subscription(&pool, application_id).await;
        sqlx::query(
            "
                INSERT INTO webhook.request_attempt (event__id, subscription__id, application__id, paused)
                VALUES ($1, $2, $3, true), ($1, $2, $3, false)
            ",
        )
        .bind(event_id)
        .bind(subscription_id)
        .bind(application_id)
        .execute(&pool)
        .await
        .expect("seed a paused and an unpaused pending request attempt");

        let state = test_state(pool.clone(), private_key.clone(), None).await;
        let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
            db: pool.clone(),
            biscuit_private_key: private_key.clone(),
            master_api_key: None,
            enable_application_secret_compatibility: true,
        };
        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1").service(
                    web::scope("/subscriptions")
                        .wrap(biscuit_auth)
                        .route("/{subscription_id}", web::put().to(super::edit)),
                ),
            ),
        )
        .await;

        let disable = test::TestRequest::put()
            .uri(&format!("/api/v1/subscriptions/{subscription_id}"))
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .set_json(serde_json::json!({
                "application_id": application_id,
                "is_enabled": false,
                "event_types": ["test.resource.created"],
                "description": "cancel-pending",
                "labels": {"e2e": "1"},
                "target": {
                    "type": "http",
                    "method": "POST",
                    "url": "https://example.test/hook",
                    "headers": {},
                },
            }))
            .to_request();
        let response = test::call_service(&app, disable).await;
        assert!(
            response.status().is_success(),
            "disabling the subscription failed: {}",
            response.status()
        );

        let still_pending: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM webhook.request_attempt WHERE subscription__id = $1 AND failed_at IS NULL",
        )
        .bind(subscription_id)
        .fetch_one(&pool)
        .await
        .expect("count the pending request attempts left");
        assert_eq!(
            still_pending, 0,
            "disabling the subscription must fail its paused and unpaused pending request attempts"
        );
    }
}
