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
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::*;
use crate::iam::{can_access_application, Role};

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

/// List subscriptions
#[api_v2_operation]
pub async fn list(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Subscription>>, UnexpectedError> {
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
            "
                WITH subs AS (
                    SELECT
                        s.subscription__id, s.is_enabled, s.description, s.secret, s.metadata, s.label_key, s.label_value, s.target__id, s.created_at,
                        CASE WHEN length((array_agg(set.event_type__name))[1]) > 0
                            THEN array_agg(set.event_type__name)
                            ELSE ARRAY[]::text[] END AS event_types
                    FROM webhook.subscription AS s
                    NATURAL LEFT JOIN webhook.subscription__event_type AS set
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
                SELECT subs.subscription__id, subs.is_enabled, subs.description, subs.secret, subs.metadata, subs.label_key, subs.label_value, subs.created_at, subs.event_types, targets.target_json
                FROM subs
                INNER JOIN targets ON subs.target__id = targets.target__id
            ",
            &qs.application_id,
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| {error!("{}", &e); UnexpectedError::InternalServerError})?;

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
        Err(UnexpectedError::Forbidden)
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

/// Create a new subscription
#[api_v2_operation]
pub async fn add(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    body: Json<SubscriptionPost>,
) -> Result<CreatedJson<Subscription>, CreateError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &body.application_id,
        &Role::Editor,
    )
    .await
    {
        let mut tx = state
            .db
            .begin()
            .await
            .map_err(|_| CreateError::InternalServerError)?;

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
        .map_err(|_| CreateError::InternalServerError)?;

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
                serde_json::to_value(headers)
                    .expect("could not serialize target headers into JSON"),
            )
            .execute(&mut tx)
            .await
            .map_err(|_| CreateError::InternalServerError)?,
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
            .map_err(|_| CreateError::InternalServerError)?;
        }

        tx.commit()
            .await
            .map_err(|_| CreateError::InternalServerError)?;

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
    } else {
        Err(CreateError::Forbidden)
    }
}

/// Edit an application secret
#[api_v2_operation]
pub async fn edit(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    subscription_id: Path<Uuid>,
    body: Json<SubscriptionPost>,
) -> Result<Json<Subscription>, EditError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &body.application_id,
        &Role::Editor,
    )
    .await
    {
        let mut tx = state
            .db
            .begin()
            .await
            .map_err(|_| EditError::InternalServerError)?;

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
            .map_err(|_| EditError::InternalServerError)?;

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
                    .map_err(|_| EditError::InternalServerError)?,
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
                .map_err(|_| EditError::InternalServerError)?;

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
                    .map_err(|_| EditError::InternalServerError)?;
                }

                tx.commit()
                    .await
                    .map_err(|_| EditError::InternalServerError)?;

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
            None => Err(EditError::NotFound),
        }
    } else {
        Err(EditError::Forbidden)
    }
}

/// Destroy a subscription
#[api_v2_operation]
pub async fn destroy(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    subscription_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, ShowError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &qs.application_id,
        &Role::Editor,
    )
    .await
    {
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
        .map_err(|_| ShowError::InternalServerError)?;

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
                .map_err(|_| ShowError::InternalServerError)?;
                Ok(NoContent)
            }
            None => Err(ShowError::NotFound),
        }
    } else {
        Err(ShowError::Forbidden)
    }
}
