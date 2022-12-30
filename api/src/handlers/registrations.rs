use log::debug;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json},
    Apiv2Schema, CreatedJson,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use super::organizations::create_organization;
use crate::keycloak_api::KeycloakApi;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Registration {
    organization_id: Uuid,
    user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RegistrationPost {
    #[validate(non_control_character, length(min = 1, max = 50))]
    organization_name: String,
    #[validate(non_control_character, length(min = 1, max = 50))]
    first_name: String,
    #[validate(non_control_character, length(min = 1, max = 50))]
    last_name: String,
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
    #[validate(non_control_character, length(min = 10, max = 100))]
    password: String,
}

#[api_v2_operation(
    summary = "Create a new user account and a new organization",
    description = "",
    operation_id = "register",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn register(
    state: Data<crate::State>,
    body: Json<RegistrationPost>,
) -> Result<CreatedJson<Registration>, Hook0Problem> {
    if state.disable_registration {
        return Err(Hook0Problem::RegistrationDisabled);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    do_register(
        body.into_inner(),
        &state.db,
        &state.keycloak_url,
        &state.keycloak_realm,
        &state.keycloak_client_id,
        &state.keycloak_client_secret,
    )
    .await
    .map(CreatedJson)
}

async fn do_register(
    registration_req: RegistrationPost,
    db: &PgPool,
    keycloak_url: &Url,
    keycloak_realm: &str,
    keycloak_client_id: &str,
    keycloak_client_secret: &str,
) -> Result<Registration, Hook0Problem> {
    debug!("Starting registration for {}", &registration_req.email);

    let kc_api = KeycloakApi::new(
        keycloak_url,
        keycloak_realm,
        keycloak_client_id,
        keycloak_client_secret,
    )
    .await?;

    check_organization_name(&registration_req.organization_name)?;
    kc_api
        .ensure_email_does_not_exist(&registration_req.email)
        .await?;

    // Let's start a transaction so DB operations can be rollback if something fails.
    // Note: there is still a change of partial failure if something fails on the Keycloak API side.
    // TODO: implement something to detect/garbage collect these inactive users/groups.
    let mut tx = db.begin().await?;

    let user_id = kc_api
        .create_user(
            &registration_req.email,
            &registration_req.password,
            &registration_req.first_name,
            &registration_req.last_name,
        )
        .await?;

    let organization_id = create_organization(
        &mut tx,
        &kc_api,
        &registration_req.organization_name,
        &user_id,
    )
    .await?;

    tx.commit().await?;
    Ok(Registration {
        organization_id,
        user_id,
    })
}

fn check_organization_name(organization_name: &str) -> Result<(), Hook0Problem> {
    if organization_name.len() <= 1 {
        Err(Hook0Problem::OrganizationNameMissing)
    } else {
        Ok(())
    }
}
