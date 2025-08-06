use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use log::error;
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
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::hook0_client::{
    EventSubscriptionCreated, EventSubscriptionRemoved, EventSubscriptionUpdated, Hook0ClientEvent,
};
use crate::iam::{Action, authorize_for_application, get_owner_organization};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::quotas::Quota;
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
    pub secret: Uuid,
    pub metadata: HashMap<String, Value>,
    /// _Kept for backward compatibility, you should use `labels`_
    pub label_key: String,
    /// _Kept for backward compatibility, you should use `labels`_
    pub label_value: String,
    pub labels: HashMap<String, String>,
    pub target: Target,
    pub created_at: DateTime<Utc>,
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
    description = "Retrieves all active event subscriptions for a given application. A subscription defines how and where event notifications will be sent.",
    operation_id = "subscriptions.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Subscription>>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionList {
            application_id: &qs.application_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
        dedicated_workers: Option<Vec<String>>,
    }

    let raw_subscriptions = query_as!(
        RawSubscription,
        r#"
            WITH subs AS (
                SELECT
                    s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.labels, s.target__id, s.created_at,
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
            SELECT subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.labels AS "labels!", subs.created_at AS "created_at!", subs.event_types, targets.target_json, subs.dedicated_workers
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
                secret: s.secret,
                metadata: serde_json::from_value(s.metadata).unwrap_or_else(|_| HashMap::new()),
                label_key: first_label.0,
                label_value: first_label.1,
                labels,
                target: serde_json::from_value(s.target_json.unwrap())
                    .expect("Could not parse subscription target"),
                created_at: s.created_at,
                dedicated_workers: s.dedicated_workers.unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(subscriptions))
}

#[api_v2_operation(
    summary = "Get a subscription by its id",
    description = "Retrieves details of a specific subscription if it belongs to the specified application and has not been deleted.",
    operation_id = "subscriptions.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
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

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionGet {
            application_id: &application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
        dedicated_workers: Option<Vec<String>>,
    }

    let raw_subscription = query_as!(
        RawSubscription,
        r#"
            WITH subs AS (
                SELECT
                    s.application__id, s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.labels, s.target__id, s.created_at,
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
            SELECT subs.application__id AS "application__id!", subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.labels AS "labels!", subs.created_at AS "created_at!", subs.event_types, targets.target_json, subs.dedicated_workers
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
                secret: s.secret,
                metadata: serde_json::from_value(s.metadata).unwrap_or_else(|_| HashMap::new()),
                label_key: first_label.0,
                label_value: first_label.1,
                labels,
                target: serde_json::from_value(s.target_json.unwrap())
                    .expect("Could not parse subscription target"),
                created_at: s.created_at,
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
    metadata: Option<HashMap<String, Value>>,
    /// _Kept for backward compatibility, you should use `labels`_
    #[validate(non_control_character, length(min = 1, max = 100))]
    label_key: Option<String>,
    /// _Kept for backward compatibility, you should use `labels`_
    #[validate(non_control_character, length(min = 1, max = 100))]
    label_value: Option<String>,
    #[validate(custom(function = "crate::validators::labels"))]
    labels: Option<HashMap<String, Value>>,
    #[validate(nested)]
    target: Target,
    #[validate(length(min = 1, max = 20))]
    dedicated_workers: Option<Vec<String>>,
}

#[api_v2_operation(
    summary = "Create a new subscription",
    description = "Creates a new event subscription for an application. This allows clients to receive event notifications via a webhook or another defined target.",
    operation_id = "subscriptions.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<SubscriptionPost>,
) -> Result<CreatedJson<Subscription>, Hook0Problem> {
    let labels = match (&body.labels, &body.label_key, &body.label_value) {
        (Some(l), _, _) => Ok(l.to_owned()),
        (None, Some(k), Some(v)) => Ok(HashMap::from_iter([(
            k.to_owned(),
            Value::String(v.to_owned()),
        )])),
        _ => Err(Hook0Problem::LabelsAmbiguity),
    }?;

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionCreate {
            application_id: &body.application_id,
            labels: &labels,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let organization_id = get_owner_organization(&state.db, &body.application_id)
        .await
        .unwrap_or(Uuid::nil());

    let quota_limit = state
        .quotas
        .get_limit_for_application(
            &state.db,
            Quota::SubscriptionsPerApplication,
            &body.application_id,
        )
        .await?;

    let quota_current = query_scalar!(
        r#"
            SELECT COUNT(subscription__id) AS "val!"
            FROM webhook.subscription
            WHERE application__id = $1
                and deleted_at IS NULL
        "#,
        &body.application_id,
    )
    .fetch_one(&state.db)
    .await?;

    if quota_current >= i64::from(quota_limit) {
        return Err(Hook0Problem::TooManySubscriptionsPerApplication(
            quota_limit,
        ));
    }

    let labels = serde_json::to_value(&labels).unwrap_or_else(|_| Value::Object(Map::new()));

    let metadata = match body.metadata.as_ref() {
        Some(m) => serde_json::to_value(m.clone())
            .expect("could not serialize subscription metadata into JSON"),
        None => json!({}),
    };

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

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
    }
    let subscription = query_as!(
            RawSubscription,
            "
                INSERT INTO webhook.subscription (subscription__id, application__id, is_enabled, description, secret, metadata, labels, target__id, created_at)
                VALUES (public.gen_random_uuid(), $1, $2, $3, public.gen_random_uuid(), $4, $5, public.gen_random_uuid(), statement_timestamp())
                RETURNING subscription__id, is_enabled, description, secret, metadata, labels, target__id, created_at
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
        secret: subscription.secret,
        metadata: serde_json::from_value(subscription.metadata.clone())
            .unwrap_or_else(|_| HashMap::new()),
        label_key: first_label.0,
        label_value: first_label.1,
        labels,
        target: body.target.clone(),
        created_at: subscription.created_at,
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
    description = "Modifies an existing subscription, including its event types, target configuration, or metadata. The subscription must belong to the specified application.",
    operation_id = "subscriptions.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
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
        (None, Some(k), Some(v)) => Ok(HashMap::from_iter([(
            k.to_owned(),
            Value::String(v.to_owned()),
        )])),
        _ => Err(Hook0Problem::LabelsAmbiguity),
    }?;

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionEdit {
            application_id: &body.application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
    }
    let subscription = query_as!(
                RawSubscription,
                "
                    UPDATE webhook.subscription
                    SET is_enabled = $1, description = $2, metadata = $3, labels = $4
                    WHERE subscription__id = $5 AND application__id = $6 AND deleted_at IS NULL
                    RETURNING subscription__id, is_enabled, description, secret, metadata, labels, target__id, created_at
                ",
                &body.is_enabled, // updatable
                body.description, // updatable
                metadata, // updatable (our validator layer ensure this will never fail)
                labels, // updatable
                &subscription_id.into_inner(), // read-only
                &body.application_id // read-only
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

            tx.commit().await.map_err(Hook0Problem::from)?;

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
                secret: s.secret,
                metadata: serde_json::from_value(s.metadata.clone())
                    .unwrap_or_else(|_| HashMap::new()),
                label_key: first_label.0,
                label_value: first_label.1,
                labels,
                target: body.target.clone(),
                created_at: s.created_at,
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
    description = "Marks a subscription as deleted, preventing any further event notifications from being sent. This operation is irreversible.",
    operation_id = "subscriptions.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionDelete {
            application_id: &qs.application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
            query!(
                "
                    UPDATE webhook.subscription
                    SET deleted_at = statement_timestamp()
                    WHERE application__id = $1 AND subscription__id = $2
                ",
                &application_id,
                &s.subscription__id
            )
            .execute(&state.db)
            .await
            .map_err(Hook0Problem::from)?;

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
