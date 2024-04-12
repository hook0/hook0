use actix_web::web::Query;
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct InstanceConfig {
    biscuit_public_key: String,
    registration_disabled: bool,
    password_minimum_length: u8,
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
        biscuit_public_key: state.biscuit_private_key.public().to_bytes_hex(),
        registration_disabled: state.registration_disabled,
        password_minimum_length: state.password_minimum_length,
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
