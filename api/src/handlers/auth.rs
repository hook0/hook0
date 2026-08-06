use actix_web::rt::task::spawn_blocking;
use actix_web::web::ReqData;
use argon2::password_hash::PasswordHashString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use biscuit_auth::{Biscuit, PrivateKey};
use chrono::{DateTime, Utc};
use lettre::Address;
use lettre::message::Mailbox;
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgPool, Postgres, query, query_as, query_scalar};
use std::str::FromStr;
use tracing::{debug, error, warn};
use url::Url;
use uuid::Uuid;
use validator::Validate;

use crate::iam::{
    Action, authorize_email_verification, authorize_only_user, authorize_refresh_token,
    authorize_reset_password, create_refresh_token, create_reset_password_token,
    create_user_access_token,
};
use crate::mailer::{Mail, Mailer};
use crate::openapi::{OaBiscuitRefresh, OaBiscuitUserAccess};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct LoginPost {
    #[validate(non_control_character, length(min = 1, max = 100))]
    email: String,
    #[validate(
        non_control_character,
        length(
            min = 1,
            max = 100,
            message = "Password must be at least 10 characters long and at most 100 characters long"
        )
    )]
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
    #[validate(
        non_control_character,
        length(
            min = 10,
            max = 100,
            message = "Password must be at least 10 characters long and at most 100 characters long"
        )
    )]
    new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ChangePasswordPost {
    #[validate(
        non_control_character,
        length(
            min = 10,
            max = 100,
            message = "Password must be at least 10 characters long and at most 100 characters long"
        )
    )]
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

        let password_hash = generate_hashed_password(password).await?;

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

/// Cooldown between two verification emails for the same account. Enforced in
/// the database (a single timestamp column) so it holds across API replicas and
/// cannot be bypassed by rotating the source IP. Short enough to stay friendly
/// (a user who missed the first email retries quickly), long enough to make the
/// endpoint useless for mailbox flooding.
const RESEND_VERIFICATION_EMAIL_COOLDOWN_SECS: f64 = 60.0;

/// Length of the window over which resends are counted, and the cap that applies
/// inside it. The cooldown above only spaces sends out — on its own it still
/// allows ~1440 mails a day into one mailbox, which is a usable flooding tool for
/// a caller that can rotate source addresses. The cap bounds the total instead of
/// the rate.
///
/// Five is well above what recovery needs (the mail landed in spam, the address
/// had a typo, a corporate filter ate it: one to three attempts) and far below
/// what a mailbox would experience as abuse. A user who really exhausts it waits
/// out the window or writes to the support address offered on the same page.
///
/// The window is fixed, anchored on the first resend inside it, rather than a
/// true sliding window: that keeps the whole decision in one atomic statement
/// with two columns instead of a table of send timestamps. The cost is that a
/// caller straddling a window boundary can land at most 2 × the cap over an
/// arbitrary 24h span, which is still bounded and still small.
const RESEND_VERIFICATION_EMAIL_WINDOW_SECS: f64 = 24.0 * 60.0 * 60.0;
const RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW: i32 = 5;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ResendVerificationEmailPost {
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
}

#[api_v2_operation(
    summary = "Resend the email verification message",
    description = "Send a fresh verification link to a user whose email is not verified yet. Always answers the same way whether or not the email matches an account, so it never discloses which addresses are registered.",
    operation_id = "auth.resend_verification_email",
    consumes = "application/json",
    produces = "application/json",
    tags("User Authentication")
)]
pub async fn resend_verification_email(
    state: Data<crate::State>,
    body: Json<ResendVerificationEmailPost>,
) -> Result<NoContent, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let body = body.into_inner();

    // SECURITY: the dominant enumeration timing channel — the mail send — is
    // decoupled below via `tokio::spawn`, so response latency is independent of
    // whether the address matches an account. Two residual signals remain, both
    // accepted rather than closed:
    //
    //  * A microsecond-scale asymmetry: the claim below writes a row only on a
    //    match, and writing a row takes marginally longer than not writing one.
    //    Closing it would mean writing dummy rows for addresses that do not
    //    exist, which buys nothing in practice.
    //  * A larger one, and unlike the first it can be induced at will: on a
    //    match the claim takes a row lock, so two concurrent requests for the
    //    SAME address serialise on that lock, while two requests for an address
    //    that matches nothing never do. A caller able to fire simultaneous
    //    requests can measure that difference. It is left open deliberately —
    //    closing it would mean taking a lock keyed on addresses that do not
    //    exist, which is a denial-of-service surface of its own, and the lock is
    //    what keeps the cooldown and the cap atomic under concurrency.
    //
    // Everything else stays symmetric: the response is the same NoContent in
    // every case, and giving a failed attempt back (see below) happens on the
    // detached task, never on the request path.

    // Atomically claim the right to send: only an unverified account past its
    // cooldown and under its window cap matches, and the very same statement
    // stamps the new send time and moves the window counter along. Everything
    // else (unknown email, already verified, still within cooldown, cap reached)
    // matches no row and silently falls through to the identical response below.
    //
    // The claim reads the row `FOR UPDATE` before writing it so the values it
    // overwrites can be returned alongside the new ones — the quota is spent
    // before the mail is handed to SMTP, so a send that never happens has to be
    // able to put them back. The lock is also what makes the decision atomic:
    // two concurrent claims for the same account serialise on it, and the second
    // re-reads the row the first one left behind rather than a stale copy.
    let claim = query_as!(
        ResendClaim,
        r#"
            WITH claimable AS (
                SELECT
                    user__id,
                    email,
                    first_name,
                    last_name,
                    email_verification_sent_at,
                    email_verification_resend_window_started_at,
                    email_verification_resend_count
                FROM iam."user"
                WHERE email = $1
                  AND email_verified_at IS NULL
                  AND (
                      email_verification_sent_at IS NULL
                      OR email_verification_sent_at < statement_timestamp() - MAKE_INTERVAL(secs => $2)
                  )
                  AND (
                      email_verification_resend_window_started_at IS NULL
                      OR email_verification_resend_window_started_at < statement_timestamp() - MAKE_INTERVAL(secs => $3)
                      OR email_verification_resend_count < $4
                  )
                FOR UPDATE
            )
            UPDATE iam."user" AS u
            SET email_verification_sent_at = statement_timestamp(),
                email_verification_resend_window_started_at = CASE
                    WHEN c.email_verification_resend_window_started_at IS NULL
                      OR c.email_verification_resend_window_started_at < statement_timestamp() - MAKE_INTERVAL(secs => $3)
                    THEN statement_timestamp()
                    ELSE c.email_verification_resend_window_started_at
                END,
                email_verification_resend_count = CASE
                    WHEN c.email_verification_resend_window_started_at IS NULL
                      OR c.email_verification_resend_window_started_at < statement_timestamp() - MAKE_INTERVAL(secs => $3)
                    THEN 1
                    ELSE c.email_verification_resend_count + 1
                END
            FROM claimable AS c
            WHERE u.user__id = c.user__id
            RETURNING
                c.user__id AS "user_id!",
                c.email AS "email!",
                c.first_name AS "first_name!",
                c.last_name AS "last_name!",
                u.email_verification_sent_at AS "claimed_at!",
                c.email_verification_sent_at AS "previous_sent_at?",
                c.email_verification_resend_window_started_at AS "previous_window_started_at?",
                c.email_verification_resend_count AS "previous_resend_count!"
        "#,
        &body.email,
        RESEND_VERIFICATION_EMAIL_COOLDOWN_SECS,
        RESEND_VERIFICATION_EMAIL_WINDOW_SECS,
        RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW,
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(claim) = claim {
        // Decouple token creation and the mail send from the request path so the
        // response latency is identical whether or not the address matches an
        // account. Awaiting the send inline only for real accounts would leak
        // their existence through timing; running it in a detached task (like
        // the signup-attribution cleanup job) keeps the response constant-time.
        // Every failure is logged (never with PII) and swallowed — the
        // anti-enumeration response is always NoContent.
        let private_key = state.biscuit_private_key.clone();
        let app_url = state.app_url.clone();
        let mailer = state.mailer.clone();
        let db = state.db.clone();
        tokio::spawn(async move {
            if let Err(e) = send_verification_email(&private_key, &app_url, &mailer, &claim).await {
                warn!(
                    "Could not resend verification email to user {}: {e}",
                    claim.user_id
                );
                // Nothing was sent, so the attempt goes back. The quota exists to
                // bound the mail that actually reaches a mailbox; charging it for
                // mail that never left would let an SMTP outage burn a user's
                // whole daily allowance in minutes and lock them out of the only
                // recovery path they have for a day.
                if let Err(e) = release_resend_claim(&db, &claim).await {
                    error!(
                        "Could not give back the verification email resend attempt of user {}: {e}",
                        claim.user_id
                    );
                }
            }
        });
    }

    Ok(NoContent)
}

/// A claimed right to send one verification email, together with the row values
/// the claim overwrote.
///
/// `claimed_at` is the send stamp the claim wrote. It doubles as the proof that
/// the claim is still the most recent one: the release only applies while that
/// exact stamp is still on the row, so a later claim that did send is never
/// rolled back underneath.
struct ResendClaim {
    user_id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    claimed_at: DateTime<Utc>,
    previous_sent_at: Option<DateTime<Utc>>,
    previous_window_started_at: Option<DateTime<Utc>>,
    previous_resend_count: i32,
}

/// Mint a fresh verification token and mail it to the account the claim was made
/// for. Errors carry no address: the caller logs them next to a user id.
async fn send_verification_email(
    biscuit_private_key: &PrivateKey,
    app_url: &Url,
    mailer: &Mailer,
    claim: &ResendClaim,
) -> anyhow::Result<()> {
    let verification_token =
        crate::iam::create_email_verification_token(biscuit_private_key, claim.user_id)
            .map_err(|e| anyhow::anyhow!("could not create an email verification token: {e}"))?;

    let mut url = app_url
        .join("verify-email")
        .map_err(|e| anyhow::anyhow!("could not build the verify-email URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("token", &verification_token.serialized_biscuit);

    let address = Address::from_str(&claim.email)
        .map_err(|e| anyhow::anyhow!("could not parse the recipient address: {e}"))?;
    let mailbox = Mailbox::new(
        Some(format!("{} {}", claim.first_name, claim.last_name)),
        address,
    );
    let mail = Mail::VerifyUserEmail {
        recipient_first_name: Some(claim.first_name.clone()),
        url,
    };

    mailer
        .send_mail(mail, mailbox)
        .await
        .map_err(|e| anyhow::anyhow!("could not send the verification email: {e}"))?;
    Ok(())
}

/// Put back what a claim took when the mail it was claimed for never left.
///
/// The window counter and its anchor go back to the values the claim overwrote,
/// and so does the send stamp the cooldown reads — the attempt cost the user
/// nothing because it bought them nothing.
///
/// Guarded on the stamp the claim wrote: if a later claim has since moved it,
/// that one either sent a mail or is still trying, and rolling it back would
/// hand out an extra send. In that case the statement matches no row and the
/// release is simply dropped.
async fn release_resend_claim(db: &PgPool, claim: &ResendClaim) -> Result<(), sqlx::Error> {
    query!(
        r#"
            UPDATE iam."user"
            SET email_verification_sent_at = $2,
                email_verification_resend_window_started_at = $3,
                email_verification_resend_count = $4
            WHERE user__id = $1
              AND email_verification_sent_at = $5
        "#,
        claim.user_id,
        claim.previous_sent_at,
        claim.previous_window_started_at,
        claim.previous_resend_count,
        claim.claimed_at,
    )
    .execute(db)
    .await?;
    Ok(())
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

            do_change_password(
                &mut tx,
                state.password_minimum_length,
                &body.new_password,
                user_id,
            )
            .await?;

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

    do_change_password(
        &state.db,
        state.password_minimum_length,
        &body.new_password,
        token.user_id,
    )
    .await?;

    Ok(NoContent)
}

async fn do_change_password<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    password_minimum_length: u8,
    new_password: &str,
    user_id: Uuid,
) -> Result<(), Hook0Problem> {
    if new_password.len() >= usize::from(password_minimum_length) {
        let password_hash = generate_hashed_password(new_password).await?;

        let mut db = db.acquire().await?;
        let mut tx = db.begin().await?;

        query!(
            "
                UPDATE iam.user
                SET password = $1
                WHERE user__id = $2
            ",
            password_hash.as_str(),
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
    } else {
        Err(Hook0Problem::PasswordTooShort(password_minimum_length))
    }
}

async fn generate_hashed_password(password: &str) -> Result<PasswordHashString, Hook0Problem> {
    let password = password.to_owned();

    spawn_blocking(move || {
        let salt = argon2::password_hash::SaltString::generate(
            &mut argon2::password_hash::rand_core::OsRng,
        );
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                error!("Error trying to hash user password: {e}");
                Hook0Problem::InternalServerError
            })
            .map(|h| h.serialize())
    })
    .await
    .map_err(|e| {
        error!("Failed to run password hashing task: {e}");
        Hook0Problem::InternalServerError
    })?
}

#[cfg(test)]
mod resend_verification_email_tests {
    use crate::google_ads::test_support::{
        DEAD_SMTP_CONNECTION_URL, seed_user, test_state_with_smtp,
    };
    use crate::mailer::test_support::FakeSmtp;
    use actix_web::{App, http::StatusCode, test, web};
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;
    use std::time::Duration;
    use uuid::Uuid;

    /// How long a test waits for the detached send (and, when it fails, the
    /// release that follows) to finish. Bounded so a broken build fails instead
    /// of hanging.
    const DETACHED_WORK_TIMEOUT: Duration = Duration::from_secs(10);

    /// How long a test waits before concluding that no further mail is coming.
    /// A send that was wrongly permitted is spawned the moment the request is
    /// answered and reaches a loopback server in milliseconds, so this is orders
    /// of magnitude more than it would need — and short enough that the whole
    /// suite is not spent proving negatives.
    const NO_FURTHER_SEND_WINDOW: Duration = Duration::from_secs(3);

    /// Insert a user whose email is NOT verified yet (the default: the column is
    /// nullable and left NULL), returning its id.
    async fn seed_unverified_user(pool: &PgPool, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
                INSERT INTO iam."user" (user__id, email, password, first_name, last_name)
                VALUES ($1, $2, 'unused-hash', 'Test', 'User')
            "#,
        )
        .bind(user_id)
        .bind(email)
        .execute(pool)
        .await
        .expect("seed unverified user");
        user_id
    }

    /// Read back when the verification email was last (re)sent for a user.
    async fn verification_sent_at(pool: &PgPool, user_id: Uuid) -> Option<DateTime<Utc>> {
        let row: (Option<DateTime<Utc>>,) = sqlx::query_as(
            r#"SELECT email_verification_sent_at FROM iam."user" WHERE user__id = $1"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("read email_verification_sent_at");
        row.0
    }

    /// Read back how many resends are recorded in the user's current window.
    async fn resend_count(pool: &PgPool, user_id: Uuid) -> i32 {
        let row: (i32,) = sqlx::query_as(
            r#"SELECT email_verification_resend_count FROM iam."user" WHERE user__id = $1"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("read email_verification_resend_count");
        row.0
    }

    /// Age the last-send stamp out of its cooldown, leaving the window counter
    /// untouched. Lets a test drive several resends back to back so the only
    /// control left standing is the per-window cap.
    async fn expire_cooldown(pool: &PgPool, user_id: Uuid) {
        sqlx::query(
            r#"
                UPDATE iam."user"
                SET email_verification_sent_at =
                    statement_timestamp() - MAKE_INTERVAL(secs => $2)
                WHERE user__id = $1
            "#,
        )
        .bind(user_id)
        .bind(super::RESEND_VERIFICATION_EMAIL_COOLDOWN_SECS * 2.0)
        .execute(pool)
        .await
        .expect("age the cooldown out");
    }

    /// Age the counting window past its length, as if a day had gone by.
    async fn expire_resend_window(pool: &PgPool, user_id: Uuid) {
        sqlx::query(
            r#"
                UPDATE iam."user"
                SET email_verification_resend_window_started_at =
                    statement_timestamp() - MAKE_INTERVAL(secs => $2)
                WHERE user__id = $1
            "#,
        )
        .bind(user_id)
        .bind(super::RESEND_VERIFICATION_EMAIL_WINDOW_SECS + 60.0)
        .execute(pool)
        .await
        .expect("age the counting window out");
    }

    /// Read back the anchor of the user's current resend window.
    async fn resend_window_started_at(pool: &PgPool, user_id: Uuid) -> Option<DateTime<Utc>> {
        let row: (Option<DateTime<Utc>>,) = sqlx::query_as(
            r#"SELECT email_verification_resend_window_started_at FROM iam."user" WHERE user__id = $1"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("read email_verification_resend_window_started_at");
        row.0
    }

    /// Wait, bounded, until the detached task has given a failed attempt back:
    /// the send stamp and the window counter are what they were before the
    /// claim. Returns whether that happened before the timeout.
    async fn wait_for_released_claim(
        pool: &PgPool,
        user_id: Uuid,
        expected_sent_at: Option<DateTime<Utc>>,
        expected_count: i32,
    ) -> bool {
        let steps = DETACHED_WORK_TIMEOUT.as_millis() / 25;
        for _ in 0..steps {
            if verification_sent_at(pool, user_id).await == expected_sent_at
                && resend_count(pool, user_id).await == expected_count
            {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        false
    }

    /// Build a test service exposing only the resend endpoint against `pool`,
    /// talking to the SMTP endpoint at `smtp_connection_url`, and POST `email`
    /// to it.
    async fn resend_via(
        pool: &PgPool,
        smtp_connection_url: &str,
        email: &str,
    ) -> actix_web::dev::ServiceResponse {
        let keypair = biscuit_auth::KeyPair::new();
        let state = test_state_with_smtp(
            pool.clone(),
            keypair.private().clone(),
            None,
            smtp_connection_url,
        )
        .await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1").service(
                    web::scope("/auth").service(
                        web::resource("/resend-verification-email")
                            .route(web::post().to(super::resend_verification_email)),
                    ),
                ),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/resend-verification-email")
            .set_json(serde_json::json!({ "email": email }))
            .to_request();
        test::call_service(&app, req).await
    }

    /// A resend for an unverified account answers NoContent, hands a real
    /// message to a real SMTP server, and stamps the send time (so the cooldown
    /// starts ticking).
    #[sqlx::test]
    async fn resend_for_unverified_user_sends_and_stamps(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("unverified-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        assert!(
            verification_sent_at(&pool, user_id).await.is_none(),
            "precondition: no verification email recorded yet"
        );

        let resp = resend_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        assert_eq!(
            smtp.wait_for(1, DETACHED_WORK_TIMEOUT).await,
            1,
            "a verification email must actually reach the SMTP server"
        );
        assert!(
            verification_sent_at(&pool, user_id).await.is_some(),
            "a verification email was sent, so the send time is stamped"
        );
    }

    /// The response for an address that matches no account is byte-for-byte
    /// identical to the response for a real unverified account: the endpoint
    /// never reveals which emails are registered.
    #[sqlx::test]
    async fn resend_response_is_identical_for_unknown_and_known_email(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let known = format!("known-{}@example.com", Uuid::new_v4());
        seed_unverified_user(&pool, &known).await;
        let unknown = format!("nobody-{}@example.com", Uuid::new_v4());

        let known_resp = resend_via(&pool, &smtp.connection_url, &known).await;
        let known_status = known_resp.status();
        let known_body = test::read_body(known_resp).await;

        let unknown_resp = resend_via(&pool, &smtp.connection_url, &unknown).await;
        let unknown_status = unknown_resp.status();
        let unknown_body = test::read_body(unknown_resp).await;

        assert_eq!(known_status, StatusCode::NO_CONTENT);
        assert_eq!(
            unknown_status, known_status,
            "status must not leak existence"
        );
        assert_eq!(unknown_body, known_body, "body must not leak existence");

        assert_eq!(
            smtp.wait_for(1, DETACHED_WORK_TIMEOUT).await,
            1,
            "the known address is mailed"
        );
        assert_eq!(
            smtp.wait_for(2, NO_FURTHER_SEND_WINDOW).await,
            1,
            "the unknown address is not, and the response says nothing about either"
        );
    }

    /// An already-verified account is never (re)sent a verification email: the
    /// response is still NoContent, but nothing is stamped and nothing leaves.
    #[sqlx::test]
    async fn resend_is_a_noop_for_a_verified_user(pool: PgPool) {
        let smtp = FakeSmtp::start();
        // `seed_user` creates a *verified* user with a deterministic email.
        let user_id = seed_user(&pool).await;
        let email = format!("e2e-{user_id}@example.com");

        let resp = resend_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        assert_eq!(
            smtp.wait_for(1, NO_FURTHER_SEND_WINDOW).await,
            0,
            "verified users are never mailed a verification link"
        );
        assert!(
            verification_sent_at(&pool, user_id).await.is_none(),
            "verified users are never sent a verification email"
        );
    }

    /// A second resend within the cooldown window is throttled: it answers the
    /// same NoContent but does not send again (the stamped send time is
    /// unchanged and no second message reaches the server).
    #[sqlx::test]
    async fn resend_is_rate_limited_per_email(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("throttled-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        let first = resend_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(first.status(), StatusCode::NO_CONTENT);
        assert_eq!(smtp.wait_for(1, DETACHED_WORK_TIMEOUT).await, 1);
        let first_sent = verification_sent_at(&pool, user_id)
            .await
            .expect("first resend stamps the send time");

        let second = resend_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(second.status(), StatusCode::NO_CONTENT);
        let second_sent = verification_sent_at(&pool, user_id)
            .await
            .expect("send time is still present after the throttled attempt");

        assert_eq!(
            first_sent, second_sent,
            "a resend within the cooldown must not send again"
        );
        assert_eq!(
            smtp.wait_for(2, NO_FURTHER_SEND_WINDOW).await,
            1,
            "a resend within the cooldown must not put a second mail on the wire"
        );
    }

    /// The 60s cooldown only spaces sends out; the cap bounds their total. Once
    /// an account has had its allowance for the window, further calls send
    /// nothing — even with the cooldown out of the way, which is exactly the
    /// position a distributed caller is in.
    #[sqlx::test]
    async fn resend_is_capped_per_account_within_one_window(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("capped-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        for expected in 1..=super::RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW {
            let resp = resend_via(&pool, &smtp.connection_url, &email).await;
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
            assert_eq!(
                resend_count(&pool, user_id).await,
                expected,
                "each allowed resend must move the window counter along"
            );
            let delivered = usize::try_from(expected).expect("small positive count");
            assert_eq!(
                smtp.wait_for(delivered, DETACHED_WORK_TIMEOUT).await,
                delivered
            );
            expire_cooldown(&pool, user_id).await;
        }

        // The allowance is spent. Nothing stands in the way but the cap.
        let sent_before_capped_attempt = verification_sent_at(&pool, user_id)
            .await
            .expect("the allowed resends stamped a send time");
        let allowance =
            usize::try_from(super::RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW).expect("small cap");

        let capped = resend_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(
            capped.status(),
            StatusCode::NO_CONTENT,
            "reaching the cap must be invisible to the caller (anti-enumeration)"
        );
        assert_eq!(
            verification_sent_at(&pool, user_id).await,
            Some(sent_before_capped_attempt),
            "a capped resend must not send, so it must not stamp a new send time"
        );
        assert_eq!(
            resend_count(&pool, user_id).await,
            super::RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW,
            "a capped resend must not consume more allowance either"
        );
        assert_eq!(
            smtp.wait_for(allowance + 1, NO_FURTHER_SEND_WINDOW).await,
            allowance,
            "the cap is what bounds the mail a mailbox receives, so nothing more leaves"
        );
    }

    /// The cap is a bound per window, not a permanent lock-out: once the window
    /// has gone by, the account can be helped again and the counter restarts.
    #[sqlx::test]
    async fn resend_cap_lifts_once_the_window_has_elapsed(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("window-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        for _ in 0..super::RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW {
            assert_eq!(
                resend_via(&pool, &smtp.connection_url, &email)
                    .await
                    .status(),
                StatusCode::NO_CONTENT
            );
            expire_cooldown(&pool, user_id).await;
        }
        let sent_while_capped = verification_sent_at(&pool, user_id)
            .await
            .expect("the allowed resends stamped a send time");
        assert_eq!(
            resend_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            verification_sent_at(&pool, user_id).await,
            Some(sent_while_capped),
            "precondition: the account is capped"
        );

        expire_resend_window(&pool, user_id).await;

        assert_eq!(
            resend_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_ne!(
            verification_sent_at(&pool, user_id).await,
            Some(sent_while_capped),
            "a new window must let the account be sent to again"
        );
        assert_eq!(
            resend_count(&pool, user_id).await,
            1,
            "the counter restarts with the new window rather than carrying over"
        );
    }

    /// An attempt that never became a mail is given back. The quota bounds what
    /// reaches a mailbox, so a send that fails must leave the account exactly as
    /// it found it: same send stamp, same window anchor, same counter.
    #[sqlx::test]
    async fn resend_gives_the_attempt_back_when_the_mail_cannot_be_sent(pool: PgPool) {
        let email = format!("smtp-down-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        let resp = resend_via(&pool, DEAD_SMTP_CONNECTION_URL, &email).await;
        assert_eq!(
            resp.status(),
            StatusCode::NO_CONTENT,
            "a failing send must stay invisible to the caller (anti-enumeration)"
        );

        assert!(
            wait_for_released_claim(&pool, user_id, None, 0).await,
            "a send that never happened must give its attempt back"
        );
        assert!(
            resend_window_started_at(&pool, user_id).await.is_none(),
            "the window anchor goes back too, so the released attempt opened no window"
        );

        // And the account really is claimable again straight away: with a server
        // that answers, the very next call delivers.
        let smtp = FakeSmtp::start();
        assert_eq!(
            resend_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            smtp.wait_for(1, DETACHED_WORK_TIMEOUT).await,
            1,
            "the cooldown must not stand in the way of an attempt that sent nothing"
        );
    }

    /// The defect this guards against: an SMTP outage used to spend the whole
    /// 24h allowance in minutes while the user received nothing, locking them
    /// out of the only recovery path they have for a day. More failed attempts
    /// than the cap allows must still leave the account able to be mailed.
    #[sqlx::test]
    async fn an_smtp_outage_does_not_burn_the_daily_allowance(pool: PgPool) {
        let email = format!("outage-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        for _ in 0..=super::RESEND_VERIFICATION_EMAIL_MAX_PER_WINDOW {
            assert_eq!(
                resend_via(&pool, DEAD_SMTP_CONNECTION_URL, &email)
                    .await
                    .status(),
                StatusCode::NO_CONTENT
            );
            assert!(
                wait_for_released_claim(&pool, user_id, None, 0).await,
                "every attempt that sent nothing must be given back"
            );
        }

        let smtp = FakeSmtp::start();
        assert_eq!(
            resend_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            smtp.wait_for(1, DETACHED_WORK_TIMEOUT).await,
            1,
            "once the mail server is back, the user must still have their allowance"
        );
    }
}

#[cfg(test)]
mod verify_email_single_use_tests {
    use crate::google_ads::test_support::test_state;
    use crate::iam::create_email_verification_token;
    use actix_web::{App, http::StatusCode, test, web};
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    /// Insert a user whose email is NOT verified yet, returning its id.
    async fn seed_unverified_user(pool: &PgPool, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
                INSERT INTO iam."user" (user__id, email, password, first_name, last_name)
                VALUES ($1, $2, 'unused-hash', 'Test', 'User')
            "#,
        )
        .bind(user_id)
        .bind(email)
        .execute(pool)
        .await
        .expect("seed unverified user");
        user_id
    }

    /// Read back when the account was marked verified (NULL until it is).
    async fn email_verified_at(pool: &PgPool, user_id: Uuid) -> Option<DateTime<Utc>> {
        let row: (Option<DateTime<Utc>>,) =
            sqlx::query_as(r#"SELECT email_verified_at FROM iam."user" WHERE user__id = $1"#)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .expect("read email_verified_at");
        row.0
    }

    /// Build a test service exposing only the verify-email endpoint against
    /// `pool`, and POST `token` to it. The SMTP transport in `test_state` points
    /// at a dead port, so the post-verification welcome mail fails fast and is
    /// swallowed — verification itself still answers, which is the behaviour under
    /// test.
    async fn verify(
        pool: &PgPool,
        private_key: &biscuit_auth::PrivateKey,
        token: &str,
    ) -> actix_web::dev::ServiceResponse {
        let state = test_state(pool.clone(), private_key.clone(), None).await;
        let app = test::init_service(App::new().app_data(web::Data::new(state)).service(
            web::scope("/api/v1").service(web::scope("/auth").service(
                web::resource("/verify-email").route(web::post().to(super::verify_email)),
            )),
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/verify-email")
            .set_json(serde_json::json!({ "token": token }))
            .to_request();
        test::call_service(&app, req).await
    }

    /// After the verification-token TTL was extended to 24h, the token is still
    /// single-use. A freshly minted token is well within its 24h validity window,
    /// so authorization succeeds on BOTH calls — yet the second call must be a
    /// no-op: single use is enforced by the `email_verified_at IS NULL` guard in
    /// the handler, not by token expiry. The replayed call neither verifies a
    /// second time (no second NoContent, no new session) nor re-stamps
    /// `email_verified_at`.
    #[sqlx::test]
    async fn verification_token_stays_single_use_within_its_24h_ttl(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let email = format!("verify-once-{}@example.com", Uuid::new_v4());
        let user_id = seed_unverified_user(&pool, &email).await;

        let token = create_email_verification_token(&keypair.private(), user_id)
            .expect("create verification token")
            .serialized_biscuit;

        assert!(
            email_verified_at(&pool, user_id).await.is_none(),
            "precondition: the account is not verified yet"
        );

        // First use: verifies the account.
        let first = verify(&pool, &keypair.private(), &token).await;
        assert_eq!(
            first.status(),
            StatusCode::NO_CONTENT,
            "the first use of a valid token verifies the account"
        );
        let verified_at = email_verified_at(&pool, user_id)
            .await
            .expect("first verification stamps email_verified_at");

        // Second use of the SAME, still-valid token: must be rejected and change
        // nothing.
        let second = verify(&pool, &keypair.private(), &token).await;
        assert_ne!(
            second.status(),
            StatusCode::NO_CONTENT,
            "a replayed verification token must not verify a second time"
        );
        let verified_at_after = email_verified_at(&pool, user_id)
            .await
            .expect("email_verified_at is still set after the replayed attempt");
        assert_eq!(
            verified_at, verified_at_after,
            "the replayed token must not re-stamp email_verified_at (no second mutation)"
        );
    }
}
