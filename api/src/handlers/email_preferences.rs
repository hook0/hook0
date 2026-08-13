//! Public endpoint behind the "stop these reminders" link of the reactivation
//! drip.
//!
//! It carries its own signed token instead of a session on purpose: an opt-out
//! that first demands a sign-in is not an opt-out, and a reader who cannot get
//! back into their account is exactly the reader most likely to want out. The
//! token grants nothing else — it only ever turns reminders off — so the trade
//! is a link that stops mail versus a link that could leak an account.

use biscuit_auth::Biscuit;
use paperclip::actix::web::{Data, Json};
use paperclip::actix::{Apiv2Schema, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query;
use tracing::debug;
use validator::Validate;

use crate::iam::authorize_reactivation_unsubscribe;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct UnsubscribeReactivationPost {
    // A bearer credential: `secret_token` bounds it without echoing it back.
    #[validate(custom(function = "crate::validators::secret_token"))]
    token: String,
}

#[api_v2_operation(
    summary = "Stop reactivation emails",
    description = "Opt an account out of the onboarding reactivation email sequence. Authenticated by the token carried by the unsubscribe link of those emails, so no session is required. Transactional email (verification, password reset) is unaffected.",
    operation_id = "emailPreferences.unsubscribeReactivation",
    consumes = "application/json",
    produces = "application/json",
    tags("Email preferences")
)]
pub async fn unsubscribe_reactivation(
    state: Data<crate::State>,
    body: Json<UnsubscribeReactivationPost>,
) -> Result<NoContent, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let body = body.into_inner();

    let biscuit =
        Biscuit::from_base64(body.token, state.biscuit_private_key.public()).map_err(|e| {
            debug!("{e}");
            Hook0Problem::AuthEmailExpired
        })?;

    let token = authorize_reactivation_unsubscribe(&biscuit).map_err(|e| {
        debug!("{e}");
        Hook0Problem::AuthEmailExpired
    })?;

    // Idempotent: a second click (or a mail client prefetching the page) keeps
    // the original opt-out timestamp instead of moving it.
    query!(
        "
            UPDATE iam.user
            SET reactivation_opted_out_at = statement_timestamp()
            WHERE user__id = $1 AND reactivation_opted_out_at IS NULL
        ",
        &token.user_id,
    )
    .execute(&state.db)
    .await?;

    Ok(NoContent)
}
