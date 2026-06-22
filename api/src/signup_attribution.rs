//! Helpers around `iam.signup_attribution` shared by the registration / email
//! verification flow (signup conversion) and the application-secret creation
//! flow (activation conversion).
//!
//! The gclid is retained until BOTH conversions have been uploaded to Google
//! Ads, then cleared (data minimisation). The 30-day cleanup in
//! `handlers::registrations` is the safety net for rows that never reach that
//! state. See
//! `documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md`.

use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

/// Maximum gclid length accepted, mirroring the `signup_attribution_gclid_length`
/// DB CHECK. Real Google gclids are ~50-100 chars; anything longer is treated as
/// invalid and dropped — this bounds untrusted input and avoids failing the
/// INSERT on the length CHECK.
pub const MAX_GCLID_LEN: usize = 256;

/// Normalize a raw gclid from the registration payload: trim surrounding
/// whitespace, then drop it if empty or longer than [`MAX_GCLID_LEN`] characters.
/// Returns the value to store, or `None` when there is nothing valid to keep.
pub fn normalize_gclid(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().count() <= MAX_GCLID_LEN)
        .map(str::to_string)
}

/// Atomically claim the activation conversion for an organization.
///
/// The `UPDATE ... RETURNING` makes this fire **at most once** per organization
/// even under concurrent first-API-key creations: only the statement that flips
/// `activation_uploaded_at` from NULL wins and returns the gclid. Returns `None`
/// when there is nothing to upload (no attribution row for the org, gclid
/// already cleared, or activation already claimed).
pub async fn claim_activation_gclid(
    db: &PgPool,
    organization_id: &Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET activation_uploaded_at = statement_timestamp()
            WHERE organization__id = $1
              AND activation_uploaded_at IS NULL
              AND gclid IS NOT NULL
            RETURNING gclid
        ",
        organization_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(row.and_then(|r| r.gclid))
}

/// Clear the gclid (data minimisation) once BOTH conversions are uploaded, for
/// the attribution row of `user_id`. Best-effort: errors are logged, never
/// surfaced (the conversion has already been queued).
pub async fn clear_gclid_if_fully_uploaded_by_user(db: &PgPool, user_id: &Uuid) {
    let result = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET gclid = NULL
            WHERE user__id = $1
              AND gclid IS NOT NULL
              AND signup_uploaded_at IS NOT NULL
              AND activation_uploaded_at IS NOT NULL
        ",
        user_id,
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        warn!(
            target: "api::signup_attribution",
            error = %e,
            "failed to clear minimised gclid (by user)"
        );
    }
}

/// Same as [`clear_gclid_if_fully_uploaded_by_user`], keyed by organization.
pub async fn clear_gclid_if_fully_uploaded_by_org(db: &PgPool, organization_id: &Uuid) {
    let result = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET gclid = NULL
            WHERE organization__id = $1
              AND gclid IS NOT NULL
              AND signup_uploaded_at IS NOT NULL
              AND activation_uploaded_at IS NOT NULL
        ",
        organization_id,
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        warn!(
            target: "api::signup_attribution",
            error = %e,
            "failed to clear minimised gclid (by org)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn normalize_drops_absent_empty_and_whitespace() {
        assert_eq!(normalize_gclid(None), None);
        assert_eq!(normalize_gclid(Some("")), None);
        assert_eq!(normalize_gclid(Some("   ")), None);
        assert_eq!(normalize_gclid(Some("\t\n ")), None);
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_gclid(Some("  Cj0KCQ...  ")),
            Some("Cj0KCQ...".to_string())
        );
    }

    #[test]
    fn normalize_drops_overlong_keeps_at_limit() {
        let too_long = "a".repeat(MAX_GCLID_LEN + 1);
        assert_eq!(normalize_gclid(Some(&too_long)), None);

        let at_limit = "a".repeat(MAX_GCLID_LEN);
        assert_eq!(normalize_gclid(Some(&at_limit)), Some(at_limit));
    }

    proptest! {
        // Output invariant: the stored gclid is always None, or a non-empty,
        // trimmed string within the DB length bound. Guarantees we never INSERT
        // a value the `signup_attribution_gclid_length` CHECK would reject.
        #[test]
        fn normalized_output_is_bounded_and_trimmed(raw in ".*") {
            if let Some(s) = normalize_gclid(Some(&raw)) {
                prop_assert!(!s.is_empty());
                prop_assert!(s.chars().count() <= MAX_GCLID_LEN);
                prop_assert_eq!(s.trim(), s.as_str());
            }
        }

        // Idempotence: normalizing an already-normalized value changes nothing.
        #[test]
        fn normalize_is_idempotent(raw in ".*") {
            let once = normalize_gclid(Some(&raw));
            let twice = normalize_gclid(once.as_deref());
            prop_assert_eq!(once, twice);
        }
    }
}
