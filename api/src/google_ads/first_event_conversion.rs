//! Background uploader for the Google Ads "first event sent" conversion.
//!
//! Unlike signup (email verification) and activation (first API key / service
//! token), which are attached to a user-facing HTTP handler, "first event sent"
//! has no natural single handler to hook into — and the event-ingestion path is
//! a hot path we must not touch. So this signal is uploaded by a periodic
//! set-based scan instead: it finds organizations that are gclid-attributed, have
//! ingested at least one event, and have not had their first-event conversion
//! uploaded yet, then uploads each and records success.
//!
//! Ordering is claim-on-success: `first_event_uploaded_at` is set only AFTER a
//! confirmed upload. A pass that crashes before marking simply re-processes the
//! org on the next scan (auto-recovery). The only cost is a rare double count if
//! a crash lands in the tiny window between Google accepting the upload and the
//! mark committing — acceptable versus permanently losing a conversion.
//!
//! Everything stays opt-in: the job is only spawned when a first-event
//! conversion action is configured (see `GOOGLE_ADS_FIRST_EVENT_CONVERSION_ACTION_ID`).

use actix_web::rt::time::sleep;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{ConversionKind, GoogleAdsClient, UploadOutcome};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(50);

/// Upper bound on organizations processed per pass. Each org is one upload
/// (network I/O), so this caps how long a single pass holds the housekeeping
/// semaphore. Pending first-events are normally near zero; a larger backlog
/// simply drains over successive passes. Bounds the work in space (rows) and,
/// transitively, in time.
const MAX_ORGS_PER_RUN: i64 = 500;

pub async fn periodically_upload_first_event_conversions(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    google_ads: Arc<GoogleAdsClient>,
    period: Duration,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        match upload_pending_first_event_conversions(db, &google_ads).await {
            Ok(0) => {}
            Ok(uploaded) => info!(
                target: "api::google_ads",
                uploaded = uploaded,
                "uploaded first-event conversions"
            ),
            Err(e) => error!("Could not upload first-event conversions: {e}"),
        }
        drop(permit);

        sleep(period).await;
    }
}

/// One scan pass: upload the first-event conversion for every eligible org and
/// mark the successful ones. Returns how many were newly marked as uploaded.
async fn upload_pending_first_event_conversions(
    db: &PgPool,
    google_ads: &GoogleAdsClient,
) -> Result<u64, sqlx::Error> {
    // Organizations that are attributed (gclid kept), have sent at least one
    // event, and whose first-event conversion is still pending. The `EXISTS`
    // mirrors the `event` onboarding step. Orgs without a gclid — including the
    // internal dogfooding org, which never carries a signup attribution — are
    // excluded by construction, so self-events never generate a conversion.
    // Backed by `signup_attribution_first_event_pending_idx`.
    let pending = sqlx::query!(
        r#"
            SELECT sa.organization__id AS "organization_id!", sa.gclid AS "gclid!"
            FROM iam.signup_attribution AS sa
            WHERE sa.gclid IS NOT NULL
              AND sa.first_event_uploaded_at IS NULL
              AND EXISTS (
                  SELECT 1
                  FROM event.event AS e
                  INNER JOIN event.application AS a ON e.application__id = a.application__id
                  WHERE a.organization__id = sa.organization__id
              )
            LIMIT $1
        "#,
        MAX_ORGS_PER_RUN,
    )
    .fetch_all(db)
    .await?;

    let mut uploaded: u64 = 0;
    for row in pending {
        let organization_id = row.organization_id;
        let gclid = row.gclid;

        // A fresh timestamp per pass. Google Ads dedups by (gclid, action,
        // dateTime); reruns of the SAME org only happen after a crash before
        // marking, so a duplicate is rare and bounded.
        match google_ads
            .upload_click_conversion(&gclid, ConversionKind::FirstEvent, Utc::now())
            .await
        {
            Ok(UploadOutcome::Success) => {
                super::report_conversion_uploaded("first_event", "success");
                if mark_and_minimise(db, &organization_id).await {
                    uploaded += 1;
                }
            }
            Ok(UploadOutcome::PartialFailure) => {
                // Terminal per-operation rejection (e.g. unknown / expired
                // gclid). Retrying will not help, so mark it done to stop
                // re-uploading a gclid Google will never accept. Counted as its
                // own outcome, never as a success.
                super::report_conversion_uploaded("first_event", "partial_failure");
                if mark_and_minimise(db, &organization_id).await {
                    uploaded += 1;
                }
            }
            Ok(UploadOutcome::Skipped) => {
                // Unreachable: the job is only spawned when the first-event
                // conversion action is configured. Leave the row for a later pass.
            }
            Err(e) => {
                // Transport/API error (transient 5xx/network, or a config 4xx).
                // Leave first_event_uploaded_at NULL so the next pass retries.
                super::report_conversion_uploaded("first_event", "failed");
                warn!(
                    target: "api::google_ads",
                    error = %e,
                    "first-event conversion upload failed, will retry next pass"
                );
            }
        }
    }

    Ok(uploaded)
}

/// Mark the org's first-event conversion as uploaded (claim-on-success) and, if
/// this call is the one that claimed it, minimise the gclid when every enabled
/// conversion is now done. Returns whether this call performed the claim.
async fn mark_and_minimise(db: &PgPool, organization_id: &Uuid) -> bool {
    match super::mark_first_event_uploaded(db, organization_id).await {
        Ok(true) => {
            // First-event tracking is necessarily enabled here (the job only
            // runs when it is), so pass `true`.
            super::clear_gclid_if_fully_uploaded_by_org(db, organization_id, true).await;
            true
        }
        Ok(false) => false,
        Err(e) => {
            error!(
                target: "api::google_ads",
                error = %e,
                "failed to mark first-event conversion uploaded"
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_ads::test_client_with_base_url;
    use crate::google_ads::test_support::{
        FakeGoogleAds, first_event_uploaded, seed_attribution, seed_event, seed_org, seed_user,
    };

    /// A gclid-attributed org that has sent an event gets its first-event
    /// conversion uploaded (to the first-event action) and marked, exactly once.
    #[sqlx::test]
    async fn uploads_and_marks_first_event_for_eligible_org(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"), Some("888"));

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_attribution(&pool, user, org, "gclid-first-event", true).await;
        seed_event(&pool, org).await;

        let uploaded = upload_pending_first_event_conversions(&pool, &client)
            .await
            .expect("pass ok");
        assert_eq!(uploaded, 1);

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 1, "exactly one first-event upload");
        let body: serde_json::Value = serde_json::from_str(&reqs[0].body).expect("json body");
        assert_eq!(body["conversions"][0]["gclid"], "gclid-first-event");
        assert_eq!(
            body["conversions"][0]["conversionAction"],
            "customers/1234567890/conversionActions/888"
        );
        assert!(first_event_uploaded(&pool, org).await);
    }

    /// Effective-once: a second pass uploads nothing because the org is marked.
    #[sqlx::test]
    async fn second_pass_is_a_noop(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"), Some("888"));

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_attribution(&pool, user, org, "gclid-once", true).await;
        seed_event(&pool, org).await;

        upload_pending_first_event_conversions(&pool, &client)
            .await
            .expect("first pass");
        let second = upload_pending_first_event_conversions(&pool, &client)
            .await
            .expect("second pass");

        assert_eq!(second, 0, "already-uploaded org is not re-processed");
        assert_eq!(fake.requests().len(), 1, "no second upload");
    }

    /// An org that has sent events but has no gclid attribution (the internal
    /// dogfooding org) never generates a first-event conversion.
    #[sqlx::test]
    async fn org_without_gclid_is_excluded(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"), Some("888"));

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // No seed_attribution → no gclid row at all.
        seed_event(&pool, org).await;

        let uploaded = upload_pending_first_event_conversions(&pool, &client)
            .await
            .expect("pass ok");

        assert_eq!(uploaded, 0);
        assert!(
            fake.requests().is_empty(),
            "no upload for a non-attributed org"
        );
    }

    /// An attributed org that has NOT sent any event yet is not processed.
    #[sqlx::test]
    async fn org_without_event_is_excluded(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"), Some("888"));

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_attribution(&pool, user, org, "gclid-no-event", true).await;
        // No seed_event.

        let uploaded = upload_pending_first_event_conversions(&pool, &client)
            .await
            .expect("pass ok");

        assert_eq!(uploaded, 0);
        assert!(fake.requests().is_empty());
        assert!(!first_event_uploaded(&pool, org).await);
    }

    /// A transient upload failure leaves the org pending so a later pass retries.
    #[sqlx::test]
    async fn transient_failure_leaves_org_pending(pool: PgPool) {
        let fake = FakeGoogleAds::start(503, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"), Some("888"));

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_attribution(&pool, user, org, "gclid-transient", true).await;
        seed_event(&pool, org).await;

        let uploaded = upload_pending_first_event_conversions(&pool, &client)
            .await
            .expect("pass ok");

        assert_eq!(uploaded, 0, "a 5xx does not mark the org as uploaded");
        assert!(
            !first_event_uploaded(&pool, org).await,
            "org stays pending for the next pass"
        );
    }
}
