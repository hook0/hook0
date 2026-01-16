use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use lettre::Address;
use lettre::message::Mailbox;
use log::error;
use paperclip::actix::web::Data;
use paperclip::actix::{Apiv2Schema, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query;
use std::str::FromStr;

use crate::iam::{Action, authorize_only_user};
use crate::mailer::Mail;
use crate::openapi::OaBiscuitUserAccess;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct AccountDeletionStatus {
    deletion_requested: bool,
}

#[api_v2_operation(
    summary = "Request account deletion",
    description = "Request the deletion of the current user's account. The account will be deleted after 30 days. This action can be cancelled during this period by calling the cancel-deletion endpoint.",
    operation_id = "me.request_deletion",
    consumes = "application/json",
    produces = "application/json",
    tags("Account Management")
)]
pub async fn request_deletion(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
) -> Result<NoContent, Hook0Problem> {
    let token = authorize_only_user(
        &biscuit,
        None,
        Action::AccountDelete,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .map_err(|_| Hook0Problem::Forbidden)?;

    let user = query!(
        "
            SELECT user__id AS user_id, email, first_name, last_name, deletion_requested_at
            FROM iam.user
            WHERE user__id = $1
                AND deleted_at IS NULL
        ",
        &token.user_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    // Check if deletion was already requested
    if user.deletion_requested_at.is_some() {
        return Err(Hook0Problem::AccountDeletionAlreadyRequested);
    }

    // Mark the account for deletion
    query!(
        "
            UPDATE iam.user
            SET deletion_requested_at = statement_timestamp()
            WHERE user__id = $1
                AND deleted_at IS NULL
                AND deletion_requested_at IS NULL
        ",
        &token.user_id,
    )
    .execute(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // Send confirmation email
    let address = Address::from_str(&user.email).map_err(|e| {
        error!("Error trying to parse email address: {e}");
        Hook0Problem::InternalServerError
    })?;
    let recipient = Mailbox::new(
        Some(format!("{} {}", user.first_name, user.last_name)),
        address,
    );

    state
        .mailer
        .send_mail(Mail::AccountDeletionRequested, recipient)
        .await
        .map_err(|e| {
            error!("Error trying to send account deletion confirmation email: {e}");
            Hook0Problem::InternalServerError
        })?;

    Ok(NoContent)
}

#[api_v2_operation(
    summary = "Cancel account deletion",
    description = "Cancel a pending account deletion request. This action is only available during the 30-day grace period after requesting deletion.",
    operation_id = "me.cancel_deletion",
    consumes = "application/json",
    produces = "application/json",
    tags("Account Management")
)]
pub async fn cancel_deletion(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
) -> Result<NoContent, Hook0Problem> {
    let token = authorize_only_user(
        &biscuit,
        None,
        Action::AccountCancelDeletion,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .map_err(|_| Hook0Problem::Forbidden)?;

    let user = query!(
        "
            SELECT user__id AS user_id, email, first_name, last_name, deletion_requested_at
            FROM iam.user
            WHERE user__id = $1
                AND deleted_at IS NULL
        ",
        &token.user_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    // Check if deletion was actually requested
    if user.deletion_requested_at.is_none() {
        return Err(Hook0Problem::AccountDeletionNotRequested);
    }

    // Cancel the deletion request
    query!(
        "
            UPDATE iam.user
            SET deletion_requested_at = NULL
            WHERE user__id = $1
                AND deleted_at IS NULL
                AND deletion_requested_at IS NOT NULL
        ",
        &token.user_id,
    )
    .execute(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // Send confirmation email
    let address = Address::from_str(&user.email).map_err(|e| {
        error!("Error trying to parse email address: {e}");
        Hook0Problem::InternalServerError
    })?;
    let recipient = Mailbox::new(
        Some(format!("{} {}", user.first_name, user.last_name)),
        address,
    );

    state
        .mailer
        .send_mail(Mail::AccountDeletionCancelled, recipient)
        .await
        .map_err(|e| {
            error!("Error trying to send account deletion cancellation email: {e}");
            Hook0Problem::InternalServerError
        })?;

    Ok(NoContent)
}

#[api_v2_operation(
    summary = "Get account deletion status",
    description = "Check if an account deletion request is pending for the current user.",
    operation_id = "me.get_deletion_status",
    produces = "application/json",
    tags("Account Management")
)]
pub async fn get_deletion_status(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
) -> Result<paperclip::actix::web::Json<AccountDeletionStatus>, Hook0Problem> {
    let token = authorize_only_user(
        &biscuit,
        None,
        Action::AccountDelete,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .map_err(|_| Hook0Problem::Forbidden)?;

    let user = query!(
        "
            SELECT deletion_requested_at
            FROM iam.user
            WHERE user__id = $1
                AND deleted_at IS NULL
        ",
        &token.user_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    Ok(paperclip::actix::web::Json(AccountDeletionStatus {
        deletion_requested: user.deletion_requested_at.is_some(),
    }))
}
