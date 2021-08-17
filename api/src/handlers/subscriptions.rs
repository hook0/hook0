use std::collections::HashMap;

use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use chrono::{DateTime, Utc};
use log::error;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query, ReqData},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::iam::{can_access_application, Role};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Subscription {
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
}

#[derive(Debug, Clone, Deserialize, Serialize, Apiv2Schema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Target {
    Http {
        method: String,
        url: String,
        headers: HashMap<String, String>,
    },
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
    unstructured_claims: ReqData<UnstructuredClaims>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Subscription>>, Hook0Problem> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &qs.application_id,
        &Role::Viewer,
    )
    .await
    {
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
        }

        let raw_subscriptions = query_as!(
            RawSubscription,
            r#"
                WITH subs AS (
                    SELECT
                        s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.label_key, s.label_value, s.target__id, s.created_at,
                        CASE WHEN length((array_agg(set.event_type__name))[1]) > 0
                            THEN array_agg(set.event_type__name)
                            ELSE ARRAY[]::text[] END AS event_types
                    FROM webhook.subscription AS s
                    LEFT JOIN webhook.subscription__event_type AS set ON set.subscription__id = s.subscription__id
                    WHERE s.application__id = $1
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
                SELECT subs.subscription__id AS "subscription__id!", subs.is_enabled AS "is_enabled!", subs.description, subs.secret AS "secret!", subs.metadata AS "metadata!", subs.label_key AS "label_key!", subs.label_value AS "label_value!", subs.created_at AS "created_at!", subs.event_types, targets.target_json
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
                subscription_id: s.subscription__id,
                is_enabled: s.is_enabled,
                event_types: s.event_types.clone().unwrap_or_else(Vec::new),
                description: s.description.to_owned(),
                secret: s.secret,
                metadata: serde_json::from_value(s.metadata.clone())
                    .unwrap_or_else(|_| HashMap::new()),
                label_key: s.label_key.to_owned(),
                label_value: s.label_value.to_owned(),
                target: serde_json::from_value(s.target_json.clone().unwrap())
                    .expect("Could not parse subscription target"),
                created_at: s.created_at,
            })
            .collect::<Vec<_>>();

        Ok(Json(subscriptions))
    } else {
        Err(Hook0Problem::Forbidden)
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct SubscriptionPost {
    application_id: Uuid,
    is_enabled: bool,
    event_types: Vec<String>,
    description: Option<String>,
    metadata: HashMap<String, Value>,
    label_key: String,
    label_value: String,
    target: Target,
}

#[api_v2_operation(
    summary = "Create a new subscription",
    description = "A subscription let your customers subscribe to events. Events will be sent through the defined medium inside the subscription (e.g. HTTP POST request) as a webhook.",
    operation_id = "subscriptions.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn add(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    body: Json<SubscriptionPost>,
) -> Result<CreatedJson<Subscription>, Hook0Problem> {
    if !can_access_application(
        &state.db,
        &unstructured_claims,
        &body.application_id,
        &Role::Editor,
    )
    .await
    {
        return Err(Hook0Problem::Forbidden);
    }

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
            serde_json::to_value(body.metadata.clone()).expect("could not serialize subscription metadata into JSON"),
            &body.label_key,
            &body.label_value,
        )
            .fetch_one(&mut tx)
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
            method,
            url,
            serde_json::to_value(headers).expect("could not serialize target headers into JSON"),
        )
        .execute(&mut tx)
        .await
        .map_err(Hook0Problem::from)?,
    };

    for event_type in &body.event_types {
        query!(
                "
                    INSERT INTO webhook.subscription__event_type (subscription__id, event_type__name)
                    VALUES ($1, $2)
                ",
                &subscription.subscription__id,
                &event_type,
            )
                .execute(&mut tx).await
                .map_err(Hook0Problem::from)?;
    }

    tx.commit().await.map_err(Hook0Problem::from)?;

    Ok(CreatedJson(Subscription {
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
    }))
}

#[api_v2_operation(
    summary = "Update a subscription",
    description = "",
    operation_id = "subscriptions.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn update(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    subscription_id: Path<Uuid>,
    body: Json<SubscriptionPost>,
) -> Result<Json<Subscription>, Hook0Problem> {
    if !can_access_application(
        &state.db,
        &unstructured_claims,
        &body.application_id,
        &Role::Editor,
    )
    .await
    {
        return Err(Hook0Problem::Forbidden);
    }

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
                    WHERE subscription__id = $6 AND application__id = $7
                    RETURNING subscription__id, is_enabled, description, secret, metadata, label_key, label_value, target__id, created_at
                ",
                &body.is_enabled,
                body.description,
                serde_json::to_value(body.metadata.clone()).expect("could not serialize subscription metadata into JSON"),
                &body.label_key,
                &body.label_value,
                &subscription_id.into_inner(),
                &body.application_id
            )
        .fetch_optional(&mut tx)
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
                    method,
                    url,
                    serde_json::to_value(headers)
                        .expect("could not serialize target headers into JSON"),
                    &s.target__id
                )
                .execute(&mut tx)
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
            .execute(&mut tx)
            .await
            .map_err(Hook0Problem::from)?;

            for event_type in &body.event_types {
                query!(
                    "
                        INSERT INTO webhook.subscription__event_type (subscription__id, event_type__name)
                        VALUES ($1, $2)
                    ",
                    &s.subscription__id,
                    &event_type,
                )
                .execute(&mut tx).await
                .map_err(Hook0Problem::from)?;
            }

            tx.commit().await.map_err(Hook0Problem::from)?;

            Ok(Json(Subscription {
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
            }))
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
    unstructured_claims: ReqData<UnstructuredClaims>,
    subscription_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    if !can_access_application(
        &state.db,
        &unstructured_claims,
        &qs.application_id,
        &Role::Editor,
    )
    .await
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
                    DELETE FROM webhook.subscription
                    WHERE application__id = $1 AND subscription__id = $2
                ",
                &application_id,
                &s.subscription__id
            )
            .execute(&state.db)
            .await
            .map_err(Hook0Problem::from)?;
            Ok(NoContent)
        }
        None => Err(Hook0Problem::NotFound),
    }
}
