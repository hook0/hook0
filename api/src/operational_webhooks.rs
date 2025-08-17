use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use uuid::Uuid;

use crate::hook0_client::{Event, INSTANCE_LABEL, INSTANCE_VALUE, ORGANIZATION_LABEL, APPLICATION_LABEL};

/// Event sent when an endpoint is automatically disabled
#[derive(Debug, Clone, Serialize)]
pub struct EventEndpointDisabled {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub endpoint_url: String,
    pub disabled_at: DateTime<Utc>,
    pub failure_count: i64,
}

impl Event for EventEndpointDisabled {
    fn event_type(&self) -> &'static str {
        "api.endpoint.disabled"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (INSTANCE_LABEL.to_owned(), to_value(INSTANCE_VALUE).unwrap()),
            (ORGANIZATION_LABEL.to_owned(), to_value(self.organization_id).unwrap()),
            (APPLICATION_LABEL.to_owned(), to_value(self.application_id).unwrap()),
            ("subscription".to_owned(), to_value(self.subscription_id).unwrap()),
        ]
    }
}

/// Event sent as a warning when an endpoint has been failing for N days
#[derive(Debug, Clone, Serialize)]
pub struct EventEndpointWarning {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub endpoint_url: String,
    pub failing_since: DateTime<Utc>,
    pub failure_count: i64,
}

impl Event for EventEndpointWarning {
    fn event_type(&self) -> &'static str {
        "api.endpoint.warning"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (INSTANCE_LABEL.to_owned(), to_value(INSTANCE_VALUE).unwrap()),
            (ORGANIZATION_LABEL.to_owned(), to_value(self.organization_id).unwrap()),
            (APPLICATION_LABEL.to_owned(), to_value(self.application_id).unwrap()),
            ("subscription".to_owned(), to_value(self.subscription_id).unwrap()),
        ]
    }
}

/// Event sent when an endpoint recovers after being in a failure state
#[derive(Debug, Clone, Serialize)]
pub struct EventEndpointRecovered {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub endpoint_url: String,
    pub recovered_at: DateTime<Utc>,
}

impl Event for EventEndpointRecovered {
    fn event_type(&self) -> &'static str {
        "api.endpoint.recovered"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (INSTANCE_LABEL.to_owned(), to_value(INSTANCE_VALUE).unwrap()),
            (ORGANIZATION_LABEL.to_owned(), to_value(self.organization_id).unwrap()),
            (APPLICATION_LABEL.to_owned(), to_value(self.application_id).unwrap()),
            ("subscription".to_owned(), to_value(self.subscription_id).unwrap()),
        ]
    }
}

/// Event sent when all attempts for a message have been exhausted
#[derive(Debug, Clone, Serialize)]
pub struct EventMessageAttemptExhausted {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub message_id: Uuid,
    pub attempts: i32,
}

impl Event for EventMessageAttemptExhausted {
    fn event_type(&self) -> &'static str {
        "api.message.attempt.exhausted"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (INSTANCE_LABEL.to_owned(), to_value(INSTANCE_VALUE).unwrap()),
            (ORGANIZATION_LABEL.to_owned(), to_value(self.organization_id).unwrap()),
            (APPLICATION_LABEL.to_owned(), to_value(self.application_id).unwrap()),
            ("subscription".to_owned(), to_value(self.subscription_id).unwrap()),
            ("message".to_owned(), to_value(self.message_id).unwrap()),
        ]
    }
}

/// API handlers for operational webhook configuration
use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use paperclip::actix::web::{Data, Json, Path};
use paperclip::actix::{Apiv2Schema, NoContent, api_v2_operation};
use sqlx::{query, PgPool};
use std::collections::HashMap;

use crate::iam::{Action, authorize_for_organization};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

/// Operational webhook configuration
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct OperationalWebhookConfig {
    pub config_id: Uuid,
    pub organization_id: Uuid,
    pub event_type: String,
    pub target_url: String,
    pub headers: HashMap<String, String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating or updating operational webhook config
#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct OperationalWebhookConfigInput {
    pub event_type: String,
    pub target_url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// List operational webhook configurations
#[api_v2_operation(
    summary = "List operational webhook configurations",
    description = "List all operational webhook configurations for an organization",
    operation_id = "operational_webhooks.list",
    produces = "application/json",
    tags("OperationalWebhooks")
)]
pub async fn list_configs(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    organization_id: Path<Uuid>,
) -> Result<Json<Vec<OperationalWebhookConfig>>, Hook0Problem> {
    let organization_id = organization_id.into_inner();
    
    // Authorize access to organization
    authorize_for_organization(&biscuit, &organization_id, Action::OrganizationGet)?;
    
    let configs = query!(
        r#"
        SELECT 
            config__id as config_id,
            organization__id as organization_id,
            event_type,
            target_url,
            headers as "headers: sqlx::types::Json<HashMap<String, String>>",
            is_enabled,
            created_at,
            updated_at
        FROM webhook.operational_webhook_config
        WHERE organization__id = $1
        ORDER BY event_type
        "#,
        organization_id
    )
    .fetch_all(db.as_ref())
    .await?
    .into_iter()
    .map(|row| {
        OperationalWebhookConfig {
            config_id: row.config_id,
            organization_id: row.organization_id,
            event_type: row.event_type,
            target_url: row.target_url,
            headers: row.headers.unwrap_or(sqlx::types::Json(HashMap::new())).0, // Extract from JSON wrapper
            is_enabled: row.is_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    })
    .collect();
    
    Ok(Json(configs))
}

/// Create or update operational webhook configuration
#[api_v2_operation(
    summary = "Upsert operational webhook configuration",
    description = "Create or update an operational webhook configuration",
    operation_id = "operational_webhooks.upsert",
    consumes = "application/json",
    produces = "application/json",
    tags("OperationalWebhooks")
)]
pub async fn upsert_config(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    organization_id: Path<Uuid>,
    input: Json<OperationalWebhookConfigInput>,
) -> Result<Json<OperationalWebhookConfig>, Hook0Problem> {
    let organization_id = organization_id.into_inner();
    
    // Authorize access to organization
    authorize_for_organization(&biscuit, &organization_id, Action::OrganizationEdit)?;
    
    // Validate event type
    let valid_event_types = vec![
        "endpoint.disabled",
        "endpoint.warning",
        "message.attempt.exhausted",
        "endpoint.recovered",
    ];
    
    if !valid_event_types.contains(&input.event_type.as_str()) {
        return Err(Hook0Problem::EventInvalidJsonPayload(format!(
            "Invalid event type. Must be one of: {:?}",
            valid_event_types
        )));
    }
    
    // Upsert the configuration
    let config = query!(
        r#"
        INSERT INTO webhook.operational_webhook_config (
            organization__id,
            event_type,
            target_url,
            headers,
            is_enabled
        ) VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (organization__id, event_type)
        DO UPDATE SET
            target_url = EXCLUDED.target_url,
            headers = EXCLUDED.headers,
            is_enabled = EXCLUDED.is_enabled,
            updated_at = NOW()
        RETURNING
            config__id as config_id,
            organization__id as organization_id,
            event_type,
            target_url,
            headers as "headers: sqlx::types::Json<HashMap<String, String>>",
            is_enabled,
            created_at,
            updated_at
        "#,
        organization_id,
        input.event_type,
        input.target_url,
        sqlx::types::Json(&input.headers) as _,
        input.is_enabled
    )
    .fetch_one(db.as_ref())
    .await?;
    
    Ok(Json(OperationalWebhookConfig {
        config_id: config.config_id,
        organization_id: config.organization_id,
        event_type: config.event_type,
        target_url: config.target_url,
        headers: config.headers.unwrap_or(sqlx::types::Json(HashMap::new())).0,
        is_enabled: config.is_enabled,
        created_at: config.created_at,
        updated_at: config.updated_at,
    }))
}

/// Delete operational webhook configuration
#[api_v2_operation(
    summary = "Delete operational webhook configuration",
    description = "Delete an operational webhook configuration",
    operation_id = "operational_webhooks.delete",
    tags("OperationalWebhooks")
)]
pub async fn delete_config(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    organization_id: Path<Uuid>,
    event_type: Path<String>,
) -> Result<NoContent, Hook0Problem> {
    let organization_id = organization_id.into_inner();
    let event_type = event_type.into_inner();
    
    // Authorize access to organization
    authorize_for_organization(&biscuit, &organization_id, Action::OrganizationEdit)?;
    
    let deleted = query!(
        r#"
        DELETE FROM webhook.operational_webhook_config
        WHERE organization__id = $1 AND event_type = $2
        "#,
        organization_id,
        event_type
    )
    .execute(db.as_ref())
    .await?;
    
    if deleted.rows_affected() == 0 {
        return Err(Hook0Problem::NotFound);
    }
    
    Ok(NoContent)
}