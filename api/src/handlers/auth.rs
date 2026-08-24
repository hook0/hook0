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
use sqlx::{Acquire, PgPool, Postgres, query, query_as, query_scalar};
use std::str::FromStr;
use std::time::Duration;
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
use crate::password;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct LoginPost {
    #[validate(non_control_character, length(min = 1, max = 100))]
    email: String,
    // Bounded, but with no policy of its own: logging in must accept whatever
    // the account's password happens to be, including one set before the policy
    // existed. `secret` bounds it without echoing it back.
    #[validate(custom(function = "crate::validators::secret"))]
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
    // A bearer credential: `secret_token` bounds it without echoing it back.
    #[validate(custom(function = "crate::validators::secret_token"))]
    token: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct BeginResetPasswordPost {
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ResetPasswordPost {
    // See `EmailVerificationPost::token`.
    #[validate(custom(function = "crate::validators::secret_token"))]
    token: String,
    // Length is deliberately not validated here: the policy owns both bounds
    // (`password::Checked::new`), so the user is told the instance's real
    // minimum instead of a number hardcoded next to the field, and an
    // oversized one is refused as `PasswordTooLong` rather than as malformed
    // input. `secret_characters` keeps the refused password out of the
    // response body, which the built-in validators echo back.
    #[validate(custom(function = "crate::validators::secret_characters"))]
    new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ChangePasswordPost {
    // Proof that whoever holds this session also knows the password it is about
    // to replace. Unlike `new_password`, no policy bounds it downstream: it goes
    // straight to `verify_password`, which will hash whatever it is handed. So
    // it is bounded here, exactly like `LoginPost::password` — the other field
    // that carries a password nothing else will size — by a validator that
    // bounds without echoing the value it refused.
    #[validate(custom(function = "crate::validators::secret"))]
    current_password: String,
    // See `ResetPasswordPost::new_password`: the policy owns the bounds.
    #[validate(custom(function = "crate::validators::secret_characters"))]
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
        let password_valid = verify_password(
            user.password_hash.clone(),
            body.password.clone(),
            user.user_id,
        )
        .await?;

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

/// Check a candidate against a stored Argon2 hash, off the async runtime:
/// verifying costs the same ~100ms hashing does and would otherwise block the
/// whole worker thread.
///
/// Shared by every path that has to prove the caller knows the current password,
/// so none of them can end up checking it a slightly different way.
async fn verify_password(
    password_hash: String,
    candidate: String,
    user_id: Uuid,
) -> Result<bool, Hook0Problem> {
    spawn_blocking(move || -> Result<bool, Hook0Problem> {
        let parsed_hash = PasswordHash::new(&password_hash).map_err(|e| {
            error!(
                "Password hash of user {} is not in the right format: {e}",
                &user_id
            );
            Hook0Problem::InternalServerError
        })?;

        Ok(Argon2::default()
            .verify_password(candidate.as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await
    .map_err(|e| {
        error!("Failed to run password verification task: {e}");
        Hook0Problem::InternalServerError
    })?
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
    let token = authorize_refresh_token(&biscuit, state.max_authorization_time)?;

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
) -> Result<CreatedJson<LoginResponse>, Hook0Problem> {
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

            // Open a session right away so the user lands authenticated on the
            // wizard instead of being bounced to the login form. This is safe:
            // the `email_verified_at IS NULL` guard on the UPDATE above means
            // this branch runs at most once per user, on the single
            // unverified→verified transition, so a replayed (still valid)
            // verification token can never mint a second session. `password_hash`
            // is required by `UserLookup` but never read by `do_login`.
            let session_user = UserLookup {
                user_id: token.user_id,
                password_hash: String::new(),
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                email_verified_at: Some(Utc::now()),
            };
            do_login(&state.db, &state.biscuit_private_key, session_user, None).await
        } else {
            // Nothing to update: the link was already used, or the account is
            // gone. Telling the two apart is the difference between "sign in"
            // and "start over", and it leaks nothing — a valid signature already
            // proves the caller holds a link we issued for that very user.
            let already_verified = query_scalar!(
                "SELECT email_verified_at IS NOT NULL FROM iam.user WHERE user__id = $1",
                &token.user_id,
            )
            .fetch_optional(&state.db)
            .await?
            .flatten()
            .unwrap_or(false);

            if already_verified {
                debug!(
                    "User {} tried to verify its email whereas it is already verified",
                    &token.user_id
                );
                Err(Hook0Problem::AuthEmailAlreadyVerified)
            } else {
                Err(Hook0Problem::AuthEmailExpired)
            }
        }
    } else {
        Err(Hook0Problem::AuthEmailExpired)
    }
}

/// Cooldown between two reset emails for the same address, and the window over
/// which they are counted with the cap that applies inside it. Same shape, same
/// values and same reasoning as the verification-email quota below: the cooldown
/// spaces sends out, the cap is what bounds the mail a mailbox actually
/// receives. Enforced in the database so it holds across API replicas and
/// survives a caller rotating its source address, which the per-IP limiter in
/// front of the endpoint cannot.
const BEGIN_RESET_PASSWORD_COOLDOWN: Duration = Duration::from_secs(60);
const BEGIN_RESET_PASSWORD_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
const BEGIN_RESET_PASSWORD_MAX_PER_WINDOW: i32 = 5;

#[api_v2_operation(
    summary = "Begin reset password",
    description = "Send an email with a link to reset the password of a user. Always answers the same way whether or not the email matches an account, so it never discloses which addresses are registered.",
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

    // SECURITY: this endpoint used to answer 204 for an address it knew and 401
    // for one it did not, which made it a plain account-existence oracle — one
    // the login page exposed without any tooling. It now answers NoContent in
    // every case, including when the mail cannot be sent, and the mail send is
    // detached so the response time says nothing either. The residual signals
    // are the same two `resend_verification_email` documents below, and they are
    // accepted for the same reasons.

    // Atomically claim the right to send: only an account past its cooldown and
    // under its window cap matches, and the very same statement rotates the
    // reset nonce — so the link this call is about to mint is the only one that
    // still works — and moves the quota along. Everything else (unknown email,
    // still within cooldown, cap reached) matches no row and falls through to
    // the identical response below.
    //
    // Rotating the nonce and claiming the quota have to be the same statement:
    // done separately, two concurrent calls could interleave into a link whose
    // nonce a later rotation had already retired before the mail even left.
    //
    // The claim reads the row `FOR UPDATE` before writing it so the values it
    // overwrites can be returned alongside the new ones — the allowance AND the
    // nonce are spent before the mail is handed to SMTP, so a send that never
    // happens has to be able to put both back.
    let claim = query_as!(
        ResetPasswordClaim,
        r#"
            WITH claimable AS (
                SELECT
                    user__id,
                    email,
                    first_name,
                    last_name,
                    password_reset_nonce,
                    password_reset_sent_at,
                    password_reset_window_started_at,
                    password_reset_count
                FROM iam."user"
                WHERE email = $1
                  AND (
                      password_reset_sent_at IS NULL
                      OR password_reset_sent_at < statement_timestamp() - MAKE_INTERVAL(secs => $2)
                  )
                  AND (
                      password_reset_window_started_at IS NULL
                      OR password_reset_window_started_at < statement_timestamp() - MAKE_INTERVAL(secs => $3)
                      OR password_reset_count < $4
                  )
                FOR UPDATE
            )
            UPDATE iam."user" AS u
            SET password_reset_nonce = public.gen_random_uuid(),
                password_reset_sent_at = statement_timestamp(),
                password_reset_window_started_at = CASE
                    WHEN c.password_reset_window_started_at IS NULL
                      OR c.password_reset_window_started_at < statement_timestamp() - MAKE_INTERVAL(secs => $3)
                    THEN statement_timestamp()
                    ELSE c.password_reset_window_started_at
                END,
                password_reset_count = CASE
                    WHEN c.password_reset_window_started_at IS NULL
                      OR c.password_reset_window_started_at < statement_timestamp() - MAKE_INTERVAL(secs => $3)
                    THEN 1
                    ELSE c.password_reset_count + 1
                END
            FROM claimable AS c
            WHERE u.user__id = c.user__id
            RETURNING
                c.user__id AS "user_id!",
                c.email AS "email!",
                c.first_name AS "first_name!",
                c.last_name AS "last_name!",
                u.password_reset_nonce AS "nonce!",
                u.password_reset_sent_at AS "claimed_at!",
                c.password_reset_nonce AS "previous_nonce!",
                c.password_reset_sent_at AS "previous_sent_at?",
                c.password_reset_window_started_at AS "previous_window_started_at?",
                c.password_reset_count AS "previous_count!"
        "#,
        &body.email,
        BEGIN_RESET_PASSWORD_COOLDOWN.as_secs_f64(),
        BEGIN_RESET_PASSWORD_WINDOW.as_secs_f64(),
        BEGIN_RESET_PASSWORD_MAX_PER_WINDOW,
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(claim) = claim {
        let private_key = state.biscuit_private_key.clone();
        let app_url = state.app_url.clone();
        let mailer = state.mailer.clone();
        let db = state.db.clone();
        tokio::spawn(async move {
            if let Err(e) = send_reset_password_email(&private_key, &app_url, &mailer, &claim).await
            {
                warn!(
                    "Could not send reset password email to user {}: {e}",
                    claim.user_id
                );
                if let Err(e) = release_reset_password_claim(&db, &claim).await {
                    error!(
                        "Could not give back the password reset attempt of user {}: {e}",
                        claim.user_id
                    );
                }
            }
        });
    }

    Ok(NoContent)
}

/// A claimed right to send one reset email, together with the quota values the
/// claim overwrote and the nonce it minted.
///
/// `claimed_at` doubles as the proof that the claim is still the most recent
/// one: the release only applies while that exact stamp is still on the row.
struct ResetPasswordClaim {
    user_id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    nonce: Uuid,
    claimed_at: DateTime<Utc>,
    previous_nonce: Uuid,
    previous_sent_at: Option<DateTime<Utc>>,
    previous_window_started_at: Option<DateTime<Utc>>,
    previous_count: i32,
}

/// Mint a reset link carrying the nonce the claim rotated in, and mail it.
/// Errors carry no address: the caller logs them next to a user id.
async fn send_reset_password_email(
    biscuit_private_key: &PrivateKey,
    app_url: &Url,
    mailer: &Mailer,
    claim: &ResetPasswordClaim,
) -> anyhow::Result<()> {
    let reset_token = create_reset_password_token(biscuit_private_key, claim.user_id, claim.nonce)
        .map_err(|e| anyhow::anyhow!("could not create a reset password token: {e}"))?;

    let mut url = app_url
        .join("reset-password")
        .map_err(|e| anyhow::anyhow!("could not build the reset-password URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("token", &reset_token.serialized_biscuit);

    let address = Address::from_str(&claim.email)
        .map_err(|e| anyhow::anyhow!("could not parse the recipient address: {e}"))?;
    let mailbox = Mailbox::new(
        Some(format!("{} {}", claim.first_name, claim.last_name)),
        address,
    );
    let mail = Mail::ResetPassword {
        recipient_first_name: Some(claim.first_name.clone()),
        url,
    };

    mailer
        .send_mail(mail, mailbox)
        .await
        .map_err(|e| anyhow::anyhow!("could not send the reset password email: {e}"))?;
    Ok(())
}

/// Put back what a claim took when the mail it was claimed for never left, so an
/// SMTP outage cannot burn a user's daily allowance and close the only recovery
/// path they have.
///
/// The nonce goes back too, and that is the security-carrying half. The claim
/// rotates it before the mail is handed to SMTP, so a send that fails has
/// retired the link the user already has in their mailbox while putting nothing
/// in its place. Leaving it rotated turns any address into a way of retiring
/// somebody else's live link — and, because the release also gives the cooldown
/// stamp back, of doing it again immediately, without bound.
///
/// Putting it back is safe precisely because the mail never left: no link was
/// ever delivered carrying the value being discarded, so nothing comes back to
/// life that the account did not already hold. The one case where a link
/// carrying it could exist is a send that reached the mail server and then
/// reported failure; both links are then in the same mailbox, and the older one
/// is the one that works.
///
/// Guarded on both the stamp and the nonce the claim wrote. The stamp alone
/// catches a later claim, but not a password write: changing the password from
/// the settings page — and using a link — rotates the nonce and leaves the stamp
/// where it was, so a release landing afterwards would put the previous nonce
/// back and bring an already-retired link back to life, for the remainder of its
/// lifetime, right after its owner acted to kill it. Matching on the nonce means
/// the release only undoes a claim nothing has superseded.
async fn release_reset_password_claim(
    db: &PgPool,
    claim: &ResetPasswordClaim,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
            UPDATE iam."user"
            SET password_reset_nonce = $2,
                password_reset_sent_at = $3,
                password_reset_window_started_at = $4,
                password_reset_count = $5
            WHERE user__id = $1
              AND password_reset_sent_at = $6
              AND password_reset_nonce = $7
        "#,
        claim.user_id,
        claim.previous_nonce,
        claim.previous_sent_at,
        claim.previous_window_started_at,
        claim.previous_count,
        claim.claimed_at,
        claim.nonce,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Cooldown between two verification emails for the same account. Enforced in
/// the database (a single timestamp column) so it holds across API replicas and
/// cannot be bypassed by rotating the source IP. Short enough to stay friendly
/// (a user who missed the first email retries quickly), long enough to make the
/// endpoint useless for mailbox flooding.
const RESEND_VERIFICATION_EMAIL_COOLDOWN: Duration = Duration::from_secs(60);

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
const RESEND_VERIFICATION_EMAIL_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
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
        RESEND_VERIFICATION_EMAIL_COOLDOWN.as_secs_f64(),
        RESEND_VERIFICATION_EMAIL_WINDOW.as_secs_f64(),
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

            let password_hash = check_and_hash_new_password(
                &mut *tx,
                state.password_minimum_length,
                &body.new_password,
                user_id,
            )
            .await?;
            store_new_password(
                &mut *tx,
                password_hash.as_str(),
                user_id,
                PasswordWrite::Reset {
                    presented_nonce: token.nonce,
                },
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
        // The same answer as every other dead link. The authorizer refuses a
        // link past its lifetime and a link minted before the nonce guard
        // shipped alike, and to whoever clicked, both are the one thing:
        // this no longer works. `Forbidden` publishes "Insufficient rights",
        // which on a reset page sends a user caught by a deployment looking for
        // a permission problem they do not have.
        Err(Hook0Problem::AuthEmailExpired)
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

    // Holding a session is not enough to take an account over: whoever asks for
    // a new password has to know the current one. Checked before the new
    // password is hashed, so a refusal does not pay the ~100ms Argon2 costs on
    // top of the one it already pays here.
    //
    // A wrong password answers exactly what an unusable session answers. A
    // problem of its own would tell a caller holding a stolen token that the
    // token itself is fine and only the password is missing.
    let current_password_hash = query_scalar!(
        r#"
            SELECT password
            FROM iam."user"
            WHERE user__id = $1
        "#,
        &token.user_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Hook0Problem::Forbidden)?;

    if !verify_password(
        current_password_hash,
        body.current_password.clone(),
        token.user_id,
    )
    .await?
    {
        return Err(Hook0Problem::Forbidden);
    }

    let password_hash = check_and_hash_new_password(
        &state.db,
        state.password_minimum_length,
        &body.new_password,
        token.user_id,
    )
    .await?;
    store_new_password(
        &state.db,
        password_hash.as_str(),
        token.user_id,
        PasswordWrite::Change,
    )
    .await?;

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

/// Which of the two ways a password can be set is being written, and what each
/// one has to present for the write to be allowed.
///
/// A type rather than a flag because the two carry different obligations: a
/// reset has a link to prove it may proceed, a change from the account settings
/// has a session and nothing to present. Writing them as one function with a
/// boolean would let a caller pass `false` and silently skip the only check
/// standing between a leaked link and a stolen account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PasswordWrite {
    /// Driven by a reset link, which carries the nonce the account had when the
    /// link was minted.
    Reset { presented_nonce: Uuid },
    /// Driven from the account settings by an authenticated user.
    Change,
}

/// Store an already checked and hashed password, and expire every token the
/// account had, so a stolen session does not survive the change.
///
/// Both paths rotate `password_reset_nonce`, which is what retires every reset
/// link outstanding for the account. On the reset path the rotation is also the
/// guard: the row is updated only while it still carries the nonce the link
/// presented, so a link works exactly once and a link superseded by a newer one
/// — or by a password changed from the settings — no longer works at all.
///
/// The guard is on the write and nowhere else on purpose. Reading the nonce
/// first and writing after would leave the ~100ms of Argon2 in between, which is
/// all the room two concurrent uses of the same link need to both pass a check
/// and both write.
async fn store_new_password<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    password_hash: &str,
    user_id: Uuid,
    write: PasswordWrite,
) -> Result<(), Hook0Problem> {
    let mut db = db.acquire().await?;
    let mut tx = db.begin().await?;

    let stored = match write {
        PasswordWrite::Reset { presented_nonce } => {
            query!(
                r#"
                    UPDATE iam."user"
                    SET password = $1,
                        password_reset_nonce = public.gen_random_uuid()
                    WHERE user__id = $2
                        AND password_reset_nonce = $3
                "#,
                password_hash,
                &user_id,
                &presented_nonce,
            )
            .execute(&mut *tx)
            .await?
        }
        PasswordWrite::Change => {
            query!(
                r#"
                    UPDATE iam."user"
                    SET password = $1,
                        password_reset_nonce = public.gen_random_uuid()
                    WHERE user__id = $2
                "#,
                password_hash,
                &user_id,
            )
            .execute(&mut *tx)
            .await?
        }
    };

    // Nothing was written. On the reset path that means the link was already
    // used, or another one has been issued since, or the password has been
    // changed in the meantime — all of which are "this link is no longer good".
    // On the change path the account is read before hashing and written after,
    // on two different connections, so this means it disappeared in between:
    // answering 204 would tell the user their password was changed when nothing
    // was stored.
    if stored.rows_affected() == 0 {
        return Err(match write {
            PasswordWrite::Reset { .. } => Hook0Problem::AuthEmailExpired,
            PasswordWrite::Change => Hook0Problem::Forbidden,
        });
    }

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
mod password_echo_tests {
    use super::{ChangePasswordPost, EmailVerificationPost, LoginPost, ResetPasswordPost};
    use crate::handlers::registrations::RegistrationPost;
    use crate::password::MAXIMUM_LENGTH as SECRET_MAX_LENGTH;
    use serde::de::DeserializeOwned;
    use validator::Validate;

    /// A password the caller must never find in the response.
    const SECRET: &str = "correct horse battery staple";

    /// The bytes the caller actually receives, built the way the API builds
    /// them: the DTO's own validation, rendered as the RFC 7807 body.
    fn response_body<T: DeserializeOwned + Validate>(body: serde_json::Value) -> String {
        let dto = serde_json::from_value::<T>(body).expect("payload deserializes into the DTO");
        let errors = dto
            .validate()
            .expect_err("payload is expected to fail validation");
        let problem = http_api_problem::HttpApiProblem::from(
            crate::problems::Hook0Problem::Validation(errors),
        );
        String::from_utf8(problem.json_bytes()).expect("problem body is valid UTF-8")
    }

    /// The `length` validator hands back the value it refused as an error
    /// parameter, and the 422 body carries the whole `ValidationErrors` tree —
    /// so validating the length of a password put that password in the
    /// response, in the browser console, and in anything logging response
    /// bodies. The policy owns the bounds instead. This pins every field that
    /// carries a password, so a validator added later cannot quietly bring the
    /// echo back.
    #[test]
    fn no_password_field_echoes_its_value_when_validation_fails() {
        // Long enough to trip any remaining length ceiling, and containing a
        // control character to trip `non_control_character` too: whichever rule
        // fires, the value must not come back.
        let refused = format!("{SECRET}\u{7}{}", "x".repeat(200));

        let bodies = [
            (
                "RegistrationPost::password",
                response_body::<RegistrationPost>(serde_json::json!({
                    "first_name": "Jordan",
                    "last_name": "Rivera",
                    "email": "jordanrivera801@example.com",
                    "password": refused,
                })),
            ),
            (
                "ResetPasswordPost::new_password",
                response_body::<ResetPasswordPost>(serde_json::json!({
                    "token": "a-reset-token",
                    "new_password": refused,
                })),
            ),
            (
                "ChangePasswordPost::new_password",
                response_body::<ChangePasswordPost>(serde_json::json!({
                    "current_password": "quilt lantern harbour",
                    "new_password": refused,
                })),
            ),
            (
                "ChangePasswordPost::current_password",
                response_body::<ChangePasswordPost>(serde_json::json!({
                    "current_password": refused,
                    "new_password": "quilt lantern harbour",
                })),
            ),
            (
                "LoginPost::password",
                response_body::<LoginPost>(serde_json::json!({
                    "email": "jordanrivera801@example.com",
                    "password": refused,
                })),
            ),
        ];

        // The other way in: a password refused for its length alone, with
        // nothing else wrong with it. Only the two fields no policy sizes
        // downstream — a login, and the current password of a change — are
        // bounded at the field, and each is bounded by a validator of its own
        // rather than by `length`, which hands the value back. `response_body`
        // fails the test outright if the field accepts this, so this pins the
        // bound as much as it pins the silence.
        let oversized = "x".repeat(SECRET_MAX_LENGTH + 1);
        let length_only = [
            (
                "ChangePasswordPost::current_password",
                response_body::<ChangePasswordPost>(serde_json::json!({
                    "current_password": oversized,
                    "new_password": "quilt lantern harbour",
                })),
            ),
            (
                "LoginPost::password",
                response_body::<LoginPost>(serde_json::json!({
                    "email": "jordanrivera801@example.com",
                    "password": oversized,
                })),
            ),
        ];

        for (field, body) in bodies {
            assert!(
                !body.contains(SECRET),
                "{field} echoed the refused password: {body}"
            );
        }

        for (field, body) in length_only {
            assert!(
                !body.contains(&oversized),
                "{field} echoed the password it refused for its length: {body}"
            );
        }
    }

    /// The token from a reset link is a credential too, and a sharper case than
    /// the password: a mail client that wraps the link and encodes the break
    /// as `%0A` produces a token that fails validation while still being live.
    /// The reset page strips it from the URL on purpose; handing it back in an
    /// error would give away exactly what that stripping withholds.
    #[test]
    fn no_token_field_echoes_its_value_when_validation_fails() {
        let refused = format!("{SECRET}\u{7}");

        let bodies = [
            (
                "ResetPasswordPost::token",
                response_body::<ResetPasswordPost>(serde_json::json!({
                    "token": refused,
                    "new_password": "quilt lantern harbour",
                })),
            ),
            (
                "EmailVerificationPost::token",
                response_body::<EmailVerificationPost>(serde_json::json!({
                    "token": refused,
                })),
            ),
        ];

        for (field, body) in bodies {
            assert!(
                !body.contains(SECRET),
                "{field} echoed the refused token: {body}"
            );
        }
    }

    /// The counterpart: only secrets lose their value. Naming the value it
    /// refused is how a validation error is useful on an ordinary field, and
    /// stripping it everywhere would be a silent downgrade of every 422.
    #[test]
    fn an_ordinary_field_still_names_the_value_it_refused() {
        let body = response_body::<RegistrationPost>(serde_json::json!({
            "first_name": "Jordan",
            "last_name": "Rivera",
            "email": "not-an-email-address",
            "password": "quilt lantern harbour",
        }));

        assert!(
            body.contains("not-an-email-address"),
            "the refused email should still be reported back: {body}"
        );
    }
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

    /// POST a change of password and return the status with the `id` of the
    /// problem it carries (empty when the response carries no problem). The
    /// body is passed whole so a test can leave a field out, which is one of
    /// the ways a caller tries to skip proving it knows the current password.
    macro_rules! change_password_with {
        ($app:expr, $token:expr, $body:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/auth/password")
                .insert_header(("Authorization", format!("Bearer {}", $token)))
                .set_json($body)
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

    /// The ordinary call: present the current password and ask for a new one.
    macro_rules! change_password {
        ($app:expr, $token:expr, $current_password:expr, $new_password:expr) => {
            change_password_with!(
                $app,
                $token,
                serde_json::json!({
                    "current_password": $current_password,
                    "new_password": $new_password,
                })
            )
        };
    }

    /// The password the seeded account holds before each test changes it.
    const CURRENT_PASSWORD: &str = "harbour quilt lantern";

    async fn stored_hash(pool: &PgPool, user_id: Uuid) -> String {
        sqlx::query_scalar("SELECT password FROM iam.user WHERE user__id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .expect("read stored password")
    }

    /// Give a seeded account a real Argon2 hash of `password`, so the endpoint
    /// can be driven the way a user drives it: by presenting the password it is
    /// about to replace.
    async fn set_password(pool: &PgPool, user_id: Uuid, password: &str) {
        let hash = crate::password::hash(crate::password::Checked::already_established(password))
            .await
            .expect("hash the current password");
        sqlx::query(r#"UPDATE iam."user" SET password = $2 WHERE user__id = $1"#)
            .bind(user_id)
            .bind(hash.as_str())
            .execute(pool)
            .await
            .expect("store the current password");
    }

    /// Whether what the account currently holds accepts `password`. Says what a
    /// login would say, without a login endpoint to mount.
    async fn stored_password_accepts(pool: &PgPool, user_id: Uuid, password: &str) -> bool {
        super::verify_password(
            stored_hash(pool, user_id).await,
            password.to_owned(),
            user_id,
        )
        .await
        .expect("verify the stored password")
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
        set_password(&pool, user, CURRENT_PASSWORD).await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let email: String = sqlx::query_scalar("SELECT email FROM iam.user WHERE user__id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .expect("read seeded email");

        let (status, problem) = change_password!(app, token, CURRENT_PASSWORD, email);

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
        set_password(&pool, user, CURRENT_PASSWORD).await;

        let app = init_api!(pool, private_key);
        let (status, problem) = change_password!(app, token, CURRENT_PASSWORD, "2026letmein!");

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
        set_password(&pool, user, CURRENT_PASSWORD).await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let (status, _) = change_password!(app, token, CURRENT_PASSWORD, "quilt lantern harbour");

        assert!(status.is_success(), "unexpected status: {status}");

        let after = stored_hash(&pool, user).await;
        assert_ne!(after, before);
        assert!(
            after.starts_with("$argon2"),
            "the stored password must be an Argon2 hash, got {after:?}"
        );
        assert!(
            stored_password_accepts(&pool, user, "quilt lantern harbour").await,
            "the account must accept the password that was just set"
        );
        assert!(
            !stored_password_accepts(&pool, user, CURRENT_PASSWORD).await,
            "the account must stop accepting the password that was replaced"
        );
    }

    /// A session alone is not enough to take an account over. Whoever asks for a
    /// new password has to know the one it replaces: without that, a token
    /// picked out of a browser — good for five minutes — buys permanent access
    /// AND locks the owner out, because setting a password signs every session
    /// out.
    ///
    /// The second half is what makes this test worth anything: a handler that
    /// refuses and writes anyway would pass on the status alone.
    #[sqlx::test]
    async fn changing_the_password_with_a_wrong_current_one_is_refused(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;
        set_password(&pool, user, CURRENT_PASSWORD).await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let (status, problem) = change_password!(
            app,
            token,
            "not the current password",
            "quilt lantern harbour"
        );

        assert_eq!(status, actix_web::http::StatusCode::FORBIDDEN);
        assert_eq!(
            problem, "Forbidden",
            "a wrong password must answer exactly what an unusable session answers: telling the two apart tells a caller holding a stolen token that only the password is missing"
        );
        assert_eq!(
            stored_hash(&pool, user).await,
            before,
            "a refused change must leave the stored password alone"
        );
        assert!(
            stored_password_accepts(&pool, user, CURRENT_PASSWORD).await,
            "the account must still accept the password it had"
        );
        assert!(
            !stored_password_accepts(&pool, user, "quilt lantern harbour").await,
            "the refused password must not have been stored"
        );
    }

    /// Leaving the field out is the same attempt made a different way, and it
    /// must not reach the write either.
    #[sqlx::test]
    async fn changing_the_password_without_presenting_the_current_one_is_refused(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;
        set_password(&pool, user, CURRENT_PASSWORD).await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let (status, _) = change_password_with!(
            app,
            token,
            serde_json::json!({ "new_password": "quilt lantern harbour" })
        );

        assert!(
            status.is_client_error(),
            "a change with no current password must be refused, got {status}"
        );
        assert_ne!(status, actix_web::http::StatusCode::NO_CONTENT);
        assert_eq!(
            stored_hash(&pool, user).await,
            before,
            "a refused change must leave the stored password alone"
        );
        assert!(
            stored_password_accepts(&pool, user, CURRENT_PASSWORD).await,
            "the account must still accept the password it had"
        );
    }

    /// An empty string is not a password anyone holds, so it can never stand in
    /// for one — including on an account whose stored hash it would be cheapest
    /// to compare against.
    #[sqlx::test]
    async fn an_empty_current_password_never_reaches_the_write(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;
        set_password(&pool, user, CURRENT_PASSWORD).await;
        let before = stored_hash(&pool, user).await;

        let app = init_api!(pool, private_key);
        let (status, _) = change_password!(app, token, "", "quilt lantern harbour");

        assert!(
            status.is_client_error(),
            "an empty current password must be refused, got {status}"
        );
        assert_eq!(
            stored_hash(&pool, user).await,
            before,
            "a refused change must leave the stored password alone"
        );
    }
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
        .bind(super::RESEND_VERIFICATION_EMAIL_COOLDOWN.as_secs_f64() * 2.0)
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
        .bind(super::RESEND_VERIFICATION_EMAIL_WINDOW.as_secs_f64() + 60.0)
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
    /// second time (no second Created, so no second session) nor re-stamps
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

        // First use: verifies the account and opens a session.
        let first = verify(&pool, &keypair.private(), &token).await;
        assert_eq!(
            first.status(),
            StatusCode::CREATED,
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
            StatusCode::CREATED,
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

#[cfg(test)]
mod auto_login_after_verify_tests {
    use super::*;
    use crate::google_ads::test_support::test_state;
    use actix_web::{App, test, web};
    use sqlx::PgPool;

    /// Insert an unverified user (`email_verified_at IS NULL`) and return its id.
    async fn seed_unverified_user(pool: &PgPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
                INSERT INTO iam."user" (user__id, email, password, first_name, last_name, email_verified_at)
                VALUES ($1, $2, 'unused-hash', 'Nina', 'Verify', NULL)
            "#,
        )
        .bind(user_id)
        .bind(format!("verify-{user_id}@example.com"))
        .execute(pool)
        .await
        .expect("seed unverified user");
        user_id
    }

    /// Full HTTP path: clicking the verification link (POST /auth/verify-email)
    /// returns a real session, and that session is immediately accepted by an
    /// authenticated endpoint — the user never re-enters credentials. Replaying
    /// the still-valid verification token yields no second session, proving the
    /// one-time guarantee is preserved. Drives the real handlers + biscuit auth
    /// middleware against real Postgres.
    #[sqlx::test]
    async fn verifying_email_opens_a_session_and_authorizes_api_without_relogin(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();
        let state = test_state(pool.clone(), private_key.clone(), None).await;

        let user_id = seed_unverified_user(&pool).await;
        let verification_token = crate::iam::create_email_verification_token(&private_key, user_id)
            .expect("create verification token")
            .serialized_biscuit;

        let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
            db: pool.clone(),
            biscuit_private_key: private_key.clone(),
            master_api_key: None,
            enable_application_secret_compatibility: true,
        };

        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/verify-email", web::post().to(super::verify_email)),
                    )
                    .service(
                        web::scope("/organizations")
                            .wrap(biscuit_auth.clone())
                            .route("", web::get().to(crate::handlers::organizations::list)),
                    ),
            ),
        )
        .await;

        // 1) Click the verification link → a session is returned (201 + tokens).
        let verify = test::TestRequest::post()
            .uri("/api/v1/auth/verify-email")
            .set_json(serde_json::json!({ "token": verification_token }))
            .to_request();
        let resp = test::call_service(&app, verify).await;
        assert_eq!(
            resp.status().as_u16(),
            201,
            "verification must open a session"
        );
        let session: serde_json::Value = test::read_body_json(resp).await;
        let access_token = session["access_token"]
            .as_str()
            .expect("access_token in verification response");
        assert!(!access_token.is_empty(), "access token must be non-empty");
        assert_eq!(
            session["user_id"].as_str(),
            Some(user_id.to_string().as_str()),
            "session belongs to the verified user"
        );

        // 2) The returned session is accepted by an authenticated endpoint right
        //    away — no re-login.
        let list = test::TestRequest::get()
            .uri("/api/v1/organizations")
            .insert_header(("Authorization", format!("Bearer {access_token}")))
            .to_request();
        let resp = test::call_service(&app, list).await;
        assert!(
            resp.status().is_success(),
            "auto-login session must authorize API calls, got {}",
            resp.status()
        );
        let orgs: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(
            orgs.as_array().map(Vec::len),
            Some(0),
            "a freshly verified account has no organization yet"
        );

        // 3) Replaying the still-valid verification token must NOT mint a second
        //    session: the one-time email_verified_at transition already happened.
        let replay = test::TestRequest::post()
            .uri("/api/v1/auth/verify-email")
            .set_json(serde_json::json!({ "token": verification_token }))
            .to_request();
        let resp = test::call_service(&app, replay).await;
        assert_ne!(
            resp.status().as_u16(),
            201,
            "a consumed verification token must never open a second session"
        );

        // 4) ...and it says why. Every second open of the link — double click,
        //    back button, a forwarded copy — lands here, and by then the address
        //    IS verified, so "the link might be expired, retry the whole
        //    process" would send the user round a loop they have already
        //    completed.
        let problem: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(
            problem["id"].as_str(),
            Some("AuthEmailAlreadyVerified"),
            "a replay must be reported as already verified, not as an expired link"
        );
    }
}

#[cfg(test)]
mod begin_reset_password_tests {
    use crate::google_ads::test_support::{DEAD_SMTP_CONNECTION_URL, test_state_with_smtp};
    use crate::mailer::test_support::FakeSmtp;
    use actix_web::{App, http::StatusCode, test, web};
    use chrono::{DateTime, Utc};
    use proptest::prelude::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
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
    /// of magnitude more than it would need.
    const NO_FURTHER_SEND_WINDOW: Duration = Duration::from_secs(3);

    /// Insert a verified account — the state a user asking for a reset link is
    /// normally in — and return its id.
    async fn seed_account(pool: &PgPool, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
                INSERT INTO iam."user" (user__id, email, password, first_name, last_name, email_verified_at)
                VALUES ($1, $2, 'unused-hash', 'Test', 'User', statement_timestamp())
            "#,
        )
        .bind(user_id)
        .bind(email)
        .execute(pool)
        .await
        .expect("seed account");
        user_id
    }

    async fn reset_sent_at(pool: &PgPool, user_id: Uuid) -> Option<DateTime<Utc>> {
        let row: (Option<DateTime<Utc>>,) =
            sqlx::query_as(r#"SELECT password_reset_sent_at FROM iam."user" WHERE user__id = $1"#)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .expect("read password_reset_sent_at");
        row.0
    }

    async fn reset_count(pool: &PgPool, user_id: Uuid) -> i32 {
        let row: (i32,) =
            sqlx::query_as(r#"SELECT password_reset_count FROM iam."user" WHERE user__id = $1"#)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .expect("read password_reset_count");
        row.0
    }

    async fn reset_window_started_at(pool: &PgPool, user_id: Uuid) -> Option<DateTime<Utc>> {
        let row: (Option<DateTime<Utc>>,) = sqlx::query_as(
            r#"SELECT password_reset_window_started_at FROM iam."user" WHERE user__id = $1"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("read password_reset_window_started_at");
        row.0
    }

    /// A release must not bring a retired link back to life.
    ///
    /// The allowance is spent before the mail is handed to SMTP, so a send that
    /// fails gives it back — and puts the previous nonce back with it, because
    /// the link the user is holding is the one that claim superseded. That is
    /// right only while nothing else has moved the nonce in between. Changing
    /// the password from the settings page rotates it and leaves the send stamp
    /// untouched, so a release guarded on the stamp alone still matches, and the
    /// row goes back to a nonce whose link is still inside its lifetime. The
    /// account holder would have acted precisely to kill that link.
    #[sqlx::test]
    async fn a_release_does_not_revive_a_link_a_password_write_retired(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("release-after-change-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;

        // A link goes out. This is the one that must stay dead.
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        let mailed_nonce = reset_nonce(&pool, user_id).await;

        // A second request claims the allowance and supersedes that link.
        expire_cooldown(&pool, user_id).await;
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        let claim = super::ResetPasswordClaim {
            user_id,
            email: email.clone(),
            first_name: "Test".to_owned(),
            last_name: "User".to_owned(),
            nonce: reset_nonce(&pool, user_id).await,
            claimed_at: reset_sent_at(&pool, user_id)
                .await
                .expect("the claim stamped the send"),
            previous_nonce: mailed_nonce,
            previous_sent_at: None,
            previous_window_started_at: None,
            previous_count: 0,
        };

        // Before that send is given back, the account holder changes their
        // password, which rotates the nonce and retires every link outstanding.
        sqlx::query(
            r#"UPDATE iam."user" SET password_reset_nonce = public.gen_random_uuid() WHERE user__id = $1"#,
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("a password write rotates the nonce");
        let after_password_write = reset_nonce(&pool, user_id).await;

        super::release_reset_password_claim(&pool, &claim)
            .await
            .expect("release the claim");

        assert_eq!(
            reset_nonce(&pool, user_id).await,
            after_password_write,
            "a release must leave a nonce a password write has moved alone"
        );
        assert_ne!(
            reset_nonce(&pool, user_id).await,
            mailed_nonce,
            "the link that was mailed must not work again after the password changed"
        );
    }

    async fn reset_nonce(pool: &PgPool, user_id: Uuid) -> Uuid {
        let row: (Uuid,) =
            sqlx::query_as(r#"SELECT password_reset_nonce FROM iam."user" WHERE user__id = $1"#)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .expect("read password_reset_nonce");
        row.0
    }

    /// Age the last-send stamp out of its cooldown, leaving the window counter
    /// untouched, so the only control left standing is the per-window cap.
    async fn expire_cooldown(pool: &PgPool, user_id: Uuid) {
        sqlx::query(
            r#"
                UPDATE iam."user"
                SET password_reset_sent_at = statement_timestamp() - MAKE_INTERVAL(secs => $2)
                WHERE user__id = $1
            "#,
        )
        .bind(user_id)
        .bind(super::BEGIN_RESET_PASSWORD_COOLDOWN.as_secs_f64() * 2.0)
        .execute(pool)
        .await
        .expect("age the cooldown out");
    }

    /// Age the counting window past its length, as if a day had gone by.
    async fn expire_window(pool: &PgPool, user_id: Uuid) {
        sqlx::query(
            r#"
                UPDATE iam."user"
                SET password_reset_window_started_at =
                    statement_timestamp() - MAKE_INTERVAL(secs => $2)
                WHERE user__id = $1
            "#,
        )
        .bind(user_id)
        .bind(super::BEGIN_RESET_PASSWORD_WINDOW.as_secs_f64() + 60.0)
        .execute(pool)
        .await
        .expect("age the counting window out");
    }

    /// Wait, bounded, until the detached task has given a failed attempt back.
    /// Returns whether that happened before the timeout.
    async fn wait_for_released_claim(
        pool: &PgPool,
        user_id: Uuid,
        expected_sent_at: Option<DateTime<Utc>>,
        expected_count: i32,
    ) -> bool {
        let steps = DETACHED_WORK_TIMEOUT.as_millis() / 25;
        for _ in 0..steps {
            if reset_sent_at(pool, user_id).await == expected_sent_at
                && reset_count(pool, user_id).await == expected_count
            {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        false
    }

    /// Build a test service exposing only the begin-reset endpoint against
    /// `pool`, talking to the SMTP endpoint at `smtp_connection_url`, and POST
    /// `email` to it.
    async fn begin_reset_via(
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
                        web::resource("/begin-reset-password")
                            .route(web::post().to(super::begin_reset_password)),
                    ),
                ),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/begin-reset-password")
            .set_json(serde_json::json!({ "email": email }))
            .to_request();
        test::call_service(&app, req).await
    }

    /// The defect the report named: the endpoint answered 204 for an address it
    /// knew and 401 for one it did not, which turns the login page into an
    /// account-existence oracle anybody can read without tooling. Status AND
    /// body have to be identical — a body that differed would be the same leak
    /// one layer down.
    #[sqlx::test]
    async fn begin_reset_response_is_identical_for_unknown_and_known_email(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let known = format!("known-{}@example.com", Uuid::new_v4());
        seed_account(&pool, &known).await;
        let unknown = format!("nobody-{}@example.com", Uuid::new_v4());

        let known_resp = begin_reset_via(&pool, &smtp.connection_url, &known).await;
        let known_status = known_resp.status();
        let known_body = test::read_body(known_resp).await;

        let unknown_resp = begin_reset_via(&pool, &smtp.connection_url, &unknown).await;
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

    /// An address that matches nothing costs nothing: no mail, and no row to
    /// stamp.
    #[sqlx::test]
    async fn begin_reset_for_unknown_email_sends_nothing(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let unknown = format!("nobody-{}@example.com", Uuid::new_v4());

        let resp = begin_reset_via(&pool, &smtp.connection_url, &unknown).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        assert_eq!(
            smtp.wait_for(1, NO_FURTHER_SEND_WINDOW).await,
            0,
            "an address that matches no account is never mailed"
        );
    }

    /// A mail server that is down used to answer 500 for a known address and 401
    /// for an unknown one, which is the same oracle with an extra state. The
    /// failure is logged and swallowed.
    #[sqlx::test]
    async fn begin_reset_answers_no_content_when_smtp_is_down(pool: PgPool) {
        let email = format!("smtp-down-{}@example.com", Uuid::new_v4());
        seed_account(&pool, &email).await;

        let known = begin_reset_via(&pool, DEAD_SMTP_CONNECTION_URL, &email).await;
        assert_eq!(
            known.status(),
            StatusCode::NO_CONTENT,
            "a failing send must never surface as a 500: that is an existence oracle"
        );

        let unknown = begin_reset_via(
            &pool,
            DEAD_SMTP_CONNECTION_URL,
            &format!("nobody-{}@example.com", Uuid::new_v4()),
        )
        .await;
        assert_eq!(unknown.status(), known.status());
    }

    /// A request that sends nothing must not rotate the nonce either: the link
    /// already in the user's mailbox is the last one they were given, and an
    /// address they do not control must not be able to retire it.
    ///
    /// Both ways of sending nothing are walked, because they fail differently.
    /// A call the quota refuses never rotates anything — no row matches. A call
    /// the quota lets through rotates first and only then discovers the mail
    /// cannot leave, so the link is already retired by the time anyone knows;
    /// only putting the nonce back makes that call indistinguishable from the
    /// refused one. Without it, a mail server that is down — or merely
    /// greylisting — is all it takes to retire a stranger's live link on demand.
    #[sqlx::test]
    async fn begin_reset_rotates_the_nonce_only_when_it_sends(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("nonce-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;

        let before = reset_nonce(&pool, user_id).await;
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        let after_send = reset_nonce(&pool, user_id).await;
        assert_ne!(
            before, after_send,
            "issuing a link must retire whatever came before it"
        );

        // Straight back, inside the cooldown: nothing is sent, so nothing is
        // retired.
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            reset_nonce(&pool, user_id).await,
            after_send,
            "a request that sends no link must not retire the one that was sent"
        );

        // And now with the quota out of the way, so the claim is granted and
        // the send is what fails. The stamps the call took are given back; the
        // link the user is holding has to come back with them.
        expire_cooldown(&pool, user_id).await;
        let sent_at = reset_sent_at(&pool, user_id).await;
        let count = reset_count(&pool, user_id).await;
        assert_eq!(
            begin_reset_via(&pool, DEAD_SMTP_CONNECTION_URL, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert!(
            wait_for_released_claim(&pool, user_id, sent_at, count).await,
            "a send that never happened must give its attempt back"
        );
        assert_eq!(
            reset_nonce(&pool, user_id).await,
            after_send,
            "a mail that never left must not retire the link the user already has"
        );
    }

    /// A second request within the cooldown answers the same NoContent but does
    /// not send again.
    #[sqlx::test]
    async fn begin_reset_is_rate_limited_per_email(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("throttled-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;

        let first = begin_reset_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(first.status(), StatusCode::NO_CONTENT);
        assert_eq!(smtp.wait_for(1, DETACHED_WORK_TIMEOUT).await, 1);
        let first_sent = reset_sent_at(&pool, user_id)
            .await
            .expect("the first request stamps the send time");

        let second = begin_reset_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(second.status(), StatusCode::NO_CONTENT);

        assert_eq!(
            reset_sent_at(&pool, user_id).await,
            Some(first_sent),
            "a request within the cooldown must not send again"
        );
        assert_eq!(
            smtp.wait_for(2, NO_FURTHER_SEND_WINDOW).await,
            1,
            "a request within the cooldown must not put a second mail on the wire"
        );
    }

    /// The cooldown only spaces sends out; the cap bounds their total. That is
    /// what stands between one mailbox and a flood, because a caller that
    /// rotates source addresses meets nothing else.
    #[sqlx::test]
    async fn begin_reset_is_capped_per_account_within_one_window(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("capped-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;

        for expected in 1..=super::BEGIN_RESET_PASSWORD_MAX_PER_WINDOW {
            let resp = begin_reset_via(&pool, &smtp.connection_url, &email).await;
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
            assert_eq!(
                reset_count(&pool, user_id).await,
                expected,
                "each allowed send must move the window counter along"
            );
            let delivered = usize::try_from(expected).expect("small positive count");
            assert_eq!(
                smtp.wait_for(delivered, DETACHED_WORK_TIMEOUT).await,
                delivered
            );
            expire_cooldown(&pool, user_id).await;
        }

        let sent_before_capped_attempt = reset_sent_at(&pool, user_id)
            .await
            .expect("the allowed sends stamped a send time");
        let allowance =
            usize::try_from(super::BEGIN_RESET_PASSWORD_MAX_PER_WINDOW).expect("small cap");

        let capped = begin_reset_via(&pool, &smtp.connection_url, &email).await;
        assert_eq!(
            capped.status(),
            StatusCode::NO_CONTENT,
            "reaching the cap must be invisible to the caller (anti-enumeration)"
        );
        assert_eq!(
            reset_sent_at(&pool, user_id).await,
            Some(sent_before_capped_attempt),
            "a capped request must not send, so it must not stamp a new send time"
        );
        assert_eq!(
            reset_count(&pool, user_id).await,
            super::BEGIN_RESET_PASSWORD_MAX_PER_WINDOW,
            "a capped request must not consume more allowance either"
        );
        assert_eq!(
            smtp.wait_for(allowance + 1, NO_FURTHER_SEND_WINDOW).await,
            allowance,
            "the cap is what bounds the mail a mailbox receives, so nothing more leaves"
        );
    }

    /// The cap is a bound per window, not a permanent lock-out — and this is a
    /// recovery path, so a user locked out of it for good would be locked out of
    /// their account.
    #[sqlx::test]
    async fn begin_reset_cap_lifts_once_the_window_has_elapsed(pool: PgPool) {
        let smtp = FakeSmtp::start();
        let email = format!("window-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;

        for _ in 0..super::BEGIN_RESET_PASSWORD_MAX_PER_WINDOW {
            assert_eq!(
                begin_reset_via(&pool, &smtp.connection_url, &email)
                    .await
                    .status(),
                StatusCode::NO_CONTENT
            );
            expire_cooldown(&pool, user_id).await;
        }
        let sent_while_capped = reset_sent_at(&pool, user_id)
            .await
            .expect("the allowed sends stamped a send time");
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            reset_sent_at(&pool, user_id).await,
            Some(sent_while_capped),
            "precondition: the account is capped"
        );

        expire_window(&pool, user_id).await;

        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
                .await
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_ne!(
            reset_sent_at(&pool, user_id).await,
            Some(sent_while_capped),
            "a new window must let the account be sent to again"
        );
        assert_eq!(
            reset_count(&pool, user_id).await,
            1,
            "the counter restarts with the new window rather than carrying over"
        );
    }

    /// An attempt that never became a mail is given back: the quota bounds what
    /// reaches a mailbox, not what was tried.
    #[sqlx::test]
    async fn begin_reset_gives_the_attempt_back_when_the_mail_cannot_be_sent(pool: PgPool) {
        let email = format!("release-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;

        let resp = begin_reset_via(&pool, DEAD_SMTP_CONNECTION_URL, &email).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        assert!(
            wait_for_released_claim(&pool, user_id, None, 0).await,
            "a send that never happened must give its attempt back"
        );
        assert!(
            reset_window_started_at(&pool, user_id).await.is_none(),
            "the window anchor goes back too, so the released attempt opened no window"
        );

        let smtp = FakeSmtp::start();
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
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

    /// The defect this guards against: an SMTP outage spending the whole daily
    /// allowance in minutes while the user received nothing, locking them out of
    /// account recovery for a day.
    ///
    /// The same sequence is what an attacker runs on purpose. Because every
    /// attempt is given back, nothing stops them repeating it — so the link the
    /// account is holding has to survive all of it. Asserted at every step: a
    /// nonce restored on the first release but not the sixth would leave the
    /// hole open behind a longer loop.
    #[sqlx::test]
    async fn an_smtp_outage_does_not_burn_the_daily_allowance(pool: PgPool) {
        let email = format!("outage-{}@example.com", Uuid::new_v4());
        let user_id = seed_account(&pool, &email).await;
        let pending_link = reset_nonce(&pool, user_id).await;

        for _ in 0..=super::BEGIN_RESET_PASSWORD_MAX_PER_WINDOW {
            assert_eq!(
                begin_reset_via(&pool, DEAD_SMTP_CONNECTION_URL, &email)
                    .await
                    .status(),
                StatusCode::NO_CONTENT
            );
            assert!(
                wait_for_released_claim(&pool, user_id, None, 0).await,
                "every attempt that sent nothing must be given back"
            );
            assert_eq!(
                reset_nonce(&pool, user_id).await,
                pending_link,
                "a caller who cannot receive the mail must not be able to retire the link that was already delivered"
            );
        }

        let smtp = FakeSmtp::start();
        assert_eq!(
            begin_reset_via(&pool, &smtp.connection_url, &email)
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

    /// How many call sequences the property below explores, and how long each
    /// one may be. Bounded because every step is a real request against a real
    /// database and a real SMTP server: the point is to cover orderings the
    /// hand-written tests above do not, not to run for minutes.
    const EXPLORED_SEQUENCES: usize = 6;
    const LONGEST_SEQUENCE: usize = 8;

    /// How long a sequence waits, once it is done, before concluding that the
    /// calls it made sent nothing more. Short because every permitted call has
    /// already been waited for exactly, so this only has to catch a stray from
    /// the last refused one.
    const NO_STRAY_SEND_WINDOW: Duration = Duration::from_secs(1);

    /// The invariant the cap exists for, over sequences rather than the one
    /// ordering a hand-written test happens to walk: whatever mix of calls and
    /// waited-out cooldowns a caller puts together, a mailbox never receives
    /// more than the allowance for its window, and what the counter records is
    /// exactly what left.
    ///
    /// The hand-written test above walks the sequence the implementation was
    /// written against — allowance sends, each after its cooldown, then one
    /// more. This one walks orderings nobody chose: bunched calls inside a
    /// cooldown, cooldowns aged out with no call in between, a cooldown expiring
    /// before the first call. Each of those is a chance for the counter and the
    /// send stamp to be moved by different branches of the claim and disagree.
    ///
    /// proptest drives the generation; the assertions cannot live inside
    /// `proptest!` because every step of them is `async`.
    #[sqlx::test]
    async fn the_cap_holds_whatever_the_order_of_calls(pool: PgPool) {
        let allowance =
            usize::try_from(super::BEGIN_RESET_PASSWORD_MAX_PER_WINDOW).expect("small cap");
        let plans = proptest::collection::vec(any::<bool>(), 1..=LONGEST_SEQUENCE);
        let mut runner = TestRunner::deterministic();

        for _ in 0..EXPLORED_SEQUENCES {
            // `true` asks for a link, `false` waits the cooldown out. The window
            // is never aged out, so the counter only ever moves forward and a
            // call that moved it is a call that was permitted.
            let mut plan = plans
                .new_tree(&mut runner)
                .expect("generate a call sequence")
                .current();

            // Then, whatever state the generated part left the account in, ask
            // more times than the allowance permits with the cooldown out of the
            // way every time. Without this the generator decides whether a
            // sequence ever reaches the cap, and a property that only sometimes
            // reaches the case it is about only sometimes fails when that case
            // breaks.
            for _ in 0..=allowance {
                plan.push(false);
                plan.push(true);
            }

            let smtp = FakeSmtp::start();
            let email = format!("ordering-{}@example.com", Uuid::new_v4());
            let user_id = seed_account(&pool, &email).await;

            let mut permitted = 0usize;
            for ask_for_a_link in &plan {
                if !ask_for_a_link {
                    expire_cooldown(&pool, user_id).await;
                    continue;
                }

                let before = reset_count(&pool, user_id).await;
                assert_eq!(
                    begin_reset_via(&pool, &smtp.connection_url, &email)
                        .await
                        .status(),
                    StatusCode::NO_CONTENT,
                    "every call answers the same way, whatever the quota decided, for {plan:?}"
                );
                let after = reset_count(&pool, user_id).await;

                if after != before {
                    permitted += 1;
                    assert_eq!(
                        smtp.wait_for(permitted, DETACHED_WORK_TIMEOUT).await,
                        permitted,
                        "a permitted call must put exactly one mail on the wire, for {plan:?}"
                    );
                }
            }

            assert_eq!(
                smtp.wait_for(permitted + 1, NO_STRAY_SEND_WINDOW).await,
                permitted,
                "a call the quota refused must send nothing, for {plan:?}"
            );
            assert!(
                permitted <= allowance,
                "{permitted} mails reached the mailbox where the allowance is {allowance}, for {plan:?}"
            );
            assert_eq!(
                reset_count(&pool, user_id).await,
                i32::try_from(permitted).expect("small count"),
                "the counter must record exactly what left, for {plan:?}"
            );
        }
    }
}

#[cfg(test)]
mod password_reset_link_tests {
    use crate::google_ads::test_support::{issue_user_token, seed_org, test_state_with_smtp};
    use crate::mailer::test_support::FakeSmtp;
    use actix_web::{App, http::StatusCode, test, web};
    use proptest::prelude::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    use sqlx::PgPool;
    use uuid::Uuid;

    /// The password the account holds before any test touches it.
    const ORIGINAL_PASSWORD: &str = "harbour quilt lantern";

    /// Spin up the real reset, change and begin-reset endpoints — the last two
    /// behind the real biscuit auth middleware where they belong — over the test
    /// database and a real SMTP server. A macro rather than a function because
    /// the type of an initialized actix test service is not nameable here.
    macro_rules! init_api {
        ($pool:expr, $private_key:expr, $smtp_connection_url:expr) => {{
            let state = test_state_with_smtp(
                $pool.clone(),
                $private_key.clone(),
                None,
                $smtp_connection_url,
            )
            .await;
            let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
                db: $pool.clone(),
                biscuit_private_key: $private_key.clone(),
                master_api_key: None,
                enable_application_secret_compatibility: true,
            };

            test::init_service(
                App::new().app_data(web::Data::new(state)).service(
                    web::scope("/api/v1/auth")
                        .service(
                            web::resource("/begin-reset-password")
                                .route(web::post().to(super::begin_reset_password)),
                        )
                        .service(
                            web::resource("/reset-password")
                                .route(web::post().to(super::reset_password)),
                        )
                        .service(
                            web::resource("/password")
                                .wrap(biscuit_auth)
                                .route(web::post().to(super::change_password)),
                        ),
                ),
            )
            .await
        }};
    }

    /// POST a reset link with a new password and return the status.
    macro_rules! reset_password {
        ($app:expr, $token:expr, $new_password:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/auth/reset-password")
                .set_json(serde_json::json!({
                    "token": $token,
                    "new_password": $new_password,
                }))
                .to_request();
            test::call_service(&$app, request).await.status()
        }};
    }

    /// Insert a verified account holding `ORIGINAL_PASSWORD`, hashed the way a
    /// registration hashes it, and return its id.
    async fn seed_account(pool: &PgPool) -> Uuid {
        let user_id = Uuid::new_v4();
        let hash = crate::password::hash(crate::password::Checked::already_established(
            ORIGINAL_PASSWORD,
        ))
        .await
        .expect("hash the original password");

        sqlx::query(
            r#"
                INSERT INTO iam."user" (user__id, email, password, first_name, last_name, email_verified_at)
                VALUES ($1, $2, $3, 'Test', 'User', statement_timestamp())
            "#,
        )
        .bind(user_id)
        .bind(format!("reset-{user_id}@example.com"))
        .bind(hash.as_str())
        .execute(pool)
        .await
        .expect("seed account");
        user_id
    }

    async fn account_email(pool: &PgPool, user_id: Uuid) -> String {
        sqlx::query_scalar(r#"SELECT email FROM iam."user" WHERE user__id = $1"#)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .expect("read the account email")
    }

    /// Whether what the account currently holds accepts `password`. Says what a
    /// login would say, without a login endpoint to mount — and it is the half
    /// that matters: a status alone cannot tell a refusal that wrote nothing
    /// from a refusal that wrote anyway.
    async fn stored_password_accepts(pool: &PgPool, user_id: Uuid, password: &str) -> bool {
        let hash: String =
            sqlx::query_scalar(r#"SELECT password FROM iam."user" WHERE user__id = $1"#)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .expect("read the stored password");
        super::verify_password(hash, password.to_owned(), user_id)
            .await
            .expect("verify the stored password")
    }

    /// Mint the link the server would mail right now: the account's current
    /// nonce, signed the way `begin_reset_password` signs it. Lets a test hold a
    /// link without reading a mailbox.
    async fn link_for(
        pool: &PgPool,
        private_key: &biscuit_auth::PrivateKey,
        user_id: Uuid,
    ) -> String {
        let nonce: Uuid = sqlx::query_scalar(
            r#"SELECT password_reset_nonce FROM iam."user" WHERE user__id = $1"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("read the current nonce");

        crate::iam::create_reset_password_token(private_key, user_id, nonce)
            .expect("mint a reset link")
            .serialized_biscuit
    }

    /// Put the account back inside the window but out of its cooldown, so a test
    /// can ask for several links in a row without waiting a minute between them.
    async fn expire_cooldown(pool: &PgPool, user_id: Uuid) {
        sqlx::query(
            r#"
                UPDATE iam."user"
                SET password_reset_sent_at = statement_timestamp() - MAKE_INTERVAL(secs => $2)
                WHERE user__id = $1
            "#,
        )
        .bind(user_id)
        .bind(super::BEGIN_RESET_PASSWORD_COOLDOWN.as_secs_f64() * 2.0)
        .execute(pool)
        .await
        .expect("age the cooldown out");
    }

    /// The report: the same link reset the password as many times as it was
    /// posted, for as long as it was valid. Anyone who read the mail once —
    /// a forwarded copy, a shared mailbox, a proxy log — could come back later
    /// and take the account over, and because setting a password signs every
    /// session out, they would evict the owner while doing it.
    ///
    /// The status alone proves nothing: what makes this test worth having is the
    /// second half, where the password that answered 204 is the one the account
    /// still holds.
    #[sqlx::test]
    async fn a_reset_link_stops_working_once_it_has_been_used(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();
        let smtp = FakeSmtp::start();

        let user_id = seed_account(&pool).await;
        let link = link_for(&pool, &private_key, user_id).await;
        let app = init_api!(pool, private_key, &smtp.connection_url);

        assert_eq!(
            reset_password!(app, link, "first quilt harbour"),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            reset_password!(app, link, "second quilt harbour"),
            StatusCode::UNAUTHORIZED,
            "a link that has already set a password must be refused"
        );

        assert!(
            stored_password_accepts(&pool, user_id, "first quilt harbour").await,
            "the account holds the password the accepted call set"
        );
        assert!(
            !stored_password_accepts(&pool, user_id, "second quilt harbour").await,
            "the replayed call must not have written anything"
        );
        assert!(
            !stored_password_accepts(&pool, user_id, ORIGINAL_PASSWORD).await,
            "the first call really did replace the password"
        );
    }

    /// Links minted before the nonce guard shipped are refused — that is what
    /// the version bump is for — but the refusal has to read as a dead link.
    /// Whoever clicks one is a user whose reset mail left minutes before a
    /// deployment; answering "Insufficient rights" sends them looking for an
    /// account problem they do not have, on the one page that exists because
    /// they cannot get into their account.
    #[sqlx::test]
    async fn a_link_from_before_the_nonce_guard_reads_as_a_dead_link(pool: PgPool) {
        use std::time::{Duration, SystemTime};

        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();
        let smtp = FakeSmtp::start();

        let user_id = seed_account(&pool).await;
        let created_at = SystemTime::now();
        let expired_at = created_at + Duration::from_secs(60 * 30);
        let superseded = biscuit_auth::macros::biscuit!(
            r#"
                type("password_reset");
                version(1);
                user_id({user_id});
                created_at({created_at});
                expired_at({expired_at});
            "#,
        )
        .build(&biscuit_auth::KeyPair::from(&private_key))
        .expect("mint a link of the superseded version")
        .to_base64()
        .expect("serialize the superseded link");

        let app = init_api!(pool, private_key, &smtp.connection_url);
        let request = test::TestRequest::post()
            .uri("/api/v1/auth/reset-password")
            .set_json(serde_json::json!({
                "token": superseded,
                "new_password": "quilt harbour lantern",
            }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "a link the authorizer refuses must answer like every other dead link"
        );
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(response).await).expect("a problem document");
        assert_eq!(
            body.get("id").and_then(|id| id.as_str()),
            Some("AuthEmailExpired"),
            "the reader must be told their link is dead, not that they lack rights: {body}"
        );

        assert!(
            stored_password_accepts(&pool, user_id, ORIGINAL_PASSWORD).await,
            "a refused link must not have written anything"
        );
    }

    /// Asking for another link used to leave every previous one alive until its
    /// own expiry, so a user who suspected the first mail had been read gained
    /// nothing by asking again. Issuing retires.
    #[sqlx::test]
    async fn issuing_a_new_link_retires_the_previous_one(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();
        let smtp = FakeSmtp::start();

        let user_id = seed_account(&pool).await;
        let email = account_email(&pool, user_id).await;
        let app = init_api!(pool, private_key, &smtp.connection_url);

        let first_link = link_for(&pool, &private_key, user_id).await;

        let ask_again = test::TestRequest::post()
            .uri("/api/v1/auth/begin-reset-password")
            .set_json(serde_json::json!({ "email": email }))
            .to_request();
        assert_eq!(
            test::call_service(&app, ask_again).await.status(),
            StatusCode::NO_CONTENT
        );
        let second_link = link_for(&pool, &private_key, user_id).await;
        assert_ne!(
            first_link, second_link,
            "precondition: asking again mints a different link"
        );

        assert_eq!(
            reset_password!(app, first_link, "stale quilt harbour"),
            StatusCode::UNAUTHORIZED,
            "the superseded link must be refused"
        );
        assert!(
            stored_password_accepts(&pool, user_id, ORIGINAL_PASSWORD).await,
            "the refused link must not have written anything"
        );

        // The guard against over-correcting: the link the user actually holds
        // still works.
        assert_eq!(
            reset_password!(app, second_link, "fresh quilt harbour"),
            StatusCode::NO_CONTENT
        );
        assert!(
            stored_password_accepts(&pool, user_id, "fresh quilt harbour").await,
            "the newest link must still be able to set the password"
        );
    }

    /// The counter-measure neither report asked for: a user who suspects a reset
    /// mail has been read has one thing they can do from inside their account,
    /// and it has to work. Changing the password from the settings retires every
    /// link outstanding for the account.
    #[sqlx::test]
    async fn changing_the_password_retires_a_pending_link(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();
        let smtp = FakeSmtp::start();

        let user_id = seed_account(&pool).await;
        let org = seed_org(&pool, user_id).await;
        let session = issue_user_token(&pool, &private_key, user_id, org, "editor").await;
        let link = link_for(&pool, &private_key, user_id).await;
        let app = init_api!(pool, private_key, &smtp.connection_url);

        let change = test::TestRequest::post()
            .uri("/api/v1/auth/password")
            .insert_header(("Authorization", format!("Bearer {session}")))
            .set_json(serde_json::json!({
                "current_password": ORIGINAL_PASSWORD,
                "new_password": "settings quilt harbour",
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, change).await.status(),
            StatusCode::NO_CONTENT
        );

        assert_eq!(
            reset_password!(app, link, "leaked quilt harbour"),
            StatusCode::UNAUTHORIZED,
            "a link outstanding when the password was changed must be refused"
        );
        assert!(
            stored_password_accepts(&pool, user_id, "settings quilt harbour").await,
            "the account holds the password its owner set"
        );
        assert!(
            !stored_password_accepts(&pool, user_id, "leaked quilt harbour").await,
            "the refused link must not have written anything"
        );
    }

    /// The case that tells a guard on the write from a guard on a read before
    /// it. Checking the nonce, then hashing, then writing leaves ~100ms of
    /// deliberate Argon2 in between — all the room two posts of the same link
    /// need to both pass the check and both write, with the second silently
    /// overwriting the first.
    ///
    /// Exactly one may answer 204, and the password the account ends up with has
    /// to be the one that call carried.
    #[sqlx::test]
    async fn two_concurrent_uses_of_the_same_link_set_exactly_one_password(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();
        let smtp = FakeSmtp::start();

        let user_id = seed_account(&pool).await;
        let link = link_for(&pool, &private_key, user_id).await;
        let app = init_api!(pool, private_key, &smtp.connection_url);

        let post = |new_password: &'static str| {
            test::TestRequest::post()
                .uri("/api/v1/auth/reset-password")
                .set_json(serde_json::json!({
                    "token": link.clone(),
                    "new_password": new_password,
                }))
                .to_request()
        };

        let (left, right) = tokio::join!(
            test::call_service(&app, post("racing quilt harbour")),
            test::call_service(&app, post("racing lantern harbour")),
        );
        let accepted = [
            ("racing quilt harbour", left.status()),
            ("racing lantern harbour", right.status()),
        ]
        .into_iter()
        .filter(|(_, status)| *status == StatusCode::NO_CONTENT)
        .map(|(password, _)| password)
        .collect::<Vec<_>>();

        assert_eq!(
            accepted.len(),
            1,
            "exactly one of two concurrent uses of the same link may be accepted, got {:?} and {:?}",
            left.status(),
            right.status()
        );

        let winner = accepted[0];
        let loser = if winner == "racing quilt harbour" {
            "racing lantern harbour"
        } else {
            "racing quilt harbour"
        };
        assert!(
            stored_password_accepts(&pool, user_id, winner).await,
            "the account must hold the password of the call that answered 204"
        );
        assert!(
            !stored_password_accepts(&pool, user_id, loser).await,
            "the refused call must not have written underneath the accepted one"
        );
    }

    /// How many sequences the property below explores, and how long each may be.
    /// Bounded because every reset pays the ~100ms of Argon2 the policy asks
    /// for, whether it is accepted or not.
    const EXPLORED_SEQUENCES: usize = 5;
    const LONGEST_SEQUENCE: usize = 6;

    /// One link, one password: whatever order links are minted, used and
    /// superseded in, no link is ever accepted twice.
    ///
    /// The hand-written tests above each pin one ordering. This one walks
    /// orderings nobody chose — an old link used after two newer ones exist, a
    /// link used again after a settings change, two links minted before either
    /// is used — because that is where a guard that holds for the sequence it
    /// was written against stops holding.
    ///
    /// Every link the sequence got accepted is then replayed, rather than left
    /// to the generator to happen to pick twice: a property that only sometimes
    /// reaches the case it is about is a property that only sometimes fails when
    /// the case breaks.
    ///
    /// proptest drives the generation; the assertions cannot live inside
    /// `proptest!` because every step of them is `async`.
    #[sqlx::test]
    async fn no_reset_link_is_ever_accepted_twice(pool: PgPool) {
        // 0 mints a link, 1..=3 use one, 4 changes the password from the
        // settings. Weighted towards using links, because a sequence that only
        // mints them exercises nothing.
        let plans = proptest::collection::vec((0u8..5, any::<u8>()), 1..=LONGEST_SEQUENCE);
        let mut runner = TestRunner::deterministic();

        for sequence in 0..EXPLORED_SEQUENCES {
            let plan = plans
                .new_tree(&mut runner)
                .expect("generate a sequence of steps")
                .current();

            let keypair = biscuit_auth::KeyPair::new();
            let private_key = keypair.private();
            let smtp = FakeSmtp::start();
            let user_id = seed_account(&pool).await;
            let email = account_email(&pool, user_id).await;
            let org = seed_org(&pool, user_id).await;
            let app = init_api!(pool, private_key, &smtp.connection_url);

            let mut links = vec![link_for(&pool, &private_key, user_id).await];
            let mut accepted: Vec<String> = Vec::new();
            let mut current_password = ORIGINAL_PASSWORD.to_owned();

            for (step, (action, pick)) in plan.iter().enumerate() {
                let fresh_password = format!("quilt harbour {sequence} {step}");
                match action {
                    0 => {
                        expire_cooldown(&pool, user_id).await;
                        let ask = test::TestRequest::post()
                            .uri("/api/v1/auth/begin-reset-password")
                            .set_json(serde_json::json!({ "email": email }))
                            .to_request();
                        assert_eq!(
                            test::call_service(&app, ask).await.status(),
                            StatusCode::NO_CONTENT
                        );
                        links.push(link_for(&pool, &private_key, user_id).await);
                    }
                    4 => {
                        // Every accepted write signs the account out, so the
                        // session is minted fresh right before it is used.
                        let session =
                            issue_user_token(&pool, &private_key, user_id, org, "editor").await;
                        let change = test::TestRequest::post()
                            .uri("/api/v1/auth/password")
                            .insert_header(("Authorization", format!("Bearer {session}")))
                            .set_json(serde_json::json!({
                                "current_password": current_password,
                                "new_password": fresh_password,
                            }))
                            .to_request();
                        if test::call_service(&app, change).await.status() == StatusCode::NO_CONTENT
                        {
                            current_password = fresh_password;
                        }
                    }
                    _ => {
                        let link = links[usize::from(*pick) % links.len()].clone();
                        if reset_password!(app, link, fresh_password) == StatusCode::NO_CONTENT {
                            assert!(
                                !accepted.contains(&link),
                                "a link that had already set a password set another one, at step {step} of {plan:?}"
                            );
                            accepted.push(link);
                            current_password = fresh_password;
                        }
                    }
                }
            }

            assert!(
                stored_password_accepts(&pool, user_id, &current_password).await,
                "the account must hold the password of the last write that was accepted, after {plan:?}"
            );

            for (replay, link) in accepted.iter().enumerate() {
                let refused_password = format!("replayed quilt harbour {sequence} {replay}");
                assert_eq!(
                    reset_password!(app, link, refused_password),
                    StatusCode::UNAUTHORIZED,
                    "a link that has already set a password must never set another one, after {plan:?}"
                );
                assert!(
                    !stored_password_accepts(&pool, user_id, &refused_password).await,
                    "a replayed link wrote a password after being refused, after {plan:?}"
                );
            }

            assert!(
                stored_password_accepts(&pool, user_id, &current_password).await,
                "replaying every used link must leave the account exactly as it was, after {plan:?}"
            );
        }
    }
}
