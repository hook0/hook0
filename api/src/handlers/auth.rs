use actix_web::web::ReqData;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use biscuit_auth::{Biscuit, PrivateKey};
use chrono::{DateTime, Utc};
use log::{error, info};
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson, NoContent};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Acquire, Postgres};
use uuid::Uuid;
use validator::Validate;

use crate::iam::{authorize_only_user, authorize_refresh_token, create_refresh_token, create_user_access_token, Action, authorize_email_verification};
use crate::openapi::{OaBiscuitRefresh, OaBiscuitUserAccess};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct LoginPost {
    #[validate(non_control_character, length(min = 1, max = 100))]
    email: String,
    #[validate(non_control_character, length(min = 1, max = 100))]
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct LoginResponse {
    access_token: String,
    access_token_expiration: DateTime<Utc>,
    refresh_token: String,
    refresh_token_expiration: DateTime<Utc>,
    email: String,
    first_name: String,
    last_name: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct EmailVerificationGet {
    #[validate(non_control_character, length(min = 1, max = 1000))]
    token: String,
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
    tags("Authentication")
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
        if user.email_verified_at.is_some() {
            let password_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
                error!(
                    "Password hash of user {} is not in the right format: {e}",
                    &user.user_id
                );
                Hook0Problem::InternalServerError
            })?;

            if Argon2::default()
                .verify_password(body.password.as_bytes(), &password_hash)
                .is_ok()
            {
                do_login(&state.db, &state.biscuit_private_key, user, None).await
            } else {
                Err(Hook0Problem::AuthFailedLogin)
            }
        } else {
            Err(Hook0Problem::AuthFailedLogin)
        }
    } else {
        #[cfg(feature = "migrate-users-from-keycloak")]
        {
            let user = import_user_from_keycloak(&state, &body.email, &body.password).await?;
            do_login(&state.db, &state.biscuit_private_key, user, None).await
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
    use argon2::PasswordHasher;
    use log::{debug, trace};

    reqwest::Client::new()
        .post(format!(
            "{}/realms/{}/protocol/openid-connect/token",
            &state.keycloak_url, &state.keycloak_realm
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

        let salt = argon2::password_hash::SaltString::generate(
            &mut argon2::password_hash::rand_core::OsRng,
        );
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                error!("Error trying to hash user password: {e}");
                Hook0Problem::InternalServerError
            })?
            .serialize();

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
            query!(
                "
                    INSERT INTO iam.user__organization (user__id, organization__id, role)
                    VALUES ($1, $2, $3)
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
    tags("Authentication")
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
    tags("Authentication")
)]
pub async fn logout(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
) -> Result<NoContent, Hook0Problem> {
    if let Ok(token) = authorize_only_user(&biscuit, None, Action::AuthLogout) {
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
    } else {
        Err(Hook0Problem::Forbidden)
    }
}

#[api_v2_operation(
    summary = "Email verification",
    description = "Verify the email of a user.",
    operation_id = "auth.verify_email",
    consumes = "application/json",
    produces = "application/json",
    tags("Authentication")
)]
pub async fn verify_email(
    state: Data<crate::State>,
    query: Json<EmailVerificationGet>,
) -> Result<NoContent, Hook0Problem> {
    info!("Email verification");
    if let Err(e) = query.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let token = query.token.clone();
    info!("Token: {}", token);
    let token = match biscuit_auth::Biscuit::from(token, &state.biscuit_private_key.public()) {
        Ok(token) => token,
        Err(e) => return Err(Hook0Problem::AuthFailedEmailVerification(Option::from(e.to_string()))),
    };
    if let Ok(token) = authorize_email_verification(&token) {

         match query!(
            "
                SELECT user__id
                FROM iam.user
                WHERE user__id = $1
                    AND email_verified_at IS NULL
            ",
            &token.user_id,
        ).fetch_optional(&state.db).await? {
            Some(_) => {},
            None => return Err(Hook0Problem::AuthFailedEmailVerification(Option::from("User already verified".to_owned()))),
         }

        query!(
            "
                UPDATE iam.user
                SET email_verified_at = statement_timestamp()
                WHERE user__id = $1
            ",
            &token.user_id,
        )
        .execute(&state.db)
        .await?;

        Ok(NoContent)
    } else {
        info!("Email verification failed");
        Err(Hook0Problem::AuthFailedEmailVerification(None))
    }
}