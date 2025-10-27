use actix_web::body::EitherBody;
use actix_web::error::JsonPayloadError;
use actix_web::http::StatusCode;
use actix_web::mime::APPLICATION_JSON;
use actix_web::rt::spawn;
use actix_web::rt::time::timeout;
use actix_web::web::Query;
use actix_web::{HttpResponse, Responder};
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{Apiv2Schema, OperationModifier, api_v2_operation};
use paperclip::v2::models::{DefaultSchemaRaw, Either, Reference, Response};
use paperclip::v2::schema::Apiv2Schema;
use pulsar::proto::command_get_topics_of_namespace::Mode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::problems::Hook0Problem;
use crate::{ObjectStorageConfig, PulsarConfig};

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
    pulsar: Option<bool>,
    object_storage: Option<bool>,
}

impl HealthCheck {
    fn is_ok(&self) -> bool {
        self.database && self.pulsar.unwrap_or(true) && self.object_storage.unwrap_or(true)
    }
}

impl Responder for HealthCheck {
    type Body = EitherBody<String>;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let status = if self.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };
        match serde_json::to_string(&self) {
            Ok(body) => match HttpResponse::build(status)
                .content_type(APPLICATION_JSON)
                .message_body(body)
            {
                Ok(res) => res.map_into_left_body(),
                Err(err) => HttpResponse::from_error(err).map_into_right_body(),
            },

            Err(err) => {
                HttpResponse::from_error(JsonPayloadError::Serialize(err)).map_into_right_body()
            }
        }
    }
}

pub struct HealthCheckWithOa(pub HealthCheck);

impl Responder for HealthCheckWithOa {
    type Body = <HealthCheck as Responder>::Body;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        self.0.respond_to(req)
    }
}

impl paperclip::v2::schema::Apiv2Schema for HealthCheckWithOa {
    fn name() -> Option<String> {
        HealthCheck::name()
    }

    fn raw_schema() -> DefaultSchemaRaw {
        HealthCheck::raw_schema()
    }
}

impl OperationModifier for HealthCheckWithOa {
    fn update_parameter(op: &mut paperclip::v2::models::DefaultOperationRaw) {
        HealthCheck::update_parameter(op);
    }

    fn update_response(op: &mut paperclip::v2::models::DefaultOperationRaw) {
        HealthCheck::update_response(op);
        let schema_with_ref = HealthCheck::schema_with_ref();
        let response = match schema_with_ref.reference {
            Some(reference) => Either::Left(Reference { reference }),
            None => Either::Right(Response {
                description: schema_with_ref.description.to_owned(),
                schema: Some(schema_with_ref),
                headers: BTreeMap::new(),
            }),
        };
        op.responses.insert("200".to_owned(), response.clone());
        op.responses.insert("503".to_owned(), response);
    }

    fn update_definitions(
        map: &mut std::collections::BTreeMap<String, paperclip::v2::models::DefaultSchemaRaw>,
    ) {
        HealthCheck::update_definitions(map);
    }

    fn update_security(op: &mut paperclip::v2::models::DefaultOperationRaw) {
        HealthCheck::update_security(op);
    }

    fn update_security_definitions(
        map: &mut std::collections::BTreeMap<String, paperclip::v2::models::SecurityScheme>,
    ) {
        HealthCheck::update_security_definitions(map);
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
) -> Result<HealthCheckWithOa, Hook0Problem> {
    let qs_key = qs.into_inner().key.unwrap_or_else(|| "".to_owned());

    match state.health_check_key.as_deref() {
        Some(k) => {
            // Comparison is not done in constant time, but stakes are very low here
            if k.is_empty() || k == qs_key {
                let pool = state.db.clone();
                let database_task =
                    spawn(timeout(state.health_check_timeout, check_database(pool)));

                let pulsar_config = state.pulsar.clone();
                let pulsar_task = spawn(timeout(
                    state.health_check_timeout,
                    check_pulsar(pulsar_config),
                ));

                let object_storage_config = state.object_storage.clone();
                let object_storage_task = spawn(timeout(
                    state.health_check_timeout,
                    check_object_storage(object_storage_config),
                ));

                let database = matches!(database_task.await, Ok(Ok(true)));
                let pulsar = if let Ok(Ok(r)) = pulsar_task.await {
                    r
                } else if state.pulsar.is_some() {
                    Some(false)
                } else {
                    None
                };
                let object_storage = if let Ok(Ok(r)) = object_storage_task.await {
                    r
                } else if state.object_storage.is_some() {
                    Some(false)
                } else {
                    None
                };

                let health_check = HealthCheck {
                    database,
                    pulsar,
                    object_storage,
                };

                Ok(HealthCheckWithOa(health_check))
            } else {
                Err(Hook0Problem::Forbidden)
            }
        }
        _ => Err(Hook0Problem::NotFound),
    }
}

async fn check_database(db: PgPool) -> bool {
    query("SELECT 1").fetch_one(&db).await.is_ok()
}

async fn check_pulsar(pulsar: Option<Arc<PulsarConfig>>) -> Option<bool> {
    if let Some(p) = pulsar {
        Some(
            p.pulsar
                .get_topics_of_namespace(format!("{}/{}", p.tenant, p.namespace), Mode::All)
                .await
                .is_ok(),
        )
    } else {
        None
    }
}

async fn check_object_storage(object_storage: Option<ObjectStorageConfig>) -> Option<bool> {
    if let Some(os) = object_storage {
        Some(
            os.client
                .head_bucket()
                .bucket(&os.bucket)
                .send()
                .await
                .is_ok(),
        )
    } else {
        None
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

    actix_web::rt::time::sleep(duration).await;

    let profile = guard.report().build()?.pprof()?;

    let mut pprof = Vec::new();
    profile.write_to_writer(&mut pprof)?;

    Ok(pprof)
}
