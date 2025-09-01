use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use log::error;
use paperclip::actix::web::{Data, Json, Path};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::authentication::config::{AuthenticationConfigRequest, AuthenticationType};
use crate::iam::{Action, AuthorizedToken, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
#[allow(dead_code)]
pub struct AuthenticationConfigResponse {
    pub authentication_config_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub auth_type: AuthenticationType,
    pub config: serde_json::Value,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
#[allow(dead_code)]
pub struct ApplicationPath {
    pub application_id: Uuid,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
#[allow(dead_code)]
pub struct SubscriptionPath {
    pub subscription_id: Uuid,
}

/// Configure authentication for an application
#[api_v2_operation(
    summary = "Configure default authentication for an application",
    description = "Sets the default authentication configuration that will be used for all subscriptions of this application unless overridden",
    operation_id = "authentication.configure_application",
    consumes = "application/json",
    produces = "application/json",
    tags("Authentication Management")
)]
pub async fn configure_application_authentication(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    path: Path<ApplicationPath>,
    body: Json<AuthenticationConfigRequest>,
) -> Result<CreatedJson<AuthenticationConfigResponse>, Hook0Problem> {
    // Check authorization
    let _auth_token = authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationEdit {
            application_id: &path.application_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .map_err(|_| Hook0Problem::Forbidden)?;

    // Validate the request
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    // Get user ID from authorized token
    let user_id = match _auth_token {
        AuthorizedToken::User(user) => Some(user.user_id),
        _ => None,
    };

    // Save configuration
    let auth_service = state
        .auth_service
        .as_ref()
        .ok_or(Hook0Problem::InternalServerError)?;

    let body_data = body.into_inner();
    let auth_type = body_data.auth_type.clone();
    let config = body_data.config.clone();

    let config_id = auth_service
        .save_authentication_config(
            path.application_id,
            None,
            body_data,
            user_id.ok_or(Hook0Problem::Forbidden)?,
        )
        .await
        .map_err(|e| {
            error!("Failed to save authentication config: {}", e);
            Hook0Problem::InternalServerError
        })?;

    Ok(CreatedJson(AuthenticationConfigResponse {
        authentication_config_id: config_id,
        application_id: path.application_id,
        subscription_id: None,
        auth_type,
        config,
        is_active: true,
    }))
}

/// Configure authentication for a subscription
#[api_v2_operation(
    summary = "Configure authentication override for a subscription",
    description = "Sets a specific authentication configuration for this subscription, overriding the application default",
    operation_id = "authentication.configure_subscription",
    consumes = "application/json",
    produces = "application/json",
    tags("Authentication Management")
)]
pub async fn configure_subscription_authentication(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    path: Path<SubscriptionPath>,
    body: Json<AuthenticationConfigRequest>,
) -> Result<CreatedJson<AuthenticationConfigResponse>, Hook0Problem> {
    // First, we need to get the application_id for this subscription
    let subscription = sqlx::query_as::<_, (Uuid,)>(
        "SELECT application__id FROM webhook.subscription WHERE subscription__id = $1",
    )
    .bind(path.subscription_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| Hook0Problem::InternalServerError)?
    .ok_or(Hook0Problem::NotFound)?;

    // Check authorization
    let _auth_token = authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionEdit {
            application_id: &subscription.0,
            subscription_id: &path.subscription_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .map_err(|_| Hook0Problem::Forbidden)?;

    // Validate the request
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    // Get user ID from authorized token
    let user_id = match _auth_token {
        AuthorizedToken::User(user) => Some(user.user_id),
        _ => None,
    };

    // Get application ID for this subscription
    let subscription = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT application__id 
        FROM webhook.subscription 
        WHERE subscription__id = $1
        "#,
    )
    .bind(path.subscription_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| Hook0Problem::NotFound)?;

    // Save configuration
    let auth_service = state
        .auth_service
        .as_ref()
        .ok_or(Hook0Problem::InternalServerError)?;

    let body_data = body.into_inner();
    let auth_type = body_data.auth_type.clone();
    let config = body_data.config.clone();

    let config_id = auth_service
        .save_authentication_config(
            subscription.0,
            Some(path.subscription_id),
            body_data,
            user_id.ok_or(Hook0Problem::Forbidden)?,
        )
        .await
        .map_err(|e| {
            error!("Failed to save authentication config: {}", e);
            Hook0Problem::InternalServerError
        })?;

    Ok(CreatedJson(AuthenticationConfigResponse {
        authentication_config_id: config_id,
        application_id: subscription.0,
        subscription_id: Some(path.subscription_id),
        auth_type,
        config,
        is_active: true,
    }))
}

/// Delete authentication configuration for an application
#[api_v2_operation(
    summary = "Remove authentication configuration for an application",
    description = "Removes the default authentication configuration for an application",
    operation_id = "authentication.delete_application",
    tags("Authentication Management")
)]
pub async fn delete_application_authentication(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    path: Path<ApplicationPath>,
) -> Result<NoContent, Hook0Problem> {
    // Check authorization
    let _auth_token = authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationEdit {
            application_id: &path.application_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .map_err(|_| Hook0Problem::Forbidden)?;

    // Delete configuration
    let auth_service = state
        .auth_service
        .as_ref()
        .ok_or(Hook0Problem::InternalServerError)?;

    auth_service
        .delete_authentication_config(path.application_id, None)
        .await
        .map_err(|e| {
            error!("Failed to delete authentication config: {}", e);
            Hook0Problem::InternalServerError
        })?;

    Ok(NoContent)
}

/// Delete authentication configuration for a subscription
#[api_v2_operation(
    summary = "Remove authentication override for a subscription",
    description = "Removes the authentication override for a subscription, reverting to application default",
    operation_id = "authentication.delete_subscription",
    tags("Authentication Management")
)]
pub async fn delete_subscription_authentication(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    path: Path<SubscriptionPath>,
) -> Result<NoContent, Hook0Problem> {
    // First, we need to get the application_id for this subscription
    let subscription = sqlx::query_as::<_, (Uuid,)>(
        "SELECT application__id FROM webhook.subscription WHERE subscription__id = $1",
    )
    .bind(path.subscription_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| Hook0Problem::InternalServerError)?
    .ok_or(Hook0Problem::NotFound)?;

    // Check authorization
    let _auth_token = authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionEdit {
            application_id: &subscription.0,
            subscription_id: &path.subscription_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .map_err(|_| Hook0Problem::Forbidden)?;

    // Get application ID for this subscription
    let subscription = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT application__id 
        FROM webhook.subscription 
        WHERE subscription__id = $1
        "#,
    )
    .bind(path.subscription_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| Hook0Problem::NotFound)?;

    // Delete configuration
    let auth_service = state
        .auth_service
        .as_ref()
        .ok_or(Hook0Problem::InternalServerError)?;

    auth_service
        .delete_authentication_config(subscription.0, Some(path.subscription_id))
        .await
        .map_err(|e| {
            error!("Failed to delete authentication config: {}", e);
            Hook0Problem::InternalServerError
        })?;

    Ok(NoContent)
}
