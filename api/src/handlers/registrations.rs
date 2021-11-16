use log::{debug, error};
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json},
    Apiv2Schema, CreatedJson,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::keycloak_api::KeycloakApi;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Registration {
    organization_id: Uuid,
    user_id: Uuid,
    temporary_password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct RegistrationPost {
    organization_name: String,
    first_name: String,
    last_name: String,
    email: String,
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
    // TODO: implement something to detect/garbage collect these inactive groups.
    let mut tx = db.begin().await?;

    let organization_id =
        create_organization_in_db(&mut tx, &registration_req.organization_name).await?;

    let editor_group_id = kc_api.create_organization(&organization_id).await?;
    let (user_id, temporary_password) = kc_api
        .create_user(
            &registration_req.email,
            &registration_req.first_name,
            &registration_req.last_name,
            Some(&editor_group_id),
        )
        .await?;

    tx.commit().await?;
    Ok(Registration {
        organization_id,
        user_id,
        temporary_password,
    })
}

fn check_organization_name(organization_name: &str) -> Result<(), Hook0Problem> {
    if organization_name.len() <= 1 {
        Err(Hook0Problem::OrganizationNameMissing)
    } else {
        Ok(())
    }
}

async fn create_organization_in_db(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
) -> Result<Uuid, Hook0Problem> {
    let organization_id = Uuid::new_v4();
    query!(
        "
            INSERT INTO event.organization (organization__id, name)
            VALUES ($1, $2)
        ",
        &organization_id,
        name
    )
    .execute(tx)
    .await
    .map_err(|e| {
        error!("Error while creating organization in DB: {}", &e);
        Hook0Problem::InternalServerError
    })?;

    Ok(organization_id)
}
