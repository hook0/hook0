use actix_web::web::Query;
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
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
    support_email_address: String,
    cloudflare_turnstile_site_key: Option<String>,
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

    let formbricks = if let Some(environment_id) = state.formbricks_environment_id.as_ref() {
        Some(FormbricksConfig {
            api_host: state.formbricks_api_host.to_owned(),
            environment_id: environment_id.to_owned(),
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
        support_email_address: state.support_email_address.to_string(),
        cloudflare_turnstile_site_key: state.cloudflare_turnstile_site_key.to_owned(),
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

#[cfg(feature = "profiling")]
#[api_v2_operation(skip)]
pub async fn pprof_heap(
    state: Data<crate::State>,
    qs: Query<Key>,
) -> Result<actix_web::HttpResponse, Hook0Problem> {
    let qs_key = qs.into_inner().key.unwrap_or_else(|| "".to_owned());

    match state.health_check_key.as_deref() {
        Some(k) => {
            // Comparison is not done in constant time, but stakes are very low here
            if k.is_empty() || k == qs_key {
                let mut prof_ctl = jemalloc_pprof::PROF_CTL
                    .as_ref()
                    .ok_or(Hook0Problem::InternalServerError)?
                    .lock()
                    .await;
                if prof_ctl.activated() {
                    let pprof = prof_ctl.dump_pprof().map_err(|err| {
                        log::error!("{err}");
                        Hook0Problem::InternalServerError
                    })?;
                    Ok(actix_web::HttpResponse::Ok()
                        .content_type("application/gzip")
                        .append_header((
                            "Content-Disposition",
                            "attachment; filename=\"hook0-api-heap.pb.gz\"",
                        ))
                        .body(pprof))
                } else {
                    Err(Hook0Problem::NotFound)
                }
            } else {
                Err(Hook0Problem::Forbidden)
            }
        }
        _ => Err(Hook0Problem::NotFound),
    }
}

#[cfg(feature = "profiling")]
#[api_v2_operation(skip)]
pub async fn pprof_cpu(
    state: Data<crate::State>,
    qs: Query<Key>,
) -> Result<actix_web::HttpResponse, Hook0Problem> {
    let qs_key = qs.into_inner().key.unwrap_or_else(|| "".to_owned());

    match state.health_check_key.as_deref() {
        Some(k) => {
            // Comparison is not done in constant time, but stakes are very low here
            if k.is_empty() || k == qs_key {
                generate_profile(std::time::Duration::from_secs(30))
                    .await
                    .map(|pprof| {
                        actix_web::HttpResponse::Ok()
                            .content_type("application/octet-stream")
                            .append_header((
                                "Content-Disposition",
                                "attachment; filename=\"hook0-api-cpu.pb\"",
                            ))
                            .body(pprof)
                    })
                    .map_err(|e| {
                        log::error!("{e}");
                        Hook0Problem::InternalServerError
                    })
            } else {
                Err(Hook0Problem::Forbidden)
            }
        }
        _ => Err(Hook0Problem::NotFound),
    }
}

#[cfg(feature = "profiling")]
async fn generate_profile(duration: std::time::Duration) -> anyhow::Result<Vec<u8>> {
    use pprof::protos::Message;

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(200)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()?;

    actix::clock::sleep(duration).await;

    let profile = guard.report().build()?.pprof()?;

    let mut pprof = Vec::new();
    profile.write_to_writer(&mut pprof)?;

    Ok(pprof)
}
