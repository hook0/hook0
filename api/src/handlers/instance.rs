use paperclip::actix::{api_v2_operation, web::Data, web::Json, Apiv2Schema};
use serde::Serialize;

use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct InstanceConfig {
    keycloak_url: String,
    keycloak_realm: String,
    keycloak_client_id: String,
    disable_registration: bool,
    auto_db_migration: bool,
}

/// Get instance configuration
#[api_v2_operation(
    summary = "Get instance configuration",
    description = "Get an object that shows how this instance was configured.",
    operation_id = "instance.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Hook0")
)]
pub async fn get(state: Data<crate::State>) -> Result<Json<InstanceConfig>, Hook0Problem> {
    Ok(Json(InstanceConfig {
        keycloak_url: state.keycloak_url.to_string(),
        keycloak_realm: state.keycloak_realm.to_owned(),
        keycloak_client_id: state.keycloak_client_id.to_owned(),
        disable_registration: state.disable_registration,
        auto_db_migration: state.auto_db_migration,
    }))
}
