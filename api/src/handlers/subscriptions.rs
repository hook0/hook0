use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson, NoContent};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use sqlx::{query, query_as, query_scalar};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventSubscriptionCreated, EventSubscriptionRemoved, EventSubscriptionUpdated, Hook0ClientEvent,
};
use crate::iam::{authorize_for_application, get_owner_organization, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Subscription {
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub is_enabled: bool,
    pub event_types: Vec<String>,
    pub description: Option<String>,
    pub secret: Uuid,
    pub metadata: HashMap<String, Value>,
    pub label_key: String,
    pub label_value: String,
    pub target: Target,
    pub created_at: DateTime<Utc>,
    pub dedicated_workers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Apiv2Schema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Target {
    Http {
        method: String,
        #[serde(deserialize_with = "deserialize_http_url")]
        url: HttpUrl,
        headers: HashMap<String, String>,
    },
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
    description = "List all subscriptions created by customers against the application events",
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
        label_key: String,
        label_value: String,
        target_json: Option<Value>,
        created_at: DateTime<Utc>,
        dedicated_workers: Option<Vec<String>>,
    }

    let raw_subscriptions = query_as!(
        RawSubscription,
        r#"
            WITH subs AS (
                SELECT
                    s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.label_key, s.label_value, s.target__id, s.created_at,
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
            SELECT subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.label_key AS "label_key!", subs.label_value AS "label_value!", subs.created_at AS "created_at!", subs.event_types, targets.target_json, subs.dedicated_workers
            FROM subs
            INNER JOIN targets ON subs.target__id = targets.target__id
        "#, // Column aliases ending with "!" are there because sqlx does not seem to infer correctly that these columns' types are not options
        &qs.application_id,
    )
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("{}", &e);
            Hook0Problem::InternalServerError
        })?;

    let subscriptions = raw_subscriptions
        .iter()
        .map(|s| Subscription {
            application_id: qs.application_id,
            subscription_id: s.subscription__id,
            is_enabled: s.is_enabled,
            event_types: s.event_types.clone().unwrap_or_default(),
            description: s.description.to_owned(),
            secret: s.secret,
            metadata: serde_json::from_value(s.metadata.clone()).unwrap_or_else(|_| HashMap::new()),
            label_key: s.label_key.to_owned(),
            label_value: s.label_value.to_owned(),
            target: serde_json::from_value(s.target_json.clone().unwrap())
                .expect("Could not parse subscription target"),
            created_at: s.created_at,
            dedicated_workers: s.dedicated_workers.clone().unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    Ok(Json(subscriptions))
}

#[api_v2_operation(
    summary = "Get a subscription by its id",
    description = "",
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
        label_key: String,
        label_value: String,
        target_json: Option<Value>,
        created_at: DateTime<Utc>,
        dedicated_workers: Option<Vec<String>>,
    }

    let raw_subscription = query_as!(
        RawSubscription,
        r#"
            WITH subs AS (
                SELECT
                    s.application__id, s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.label_key, s.label_value, s.target__id, s.created_at,
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
            SELECT subs.application__id AS "application__id!", subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.label_key AS "label_key!", subs.label_value AS "label_value!", subs.created_at AS "created_at!", subs.event_types, targets.target_json, subs.dedicated_workers
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
            error!("{}", &e);
            Hook0Problem::InternalServerError
        })?;

    match raw_subscription {
        Some(s) => Ok(Json(Subscription {
            application_id: s.application__id,
            subscription_id: s.subscription__id,
            is_enabled: s.is_enabled,
            event_types: s.event_types.clone().unwrap_or_default(),
            description: s.description.to_owned(),
            secret: s.secret,
            metadata: serde_json::from_value(s.metadata.clone()).unwrap_or_else(|_| HashMap::new()),
            label_key: s.label_key.to_owned(),
            label_value: s.label_value.to_owned(),
            target: serde_json::from_value(s.target_json.clone().unwrap())
                .expect("Could not parse subscription target"),
            created_at: s.created_at,
            dedicated_workers: s.dedicated_workers.unwrap_or_default(),
        })),
        None => Err(Hook0Problem::NotFound),
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct SubscriptionPost {
    application_id: Uuid,
    is_enabled: bool,
    #[validate(custom = "crate::validators::event_types")]
    event_types: Vec<String>,
    #[validate(length(min = 1, max = 100))]
    description: Option<String>,
    #[validate(custom = "crate::validators::metadata")]
    metadata: Option<HashMap<String, Value>>,
    #[validate(non_control_character, length(min = 1, max = 100))]
    label_key: String,
    #[validate(non_control_character, length(min = 1, max = 100))]
    label_value: String,
    target: Target,
    #[validate(length(min = 1, max = 20))]
    dedicated_workers: Option<Vec<String>>,
}

#[api_v2_operation(
    summary = "Create a new subscription",
    description = "A subscription let your customers subscribe to events. Events will be sent through the defined medium inside the subscription (e.g. HTTP POST request) as a webhook.",
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
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionCreate {
            application_id: &body.application_id,
            label_key: &body.label_key,
            label_value: &body.label_value,
        },
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
        label_key: String,
        label_value: String,
        target__id: Uuid,
        created_at: DateTime<Utc>,
    }
    let subscription = query_as!(
            RawSubscription,
            "
                INSERT INTO webhook.subscription (subscription__id, application__id, is_enabled, description, secret, metadata, label_key, label_value, target__id, created_at)
                VALUES (public.gen_random_uuid(), $1, $2, $3, public.gen_random_uuid(), $4, $5, $6, public.gen_random_uuid(), statement_timestamp())
                RETURNING subscription__id, is_enabled, description, secret, metadata, label_key, label_value, target__id, created_at
            ",
            &body.application_id,
            &body.is_enabled,
            body.description,
            metadata,
            &body.label_key,
            &body.label_value,
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

    let subscription = Subscription {
        application_id: body.application_id,
        subscription_id: subscription.subscription__id,
        is_enabled: subscription.is_enabled,
        event_types: body.event_types.clone(),
        description: subscription.description,
        secret: subscription.secret,
        metadata: serde_json::from_value(subscription.metadata.clone())
            .unwrap_or_else(|_| HashMap::new()),
        label_key: subscription.label_key,
        label_value: subscription.label_value,
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
            label_key: subscription.label_key.to_owned(),
            label_value: subscription.label_value.to_owned(),
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
    description = "",
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
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionEdit {
            application_id: &body.application_id,
            subscription_id: &subscription_id,
        },
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
        label_key: String,
        label_value: String,
        target__id: Uuid,
        created_at: DateTime<Utc>,
    }
    let subscription = query_as!(
                RawSubscription,
                "
                    UPDATE webhook.subscription
                    SET is_enabled = $1, description = $2, metadata = $3, label_key = $4, label_value = $5
                    WHERE subscription__id = $6 AND application__id = $7 AND deleted_at IS NULL
                    RETURNING subscription__id, is_enabled, description, secret, metadata, label_key, label_value, target__id, created_at
                ",
                &body.is_enabled, // updatable
                body.description, // updatable
                metadata, // updatable (our validator layer ensure this will never fail)
                &body.label_key, // updatable
                &body.label_value, // updatable
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

            let subscription = Subscription {
                application_id: body.application_id,
                subscription_id: s.subscription__id,
                is_enabled: s.is_enabled,
                event_types: body.event_types.clone(),
                description: s.description,
                secret: s.secret,
                metadata: serde_json::from_value(s.metadata.clone())
                    .unwrap_or_else(|_| HashMap::new()),
                label_key: s.label_key,
                label_value: s.label_value,
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
                    label_key: subscription.label_key.to_owned(),
                    label_value: subscription.label_value.to_owned(),
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
    description = "",
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
            headers: HashMap::new(),
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
        assert!(from_value::<Target>(input)
            .unwrap_err()
            .to_string()
            .contains("scheme"));
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
        assert!(from_value::<Target>(input)
            .unwrap_err()
            .to_string()
            .contains("host"));
    }
}
