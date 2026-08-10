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
use crate::password;
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
    // Length is deliberately not validated here: the policy owns both bounds
    // (`password::Checked::new`), so the user is told the instance's real
    // minimum instead of a number hardcoded next to the field, and an
    // oversized one is refused as `PasswordTooLong` rather than as malformed
    // input. `secret_characters` keeps the refused password out of the
    // response body, which the built-in validators echo back.
    #[validate(custom(function = "crate::validators::secret_characters"))]
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

    let checked_password = password::Checked::new(
        &body.password,
        state.password_minimum_length,
        &password::UserIdentity {
            email: &body.email,
            first_name: &body.first_name,
            last_name: &body.last_name,
        },
    )
    .map_err(|rejection| rejection.into_problem(state.password_minimum_length))?;

    let user_id = Uuid::new_v4();
    let password_hash = password::hash(checked_password).await?;

    let mut tx = state.db.begin().await?;
    // email_verification_sent_at is stamped here, not left NULL: the
    // verification email below goes out as part of this transaction, and the
    // resend endpoint anchors its cooldown on that column. Without the stamp
    // a signup followed immediately by a resend would put two identical mails
    // in the same mailbox before any throttle applied. The transaction rolls
    // back if the mail cannot be sent, so the stamp only survives alongside a
    // mail that really left.
    let user_insert = query!(
        "
                INSERT INTO iam.user (user__id, email, password, first_name, last_name, email_verification_sent_at)
                VALUES ($1, $2, $3, $4, $5, statement_timestamp())
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
            body.first_name, body.last_name
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
            create_email_verification_token(&state.biscuit_private_key, user_id).map_err(|e| {
                error!("Error trying to create email verification token: {e}");
                Hook0Problem::InternalServerError
            })?;
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
            .send_mail(
                Mail::VerifyUserEmail {
                    recipient_first_name: Some(body.first_name.clone()),
                    url,
                },
                recipient,
            )
            .await
            .map_err(|e| {
                warn!("Could not send verification email: {e}");
                e
            })?;

        // Persist the gclid alongside the user (and their personal
        // organization) so the enabled Google Ads conversions can be uploaded
        // server-side: "signup" once the email is verified (filters out
        // throwaway / bot signups), plus the optional "first event" and
        // "first webhook delivered" conversions when those conversion actions
        // are configured. The gclid is kept until every enabled conversion is
        // uploaded, then nulled (data minimisation); stale rows are pruned
        // both lazily here and by a periodic timer job
        // (signup_attribution_cleanup) so the retention window holds even
        // when signups pause.
        let normalized_gclid = crate::google_ads::normalize_gclid(body.gclid.as_deref());
        if let Some(gclid) = normalized_gclid {
            query!(
                "
                        INSERT INTO iam.signup_attribution (user__id, organization__id, gclid)
                        VALUES ($1, $2, $3)
                    ",
                &user_id,
                &organization_id,
                &gclid,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Lazy cleanup: drop attributions older than the retention window
        // (the gclid safety net — long enough for a B2B first event to
        // arrive, see SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS). Runs in a
        // separate connection (not in tx) so it never blocks signup.
        // Errors are logged but never surfaced.
        let pool = state.db.clone();
        let retention_in_days = state.signup_attribution_retention_in_days;
        tokio::spawn(async move {
            let result = query!(
                "
                        DELETE FROM iam.signup_attribution
                        WHERE created_at < statement_timestamp() - MAKE_INTERVAL(days => $1)
                    ",
                retention_in_days,
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
}

#[cfg(test)]
mod password_policy_tests {
    use crate::google_ads::test_support::test_state;
    use actix_web::{App, test, web};
    use sqlx::PgPool;

    /// Spin up the real registration endpoint over the test database, with no
    /// mocking: the policy has to reject through the actual HTTP contract.
    /// A macro rather than a function because the type of an initialized actix
    /// test service is not nameable here.
    macro_rules! init_api {
        ($pool:expr) => {{
            let keypair = biscuit_auth::KeyPair::new();
            let state = test_state($pool.clone(), keypair.private(), None).await;

            // The real IP middleware, because the handler extracts `UserIp` and
            // would answer 500 in plain text without it. No reverse proxy in
            // front here, so the peer address of the request is the user's.
            let get_user_ip = crate::middleware_get_user_ip::GetUserIp {
                reverse_proxy_cidrs: vec![],
                behind_cloudflare: false,
            };

            test::init_service(
                App::new().app_data(web::Data::new(state)).service(
                    web::scope("/api/v1").service(
                        web::scope("/register")
                            .wrap(get_user_ip)
                            .route("", web::post().to(super::register)),
                    ),
                ),
            )
            .await
        }};
    }

    /// POST a registration and return the response status and the `id` of the
    /// problem it carries (empty when the response is not a problem).
    macro_rules! register {
        ($app:expr, $email:expr, $password:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/register")
                // TEST-NET-1: the IP middleware needs a peer to call the user.
                .peer_addr("192.0.2.10:54321".parse().expect("test peer address"))
                .set_json(serde_json::json!({
                    "first_name": "Jordan",
                    "last_name": "Rivera",
                    "email": $email,
                    "password": $password,
                }))
                .to_request();
            let response = test::call_service(&$app, request).await;
            let status = response.status();
            let body: serde_json::Value = test::read_body_json(response).await;
            (
                status,
                body["id"].as_str().unwrap_or_default().to_owned(),
            )
        }};
    }

    /// The reported vulnerability, through the endpoint that carried it: an
    /// account could be registered with its own email address as its password.
    /// The request must be refused, and no account must be left behind.
    #[sqlx::test]
    async fn registering_with_the_email_address_as_password_is_refused(pool: PgPool) {
        let app = init_api!(pool);
        let email = "jordanrivera801@example.com";

        let (status, problem) = register!(app, email, email);

        assert_eq!(status, actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(problem, "PasswordSimilarToEmail");

        let created: i64 = sqlx::query_scalar("SELECT count(*) FROM iam.user WHERE email = $1")
            .bind(email)
            .fetch_one(&pool)
            .await
            .expect("count registered users");
        assert_eq!(created, 0, "a refused registration must create no account");
    }

    #[sqlx::test]
    async fn registering_with_a_common_password_is_refused(pool: PgPool) {
        let app = init_api!(pool);

        let (status, problem) = register!(app, "someone@example.com", "password1234");

        assert_eq!(status, actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(problem, "PasswordTooCommon");
    }

    #[sqlx::test]
    async fn registering_with_a_password_shorter_than_the_floor_is_refused(pool: PgPool) {
        let app = init_api!(pool);

        let (status, problem) = register!(app, "someone@example.com", "quilt-lamp");

        assert_eq!(status, actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(problem, "PasswordTooShort");
    }

    /// The counterpart of the tests above: the policy must let a real password
    /// through. The registration still fails here, but on the verification
    /// email the test SMTP server never answers — never on the password.
    #[sqlx::test]
    async fn a_password_unrelated_to_the_user_passes_the_policy(pool: PgPool) {
        let app = init_api!(pool);

        let (status, problem) = register!(app, "someone@example.com", "quilt lantern harbour");

        // Registration does not complete here — the verification mail cannot
        // leave a test harness with no SMTP server — so what this pins is that
        // the policy let the password through. Every rejection it can raise is
        // a 400 named "Password…"; the end-to-end proof that an accepted
        // password is stored and logs in lives in the Playwright suite.
        assert_ne!(
            status,
            actix_web::http::StatusCode::BAD_REQUEST,
            "a strong password was refused: {problem}"
        );
        assert!(
            !problem.starts_with("Password"),
            "a strong password was refused by the policy: {problem}"
        );
    }
}
