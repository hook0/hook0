use actix_web::web::Query;
use paperclip::actix::{api_v2_operation, web::Data, web::Json, Apiv2Schema};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct InstanceConfig {
    keycloak_url: String,
    keycloak_realm: String,
    keycloak_front_client_id: String,
    disable_registration: bool,
    auto_db_migration: bool,
}

/// Get instance configuration
#[api_v2_operation(
    summary = "Get instance configuration",
    description = "Get an object that shows how this instance is configured.",
    operation_id = "instance.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Hook0")
)]
pub async fn get(state: Data<crate::State>) -> Result<Json<InstanceConfig>, Hook0Problem> {
    Ok(Json(InstanceConfig {
        keycloak_url: state.keycloak_url.to_string(),
        keycloak_realm: state.keycloak_realm.to_owned(),
        keycloak_front_client_id: state.keycloak_front_client_id.to_owned(),
        disable_registration: state.disable_registration,
        auto_db_migration: state.auto_db_migration,
    }))
}

#[derive(Debug, Clone, Copy, Default, Serialize, Apiv2Schema)]
pub struct HealthCheck {
    database: bool,
}

impl Display for HealthCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.database {
            write!(f, "Some components are down: database")
        } else {
            write!(f, "All components seem OK")
        }
    }
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct Key {
    key: Option<String>,
}

/// Check instance health
#[api_v2_operation(
    summary = "Check instance health",
    description = "Get an object that shows if this instance is up.",
    operation_id = "instance.health",
    consumes = "application/json",
    produces = "application/json",
    tags("Hook0")
)]
pub async fn health(
    state: Data<crate::State>,
    qs: Query<Key>,
) -> Result<Json<HealthCheck>, Hook0Problem> {
    let qs_key = qs.into_inner().key.unwrap_or_else(|| "".to_owned());

    match state.health_check_key.as_deref() {
        Some(k) => {
            // Comparison is not done in constant time, but stakes are very low here
            if k.is_empty() || k == qs_key {
                let database = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();
                let health_check = HealthCheck { database };

                if database {
                    Ok(Json(health_check))
                } else {
                    Err(Hook0Problem::ServiceUnavailable(health_check))
                }
            } else {
                Err(Hook0Problem::Forbidden)
            }
        }
        _ => Err(Hook0Problem::NotFound),
    }
}
