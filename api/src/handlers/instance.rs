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
    application_secret_compatibility: bool,
    quota_enforcement: bool,
    matomo: Option<MatomoConfig>,
    formbricks: Option<FormbricksConfig>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct MatomoConfig {
    url: String,
    site_id: u16,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct FormbricksConfig {
    api_host: String,
    environment_id: String,
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
    let matomo =
        if let (Some(url), Some(site_id)) = (state.matomo_url.as_ref(), state.matomo_site_id) {
            Some(MatomoConfig {
                url: url.to_string().trim_end_matches('/').to_owned(),
                site_id,
            })
        } else {
            None
        };

    let formbricks = if let (Some(api_host), Some(environment_id)) = (
        state.formbricks_api_host.as_ref(),
        state.formbricks_environment_id.as_ref(),
    ) {
        Some(FormbricksConfig {
            api_host: api_host.to_string().trim_end_matches('/').to_owned(),
            environment_id: environment_id.to_string(),
        })
    } else {
        None
    };

    Ok(Json(InstanceConfig {
        biscuit_public_key: state.biscuit_private_key.public().to_bytes_hex(),
        registration_disabled: state.registration_disabled,
        password_minimum_length: state.password_minimum_length,
        auto_db_migration: state.auto_db_migration,
        application_secret_compatibility: state.application_secret_compatibility,
        quota_enforcement: state.enable_quota_enforcement,
        matomo,
        formbricks,
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
