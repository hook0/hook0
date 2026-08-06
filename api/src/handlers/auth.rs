use actix_web::rt::task::spawn_blocking;
use actix_web::web::ReqData;
use argon2::password_hash::PasswordHashString;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use biscuit_auth::{Biscuit, PrivateKey};
use chrono::{DateTime, Utc};
use lettre::Address;
use lettre::message::Mailbox;
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Postgres, query, query_as, query_scalar};
use std::str::FromStr;
use tracing::{debug, error, warn};
use uuid::Uuid;
use validator::Validate;

use crate::iam::{
    Action, authorize_email_verification, authorize_only_user, authorize_refresh_token,
    authorize_reset_password, create_refresh_token, create_reset_password_token,
    create_user_access_token,
};
use crate::mailer::Mail;
use crate::openapi::{OaBiscuitRefresh, OaBiscuitUserAccess};
use crate::password;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct LoginPost {
    #[validate(non_control_character, length(min = 1, max = 100))]
    email: String,
    // Bounded, but with no policy of its own: logging in must accept whatever
    // the account's password happens to be, including one set before the policy
    // existed.
    #[validate(non_control_character, length(min = 1, max = 100))]
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct LoginResponse {
    access_token: String,
    access_token_expiration: DateTime<Utc>,
    refresh_token: String,
    refresh_token_expiration: DateTime<Utc>,
    user_id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct EmailVerificationPost {
    #[validate(non_control_character, length(min = 1, max = 1000))]
    token: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct BeginResetPasswordPost {
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ResetPasswordPost {
    #[validate(non_control_character, length(min = 1, max = 1000))]
    token: String,
    // Length is deliberately not validated here: the policy owns both bounds
    // (`password::Checked::new`), so the user is told the instance's real
    // minimum instead of a number hardcoded next to the field. It also keeps
    // the rejected password out of the response body, which the `length`
    // validator echoes back as an error parameter.
    #[validate(non_control_character)]
    new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ChangePasswordPost {
    // See `ResetPasswordPost::new_password`: the policy owns the bounds.
    #[validate(non_control_character)]
    new_password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UserLookup {
    user_id: Uuid,
    password_hash: String,
    email: String,
    first_name: String,
    last_name: String,
    email_verified_at: Option<DateTime<Utc>>,
}

#[api_v2_operation(
    summary = "Login",
    description = "Get an access token using a user's credentials.",
    operation_id = "auth.login",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn login(
    state: Data<crate::State>,
    body: Json<LoginPost>,
) -> Result<CreatedJson<LoginResponse>, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let user_lookup = query_as!(
        UserLookup,
        "
            SELECT user__id AS user_id, password AS password_hash, email, first_name, last_name, email_verified_at
            FROM iam.user
            WHERE email = $1
        ",
        &body.email,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(user) = user_lookup {
        let password = body.password.clone();
        let password_hash = user.password_hash.clone();
        let user_id = user.user_id;

        let password_valid = spawn_blocking(move || -> Result<bool, Hook0Problem> {
            let parsed_hash = PasswordHash::new(&password_hash).map_err(|e| {
                error!(
                    "Password hash of user {} is not in the right format: {e}",
                    &user_id
                );
                Hook0Problem::InternalServerError
            })?;

            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|e| {
            error!("Failed to run password verification task: {e}");
            Hook0Problem::InternalServerError
        })??;

        if password_valid {
            if user.email_verified_at.is_some() {
                do_login(&state.db, &state.biscuit_private_key, user, None).await
            } else {
                Err(Hook0Problem::AuthEmailNotVerified)
            }
        } else {
            Err(Hook0Problem::AuthFailedLogin)
        }
    } else {
        #[cfg(feature = "migrate-users-from-keycloak")]
        {
            if state.enable_keycloak_migration {
                let user =
                    import_user_from_keycloak(&state, &body.email.to_lowercase(), &body.password)
                        .await?;
                do_login(&state.db, &state.biscuit_private_key, user, None).await
            } else {
                Err(Hook0Problem::AuthFailedLogin)
            }
        }

        #[cfg(not(feature = "migrate-users-from-keycloak"))]
        {
            Err(Hook0Problem::AuthFailedLogin)
        }
    }
}

#[cfg(feature = "migrate-users-from-keycloak")]
async fn import_user_from_keycloak(
    state: &crate::State,
    email: &str,
    password: &str,
) -> Result<UserLookup, Hook0Problem> {
    use tracing::trace;

    reqwest::Client::new()
        .post(format!(
            "{}/realms/{}/protocol/openid-connect/token",
            state.keycloak_url, state.keycloak_realm
        ))
        .form(&[
            ("grant_type", "password"),
            ("client_id", &state.keycloak_client_id),
            ("client_secret", &state.keycloak_client_secret),
            ("username", email),
            ("password", password),
        ])
        .send()
        .await
        .map_err(|e| {
            trace!("Error trying to login on Keycloak using 'Direct access grant' mode: {e}");
            Hook0Problem::AuthFailedLogin
        })?
        .error_for_status()
        .map_err(|e| {
            trace!("Error trying to login on Keycloak using 'Direct access grant' mode: {e}");
            Hook0Problem::AuthFailedLogin
        })?;

    let keycloak_api = crate::keycloak_api::KeycloakApi::new(
        &state.keycloak_url,
        &state.keycloak_realm,
        &state.keycloak_client_id,
        &state.keycloak_client_secret,
    )
    .await?;

    let kc_user = keycloak_api
        .get_user_by_email(email)
        .await?
        .ok_or_else(|| {
            error!("Error trying to get user from Keycloak API");
            Hook0Problem::InternalServerError
        })?;

    if kc_user.enabled && kc_user.email_verified {
        let groups = keycloak_api.get_user_groups(&kc_user.id).await?;
        let roles = crate::iam::kc_group_paths_to_roles(
            &groups.into_iter().map(|g| g.path).collect::<Vec<_>>(),
        );

        let password_hash =
            password::hash(password::Checked::already_established(password)).await?;

        let mut tx = state.db.begin().await?;

        query!(
            "
                INSERT INTO iam.user (user__id, email, password, first_name, last_name, email_verified_at)
                VALUES ($1, $2, $3, $4, $5, statement_timestamp())
            ",
            &kc_user.id,
            &kc_user.email,
            password_hash.as_str(),
            &kc_user.first_name,
            &kc_user.last_name,
        )
        .execute(&mut *tx)
        .await?;

        for (organization_id, role) in &roles {
            // The ON CONFLICT DO NOTHING part is because an organization may have been deleted with the user still in the corresponding Keycloak group
            query!(
                "
                    INSERT INTO iam.user__organization (user__id, organization__id, role)
                    VALUES ($1, $2, $3)
                    ON CONFLICT DO NOTHING
                ",
                &kc_user.id,
                organization_id,
                role.as_ref(),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        debug!(
            "User {} was successfuly imported from Keycloak",
            &kc_user.id
        );

        Ok(UserLookup {
            user_id: kc_user.id,
            password_hash: password_hash.to_string(),
            email: kc_user.email,
            first_name: kc_user.first_name,
            last_name: kc_user.last_name,
            email_verified_at: Some(Utc::now()),
        })
    } else {
        trace!("Error trying to import a non-verified user from Keycloak API");
        Err(Hook0Problem::AuthFailedLogin)
    }
}

async fn do_login<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    biscuit_private_key: &PrivateKey,
    user: UserLookup,
    session_id: Option<Uuid>,
) -> Result<CreatedJson<LoginResponse>, Hook0Problem> {
    let mut db = db.acquire().await?;

    #[derive(Debug, Clone)]
    struct OrganizationRole {
        organization_id: Uuid,
        role: String,
    }
    let roles = query_as!(
        OrganizationRole,
        "
            SELECT organization__id AS organization_id, role
            FROM iam.user__organization
            WHERE user__id = $1
        ",
        &user.user_id,
    )
    .fetch_all(&mut *db)
    .await
    .map_err(Hook0Problem::from)?
    .into_iter()
    .map(|or| (or.organization_id, or.role))
    .collect::<Vec<_>>();

    let session_id = session_id.unwrap_or_else(Uuid::new_v4);
    let access_token_id = Uuid::new_v4();
    let (access_token, access_token_expiration) = create_user_access_token(
        biscuit_private_key,
        access_token_id,
        session_id,
        user.user_id,
        &user.email,
        &user.first_name,
        &user.last_name,
        roles,
    )
    .and_then(|rt| {
        if let Some(expired_at) = rt.expired_at {
            Ok((rt, expired_at))
        } else {
            Err(biscuit_auth::error::Token::InternalError)
        }
    })
    .map_err(|e| {
        error!("Could not create a Biscuit (user access token): {e}");
        Hook0Problem::InternalServerError
    })?;

    let refresh_token_id = Uuid::new_v4();
    let (refresh_token, refresh_token_expiration) = create_refresh_token(
        biscuit_private_key,
        refresh_token_id,
        session_id,
        user.user_id,
    )
    .and_then(|rt| {
        if let Some(expired_at) = rt.expired_at {
            Ok((rt, expired_at))
        } else {
            Err(biscuit_auth::error::Token::InternalError)
        }
    })
    .map_err(|e| {
        error!("Could not create a Biscuit (refresh token): {e}");
        Hook0Problem::InternalServerError
    })?;

    query!(
        "
            INSERT INTO iam.token (token__id, type, revocation_id, expired_at, user__id, session_id)
            VALUES
                ($1, 'user_access', $2, $3, $4, $5),
                ($6, 'refresh', $7, $8, $4, $5)
        ",
        &access_token_id,
        &access_token.revocation_id,
        access_token_expiration,
        &user.user_id,
        &session_id,
        &refresh_token_id,
        &refresh_token.revocation_id,
        refresh_token_expiration,
    )
    .execute(&mut *db)
    .await?;

    query!(
        "
            UPDATE iam.user
            SET last_login = statement_timestamp()
            WHERE user__id = $1
        ",
        &user.user_id,
    )
    .execute(&mut *db)
    .await?;

    Ok(CreatedJson(LoginResponse {
        access_token: access_token.serialized_biscuit,
        access_token_expiration,
        refresh_token: refresh_token.serialized_biscuit,
        refresh_token_expiration,
        user_id: user.user_id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
    }))
}

#[api_v2_operation(
    summary = "Refresh access token",
    description = "Get a new access token in exchange of a refresh token.",
    operation_id = "auth.refresh",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn refresh(
    state: Data<crate::State>,
    _: OaBiscuitRefresh,
    biscuit: ReqData<Biscuit>,
) -> Result<CreatedJson<LoginResponse>, Hook0Problem> {
    if let Ok(token) = authorize_refresh_token(&biscuit) {
        let mut tx = state.db.begin().await?;

        query!(
            "
                UPDATE iam.token
                SET expired_at = statement_timestamp()
                WHERE token__id = $1
                    AND type = 'refresh'
                    AND expired_at > statement_timestamp()
            ",
            &token.token_id,
        )
        .execute(&mut *tx)
        .await?;

        let user = query_as!(
            UserLookup,
            "
                SELECT user__id AS user_id, password AS password_hash, email, first_name, last_name, email_verified_at
                FROM iam.user
                WHERE user__id = $1
            ",
            &token.user_id,
        )
        .fetch_one(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        let res = do_login(
            &mut tx,
            &state.biscuit_private_key,
            user,
            Some(token.session_id),
        )
        .await?;
        tx.commit().await?;
        Ok(res)
    } else {
        Err(Hook0Problem::AuthFailedRefresh)
    }
}

#[api_v2_operation(
    summary = "Logout",
    description = "Revoke all tokens associated to the current session.",
    operation_id = "auth.logout",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn logout(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
) -> Result<NoContent, Hook0Problem> {
    let token = authorize_only_user(
        &biscuit,
        None,
        Action::AuthLogout,
        state.max_authorization_time,
        state.debug_authorizer,
    )?;

    query!(
        "
            UPDATE iam.token
            SET expired_at = statement_timestamp()
            WHERE user__id = $1
                AND expired_at > statement_timestamp()
                AND session_id = $2
                AND type IN ('user_access', 'refresh')
        ",
        &token.user_id,
        &token.session_id,
    )
    .execute(&state.db)
    .await?;

    Ok(NoContent)
}

#[api_v2_operation(
    summary = "Email verification",
    description = "Verify the email of a user.",
    operation_id = "auth.verify_email",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn verify_email(
    state: Data<crate::State>,
    body: Json<EmailVerificationPost>,
) -> Result<NoContent, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let body = body.into_inner();

    let token =
        Biscuit::from_base64(body.token, state.biscuit_private_key.public()).map_err(|e| {
            debug!("{e}");
            Hook0Problem::AuthEmailExpired
        })?;

    if let Ok(token) = authorize_email_verification(&token) {
        struct VerifiedUser {
            email: String,
            first_name: String,
            last_name: String,
        }
        let verified_user = query_as!(
            VerifiedUser,
            "
                UPDATE iam.user
                SET email_verified_at = statement_timestamp()
                WHERE user__id = $1 AND email_verified_at IS NULL
                RETURNING email, first_name, last_name
            ",
            &token.user_id,
        )
        .fetch_optional(&state.db)
        .await?;

        if let Some(user) = verified_user {
            // Fire the Google Ads "signup" conversion. Done after the email is
            // confirmed so unverified / bot signups never reach Google Ads.
            // The attribution row is KEPT (gclid retained) so the later
            // first-event / first-webhook-delivered conversions can reuse the
            // gclid. The `signup_uploaded_at IS NULL` guard makes this fire at
            // most once per user. The row is cleaned up by the 30-day job, or its
            // gclid is nulled once every enabled conversion is uploaded (data
            // minimisation).
            if let Some(client) = state.google_ads.as_ref().cloned() {
                let attribution = query!(
                    "
                        UPDATE iam.signup_attribution
                        SET signup_uploaded_at = statement_timestamp()
                        WHERE user__id = $1
                          AND signup_uploaded_at IS NULL
                          AND gclid IS NOT NULL
                        RETURNING gclid
                    ",
                    &token.user_id,
                )
                .fetch_optional(&state.db)
                .await?;

                if let Some(row) = attribution
                    && let Some(gclid) = row.gclid
                {
                    let first_event_tracking_enabled = client.has_first_event_conversion();
                    let first_webhook_delivered_tracking_enabled =
                        client.has_first_webhook_delivered_conversion();
                    crate::google_ads::spawn_upload(
                        client,
                        gclid,
                        crate::google_ads::ConversionKind::Signup,
                    );
                    // The first event (and even the first webhook delivery) can
                    // happen before email verification, so gate on every enabled
                    // conversion before minimising the gclid.
                    crate::google_ads::clear_gclid_if_fully_uploaded_by_user(
                        &state.db,
                        &token.user_id,
                        first_event_tracking_enabled,
                        first_webhook_delivered_tracking_enabled,
                    )
                    .await;
                }
            }

            // Send the welcome email post-verification (non-blocking: the
            // verification itself MUST succeed regardless of SMTP availability).
            // The UPDATE ... WHERE email_verified_at IS NULL filter above
            // guarantees the welcome can only fire once per user (idempotent
            // even on double-click on the verify link).
            match Address::from_str(&user.email) {
                Ok(address) => {
                    let recipient = Mailbox::new(
                        Some(format!("{} {}", user.first_name, user.last_name)),
                        address,
                    );
                    let welcome_mail = crate::mailer::Mail::Welcome {
                        recipient_first_name: Some(user.first_name.clone()),
                    };
                    if let Err(e) = state.mailer.send_mail(welcome_mail, recipient).await {
                        warn!("Could not send welcome email to {}: {e}", user.email);
                    }
                }
                Err(e) => warn!("Could not parse verified user email for welcome message: {e}"),
            }

            Ok(NoContent)
        } else {
            debug!(
                "User {} tried to verify its email whereas it is already verified",
                &token.user_id
            );
            Err(Hook0Problem::AuthEmailExpired)
        }
    } else {
        Err(Hook0Problem::AuthEmailExpired)
    }
}

#[api_v2_operation(
    summary = "Begin reset password",
    description = "Send an email with a link to reset the password of a user.",
    operation_id = "auth.begin_reset_password",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn begin_reset_password(
    state: Data<crate::State>,
    body: Json<BeginResetPasswordPost>,
) -> Result<NoContent, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let body = body.into_inner();

    struct UserLookup {
        user_id: Uuid,
        email: String,
        first_name: String,
        last_name: String,
    }
    let user_lookup = query_as!(
        UserLookup,
        "
            SELECT user__id AS user_id, email, first_name, last_name
            FROM iam.user
            WHERE email = $1
        ",
        &body.email,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(user) = user_lookup {
        let biscuit_token = create_reset_password_token(&state.biscuit_private_key, user.user_id)
            .map_err(|e| {
            error!("Error trying to create reset password token: {e}");
            Hook0Problem::InternalServerError
        })?;

        let address = Address::from_str(&user.email).map_err(|e| {
            error!("Error trying to parse email address: {e}");
            Hook0Problem::InternalServerError
        })?;
        let recipient = Mailbox::new(
            Some(format!("{} {}", user.first_name, user.last_name)),
            address,
        );
        let url = {
            let mut url = state
                .app_url
                .join("reset-password")
                .map_err(|_| Hook0Problem::InternalServerError)?;
            url.query_pairs_mut()
                .append_pair("token", &biscuit_token.serialized_biscuit);
            url
        };

        match state
            .mailer
            .send_mail(
                Mail::ResetPassword {
                    recipient_first_name: Some(user.first_name.clone()),
                    url,
                },
                recipient,
            )
            .await
        {
            Ok(_) => Ok(NoContent),
            Err(e) => {
                error!("Error trying to send email: {e}");
                Err(Hook0Problem::InternalServerError)
            }
        }
    } else {
        Err(Hook0Problem::AuthEmailExpired)
    }
}

#[api_v2_operation(
    summary = "Reset password",
    description = "Reset the password of a user.",
    operation_id = "auth.reset_password",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn reset_password(
    state: Data<crate::State>,
    body: Json<ResetPasswordPost>,
) -> Result<NoContent, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let body = body.into_inner();

    let token =
        Biscuit::from_base64(body.token, state.biscuit_private_key.public()).map_err(|e| {
            debug!("{e}");
            Hook0Problem::AuthEmailExpired
        })?;

    if let Ok(token) = authorize_reset_password(&token) {
        let uid = query_scalar!(
            "
                SELECT user__id
                FROM iam.user
                WHERE user__id = $1
            ",
            &token.user_id,
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        if let Some(user_id) = uid {
            let mut tx = state.db.begin().await?;

            let password_hash = check_and_hash_new_password(
                &mut *tx,
                state.password_minimum_length,
                &body.new_password,
                user_id,
            )
            .await?;
            store_new_password(&mut *tx, password_hash.as_str(), user_id).await?;

            query!(
                "
                    UPDATE iam.user
                    SET email_verified_at = statement_timestamp()
                    WHERE user__id = $1
                        AND email_verified_at IS NULL
                ",
                &user_id,
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok(NoContent)
        } else {
            Err(Hook0Problem::AuthEmailExpired)
        }
    } else {
        Err(Hook0Problem::Forbidden)
    }
}

#[api_v2_operation(
    summary = "Change password",
    description = "Change the password of a user.",
    operation_id = "auth.change_password",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn change_password(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
    body: Json<ChangePasswordPost>,
) -> Result<NoContent, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let body = body.into_inner();

    let token = authorize_only_user(
        &biscuit,
        None,
        Action::AuthChangePassword,
        state.max_authorization_time,
        state.debug_authorizer,
    )?;

    let password_hash = check_and_hash_new_password(
        &state.db,
        state.password_minimum_length,
        &body.new_password,
        token.user_id,
    )
    .await?;
    store_new_password(&state.db, password_hash.as_str(), token.user_id).await?;

    Ok(NoContent)
}

/// Run the policy against who the account belongs to, then hash.
///
/// Split from the write below so no connection is held for the ~100ms Argon2
/// deliberately costs: on the change-password path the connection comes from
/// the pool, and holding it across the hash would starve the pool one request
/// at a time.
async fn check_and_hash_new_password<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    password_minimum_length: u8,
    new_password: &str,
    user_id: Uuid,
) -> Result<PasswordHashString, Hook0Problem> {
    let mut db = db.acquire().await?;

    // Only the database knows who this account belongs to: neither the reset
    // link nor the biscuit carries the email address or the name.
    let identity = query!(
        "
            SELECT email, first_name, last_name
            FROM iam.user
            WHERE user__id = $1
        ",
        &user_id,
    )
    .fetch_optional(&mut *db)
    .await?
    .ok_or(Hook0Problem::Forbidden)?;

    let checked_password = password::Checked::new(
        new_password,
        password_minimum_length,
        &password::UserIdentity {
            email: &identity.email,
            first_name: &identity.first_name,
            last_name: &identity.last_name,
        },
    )
    .map_err(|rejection| rejection.into_problem(password_minimum_length))?;

    drop(db);

    password::hash(checked_password).await
}

/// Store an already checked and hashed password, and expire every token the
/// account had, so a stolen session does not survive the change.
async fn store_new_password<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    password_hash: &str,
    user_id: Uuid,
) -> Result<(), Hook0Problem> {
    let mut db = db.acquire().await?;
    let mut tx = db.begin().await?;

    query!(
        "
                UPDATE iam.user
                SET password = $1
                WHERE user__id = $2
            ",
        password_hash,
        &user_id,
    )
    .execute(&mut *tx)
    .await?;

    query!(
        "
                UPDATE iam.token
                SET expired_at = statement_timestamp()
                WHERE user__id = $1
                    AND (expired_at IS NULL OR expired_at > statement_timestamp())
            ",
        &user_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod password_policy_tests {
    use crate::google_ads::test_support::{issue_user_token, seed_org, seed_user, test_state};
    use actix_web::{App, test, web};
    use sqlx::PgPool;
    use uuid::Uuid;

    /// Spin up the real change-password endpoint behind the real biscuit auth
    /// middleware, over the test database. A macro rather than a function
    /// because the type of an initialized actix test service is not nameable
    /// here.
    macro_rules! init_api {
        ($pool:expr, $private_key:expr) => {{
            let state = test_state($pool.clone(), $private_key.clone(), None).await;
            let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
                db: $pool.clone(),
                biscuit_private_key: $private_key.clone(),
                master_api_key: None,
                enable_application_secret_compatibility: true,
            };

            test::init_service(
                App::new().app_data(web::Data::new(state)).service(
                    web::scope("/api/v1/auth").service(
                        web::resource("/password")
                            .wrap(biscuit_auth)
                            .route(web::post().to(super::change_password)),
                    ),
                ),
            )
            .await
        }};
    }

    /// POST a new password and return the status with the `id` of the problem
    /// it carries (empty when the response carries no problem).
    macro_rules! change_password {
        ($app:expr, $token:expr, $new_password:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/auth/password")
                .insert_header(("Authorization", format!("Bearer {}", $token)))
                .set_json(serde_json::json!({ "new_password": $new_password }))
                .to_request();
            let response = test::call_service(&$app, request).await;
            let status = response.status();
            let body = test::read_body(response).await;
            let problem = serde_json::from_slice::<serde_json::Value>(&body)
                .ok()
                .and_then(|body| body["id"].as_str().map(str::to_owned))
                .unwrap_or_default();
            (status, problem)
        }};
    }

    async fn stored_hash(pool: &PgPool, user_id: Uuid) -> String {
        sqlx::query_scalar("SELECT password FROM iam.user WHERE user__id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .expect("read stored password")
    }

    /// The change-password path is one of the three the policy must cover, and
    /// the only one where the account's identity is read back from the database
    /// rather than taken from the request. A wrong lookup would check the new
    /// password against somebody else's identity, and every other test would
    /// still pass.
    #[sqlx::test]
    async fn changing_to_the_account_email_address_is_refused(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let email: String = sqlx::query_scalar("SELECT email FROM iam.user WHERE user__id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .expect("read seeded email");

        let (status, problem) = change_password!(app, token, email);

        assert_eq!(status, actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(problem, "PasswordSimilarToEmail");
        assert_eq!(
            stored_hash(&pool, user).await,
            before,
            "a refused change must leave the stored password alone"
        );
    }

    #[sqlx::test]
    async fn changing_to_a_common_password_is_refused(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let app = init_api!(pool, private_key);
        let (status, problem) = change_password!(app, token, "2026letmein!");

        assert_eq!(status, actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(problem, "PasswordTooCommon");
    }

    /// The counterpart: a password the policy accepts must actually be stored,
    /// hashed. Without this, an implementation that refuses everything would
    /// pass every test above.
    #[sqlx::test]
    async fn changing_to_a_strong_password_replaces_the_stored_hash(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let (status, _) = change_password!(app, token, "quilt lantern harbour");

        assert!(status.is_success(), "unexpected status: {status}");

        let after = stored_hash(&pool, user).await;
        assert_ne!(after, before);
        assert!(
            after.starts_with("$argon2"),
            "the stored password must be an Argon2 hash, got {after:?}"
        );
    }
}
