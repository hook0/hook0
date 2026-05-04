use actix_web::rt::task::spawn_blocking;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use lettre::Address;
use lettre::message::Mailbox;
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{Apiv2Schema, CreatedJson, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query;
use std::str::FromStr;
use tracing::{error, warn};
use uuid::Uuid;
use validator::Validate;

use crate::extractor_user_ip::UserIp;
use crate::iam::{Role, create_email_verification_token};
use crate::mailer::Mail;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Registration {
    organization_id: Uuid,
    user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RegistrationPost {
    #[validate(non_control_character, length(min = 1, max = 50))]
    first_name: String,
    #[validate(non_control_character, length(min = 1, max = 50))]
    last_name: String,
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
    #[validate(
        non_control_character,
        length(
            min = 10,
            max = 100,
            message = "Password must be at least 10 characters long and at most 100 characters long"
        )
    )]
    password: String,
    turnstile_token: Option<String>,
    /// Optional Google Ads click identifier captured during the user's
    /// journey from a Google Ad. When present and the API has Google Ads
    /// credentials configured, the signup is uploaded as a click conversion
    /// (server-side, no PII leaves Hook0). Bounded length to defend against
    /// abuse — real gclid values are ~50–60 chars.
    #[validate(non_control_character, length(max = 256))]
    gclid: Option<String>,
}

#[api_v2_operation(
    summary = "Create a new user account and its own personal organization",
    description = "If instance has Cloudflare Turnstile enabled (see response of /instance endpoint), the `turnstile_token` field is mandatory.",
    operation_id = "register",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn register(
    state: Data<crate::State>,
    ip: UserIp,
    body: Json<RegistrationPost>,
) -> Result<CreatedJson<Registration>, Hook0Problem> {
    if state.registration_disabled {
        return Err(Hook0Problem::RegistrationDisabled);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    if let Some(secret_key) = state.cloudflare_turnstile_secret_key.as_deref() {
        crate::cloudflare_turnstile::verify(secret_key, body.turnstile_token.as_deref(), &ip)
            .await?;
    }

    let recipient_address = Address::from_str(&body.email).map_err(|e| {
        // Should not happen because we checked (using a validator) that body.email is a well structured email address
        error!("Error trying to parse email address: {e}");
        Hook0Problem::InternalServerError
    })?;

    if body.password.len() >= usize::from(state.password_minimum_length) {
        let user_id = Uuid::new_v4();
        let password = body.password.clone();
        let password_hash = spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
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
        })??;

        let mut tx = state.db.begin().await?;
        let user_insert = query!(
            "
                INSERT INTO iam.user (user__id, email, password, first_name, last_name)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (email) DO NOTHING
            ",
            &user_id,
            &body.email,
            password_hash.as_str(),
            &body.first_name,
            &body.last_name,
        )
        .execute(&mut *tx)
        .await?;

        if user_insert.rows_affected() > 0 {
            let organization_id = Uuid::new_v4();
            let organization_name = format!(
                "{} {}'s personal organization",
                &body.first_name, &body.last_name
            );
            query!(
                "
                    INSERT INTO iam.organization (organization__id, name, created_by)
                    VALUES ($1, $2, $3)
                ",
                &organization_id,
                &organization_name,
                &user_id,
            )
            .execute(&mut *tx)
            .await?;

            query!(
                "
                    INSERT INTO iam.user__organization (user__id, organization__id, role)
                    VALUES ($1, $2, $3)
                ",
                &user_id,
                &organization_id,
                Role::Editor.as_ref(),
            )
            .execute(&mut *tx)
            .await?;

            let verification_token =
                create_email_verification_token(&state.biscuit_private_key, user_id).map_err(
                    |e| {
                        error!("Error trying to create email verification token: {e}");
                        Hook0Problem::InternalServerError
                    },
                )?;
            let recipient = Mailbox::new(
                Some(format!("{} {}", body.first_name, body.last_name)),
                recipient_address,
            );
            let url = {
                let mut url = state
                    .app_url
                    .join("verify-email")
                    .map_err(|_| Hook0Problem::InternalServerError)?;
                url.query_pairs_mut()
                    .append_pair("token", &verification_token.serialized_biscuit);
                url
            };
            state
                .mailer
                .send_mail(Mail::VerifyUserEmail { url }, recipient)
                .await
                .map_err(|e| {
                    warn!("Could not send verification email: {e}");
                    e
                })?;

            // Persist the gclid alongside the user so the conversion can be
            // uploaded once the user verifies their email — this filters out
            // throwaway / bot signups from the conversion stream. The
            // attribution row is deleted on successful upload (verify_email
            // handler); a periodic 30-day cleanup runs lazily here too so
            // unverified signups don't accumulate.
            let trimmed_gclid = body
                .gclid
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            if let Some(gclid) = trimmed_gclid {
                query!(
                    "
                        INSERT INTO iam.signup_attribution (user__id, gclid)
                        VALUES ($1, $2)
                    ",
                    &user_id,
                    &gclid,
                )
                .execute(&mut *tx)
                .await?;
            }

            tx.commit().await?;

            // Lazy cleanup: drop attributions older than 30 days. Runs in a
            // separate connection (not in tx) so it never blocks signup.
            // Errors are logged but never surfaced.
            let pool = state.db.clone();
            tokio::spawn(async move {
                let result = query!(
                    "
                        DELETE FROM iam.signup_attribution
                        WHERE created_at < statement_timestamp() - INTERVAL '30 days'
                    "
                )
                .execute(&pool)
                .await;
                match result {
                    Ok(done) if done.rows_affected() > 0 => {
                        tracing::info!(
                            target: "api::signup_attribution",
                            rows = done.rows_affected(),
                            "pruned stale signup attribution rows"
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(
                            target: "api::signup_attribution",
                            error = %e,
                            "failed to prune stale signup attribution rows"
                        );
                    }
                }
            });

            Ok(CreatedJson(Registration {
                organization_id,
                user_id,
            }))
        } else {
            Err(Hook0Problem::UserAlreadyExist)
        }
    } else {
        Err(Hook0Problem::PasswordTooShort(
            state.password_minimum_length,
        ))
    }
}
