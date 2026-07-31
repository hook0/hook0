//! Server-side Matomo activation-event tracker.
//!
//! Emits Hook0's product-activation Goal — one Matomo event
//! (`category = activation`, `action = first-webhook-delivered`) per
//! organization on its genuine first successful webhook delivery — through
//! Matomo's HTTP Tracking API. The signal is inherently server-side (a
//! successful delivery happens deep inside the webhook-delivery worker, not in
//! the browser), so it cannot be a front-end `trackEvent`.
//!
//! RGPD posture: no PII ever leaves Hook0. The Matomo visitor id (`_id`) is a
//! pseudonymous 16-hex value deterministically derived from the (already
//! opaque) organization id; no email, name, IP or user identifier is sent.
//!
//! Everything ships dark: the event is emitted only when the Matomo URL, site
//! id and tracking `token_auth` are all configured. Detection runs as a
//! periodic, bounded background scan (never on the webhook-delivery hot path)
//! that claims each eligible organization exactly once via the nullable
//! `iam.organization.matomo_activation_emitted_at` marker: the marker is
//! stamped first (so the claim is exclusive across instances and passes), the
//! event is sent, and the claim is released (marker back to `NULL`) if the send
//! fails so the organization stays eligible on the next pass.
//!
//! Claim-before-send makes the *claim* exactly-once across replicas, but the
//! *emission* is at-most-once / best-effort: because the marker commits before
//! the send, a process crash between the claim and a completed send leaves the
//! marker stamped, so that organization is never re-claimed and its activation
//! event is dropped permanently — there is no reconciliation. This is the
//! deliberate inverse of the at-least-once Google Ads conversion job: an
//! explicit send failure retries, but a crash mid-emission under-counts rather
//! than risk over-counting a Goal that already de-dupes within a visit.

use actix_web::rt::time::sleep;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

/// Matomo event category for the product-activation signal.
const ACTIVATION_EVENT_CATEGORY: &str = "activation";
/// Matomo event action for the product-activation signal.
const ACTIVATION_EVENT_ACTION: &str = "first-webhook-delivered";

/// Timeout bounding a single Matomo tracking HTTP call.
const MATOMO_TRACKING_TIMEOUT: Duration = Duration::from_secs(15);

/// Wait before the first scan so the rest of the process has finished starting.
const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(50);

#[derive(Debug, Error)]
pub enum MatomoError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid Matomo URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("Matomo tracking API error (HTTP {status}): {body}")]
    Api { status: u16, body: String },
}

/// Configuration required to send server-side events to Matomo's Tracking API.
///
/// `Debug` is intentionally NOT derived: this struct holds the `token_auth`
/// secret, and printing it would leak a credential into logs.
#[derive(Clone)]
pub struct MatomoTrackingConfig {
    /// Base URL of the Matomo instance (e.g. `https://matomo.hook0.com/`).
    pub base_url: Url,
    /// Target Matomo site id (the Hook0 app site).
    pub site_id: u16,
    /// Matomo `token_auth`, required to override the event datetime (`cdt`).
    pub token_auth: String,
}

/// Outcome of a single activation-event emission, so callers can tell a real
/// send from a no-op when tracking is not configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackOutcome {
    /// The event was sent to Matomo (HTTP 2xx).
    Sent,
    /// Matomo tracking is not configured; nothing was sent.
    Skipped,
}

/// Matomo tracking client. When `config` is `None` the client is a no-op
/// returning [`TrackOutcome::Skipped`] (mirrors the Google Ads client's
/// `Skipped` behaviour when a conversion action is unset).
pub struct MatomoTrackingClient {
    http: reqwest::Client,
    config: Option<MatomoTrackingConfig>,
}

impl MatomoTrackingClient {
    pub fn new(config: Option<MatomoTrackingConfig>) -> Result<Arc<Self>, MatomoError> {
        let http = reqwest::Client::builder()
            .timeout(MATOMO_TRACKING_TIMEOUT)
            .build()?;
        Ok(Arc::new(Self { http, config }))
    }

    /// Returns `true` when server-side Matomo tracking is fully configured.
    pub fn is_enabled(&self) -> bool {
        self.config.is_some()
    }

    /// Deterministic 16-hex pseudonymous Matomo visitor id derived from the
    /// organization id. XOR-folds the two 64-bit halves of the (already opaque)
    /// organization UUID into a stable 64-bit value; carries no PII.
    fn visitor_id(organization_id: &Uuid) -> String {
        let n = organization_id.as_u128();
        let folded = ((n >> 64) as u64) ^ (n as u64);
        format!("{folded:016x}")
    }

    /// Emit the activation event for `organization_id`, dated at the moment of
    /// its first successful webhook delivery (`occurred_at`, a server-time
    /// override that requires `token_auth`).
    ///
    /// When no tracking config is set, this is a silent no-op returning
    /// [`TrackOutcome::Skipped`].
    pub async fn track_first_webhook_delivered(
        &self,
        organization_id: &Uuid,
        occurred_at: DateTime<Utc>,
    ) -> Result<TrackOutcome, MatomoError> {
        let Some(config) = &self.config else {
            debug!(
                target: "api::matomo",
                "matomo tracking not configured; skipping activation event"
            );
            return Ok(TrackOutcome::Skipped);
        };

        let endpoint = config.base_url.join("matomo.php")?;
        let visitor_id = Self::visitor_id(organization_id);
        let site_id = config.site_id.to_string();
        // UTC datetime override; requires token_auth. No PII, just the moment of
        // the org's first successful delivery.
        let cdt = occurred_at.format("%Y-%m-%d %H:%M:%S").to_string();

        let params: [(&str, &str); 8] = [
            ("idsite", site_id.as_str()),
            ("rec", "1"),
            ("e_c", ACTIVATION_EVENT_CATEGORY),
            ("e_a", ACTIVATION_EVENT_ACTION),
            ("_id", visitor_id.as_str()),
            ("cdt", cdt.as_str()),
            ("token_auth", config.token_auth.as_str()),
            // Ask Matomo to answer with a 204 instead of a tracking GIF.
            ("send_image", "0"),
        ];

        let resp = self.http.post(endpoint).form(&params).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MatomoError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(TrackOutcome::Sent)
    }
}

/// Periodically emit the server-side Matomo activation event for every
/// organization that reached its first successful webhook delivery. Runs a
/// bounded scan under the housekeeping semaphore, then sleeps for `period`.
pub async fn periodically_emit_activation_events(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    matomo: Arc<MatomoTrackingClient>,
    period: Duration,
    scan_limit: u32,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        match emit_pending_activation_events(db, &matomo, scan_limit).await {
            Ok(0) => {}
            Ok(emitted) => info!(
                target: "api::matomo",
                emitted = emitted,
                "emitted server-side Matomo activation events"
            ),
            Err(e) => error!("Could not emit Matomo activation events: {e}"),
        }
        drop(permit);

        sleep(period).await;
    }
}

/// One scan pass: claim eligible organizations, emit their activation event and
/// release the claim of any whose emission failed. Returns how many events were
/// actually sent.
async fn emit_pending_activation_events(
    db: &PgPool,
    matomo: &MatomoTrackingClient,
    scan_limit: u32,
) -> Result<u64, sqlx::Error> {
    // Dark by default: when Matomo tracking is not fully configured we neither
    // claim nor emit. In production the job is not even spawned in that case
    // (main.rs gates on the same config); this guard makes the pass a safe
    // no-op if ever called while unconfigured.
    if !matomo.is_enabled() {
        return Ok(0);
    }

    // Claim up to `scan_limit` organizations that have delivered at least one
    // webhook successfully (a request attempt with succeeded_at set) and have
    // not had their activation event emitted yet, stamping
    // matomo_activation_emitted_at to reserve them exclusively. The `IS NULL`
    // guard in both the CTE and the UPDATE holds the claim across instances and
    // passes (READ COMMITTED re-checks the guard on a concurrently-updated row).
    // The eligibility EXISTS probe and the first-delivery timestamp reuse the
    // existing request_attempt (application__id) index; no index is added on
    // that hot table. The pending set is backed by
    // `organization_matomo_activation_pending_idx`.
    let limit = i64::from(scan_limit);
    let claimed = sqlx::query!(
        r#"
            WITH eligible AS (
                SELECT
                    o.organization__id AS organization_id,
                    (
                        SELECT MIN(ra.succeeded_at)
                        FROM webhook.request_attempt AS ra
                        INNER JOIN event.application AS a ON ra.application__id = a.application__id
                        WHERE a.organization__id = o.organization__id
                          AND ra.succeeded_at IS NOT NULL
                    ) AS first_delivered_at
                FROM iam.organization AS o
                WHERE o.matomo_activation_emitted_at IS NULL
                  AND EXISTS (
                      SELECT 1
                      FROM webhook.request_attempt AS ra
                      INNER JOIN event.application AS a ON ra.application__id = a.application__id
                      WHERE a.organization__id = o.organization__id
                        AND ra.succeeded_at IS NOT NULL
                  )
                ORDER BY o.organization__id
                LIMIT $1
            )
            UPDATE iam.organization AS o
            SET matomo_activation_emitted_at = statement_timestamp()
            FROM eligible
            WHERE o.organization__id = eligible.organization_id
              AND o.matomo_activation_emitted_at IS NULL
            RETURNING
                o.organization__id AS "organization_id!",
                eligible.first_delivered_at AS "first_delivered_at!"
        "#,
        limit,
    )
    .fetch_all(db)
    .await?;

    let mut emitted: u64 = 0;
    for row in claimed {
        let organization_id = row.organization_id;
        let first_delivered_at = row.first_delivered_at;

        match matomo
            .track_first_webhook_delivered(&organization_id, first_delivered_at)
            .await
        {
            Ok(TrackOutcome::Sent) => {
                emitted += 1;
            }
            Ok(TrackOutcome::Skipped) => {
                // Unreachable while enabled (checked above); release the claim so
                // the organization stays eligible on the next pass.
                release_claim(db, &organization_id).await;
            }
            Err(e) => {
                // Transport/API error: release the claim so the next pass retries.
                warn!(
                    target: "api::matomo",
                    error = %e,
                    "Matomo activation event emission failed, will retry next pass"
                );
                release_claim(db, &organization_id).await;
            }
        }
    }

    Ok(emitted)
}

/// Release a previously-claimed organization by clearing its activation marker
/// so a failed emission is retried on the next pass. Best-effort: an error here
/// is logged, not propagated (the enclosing pass already handled the send).
async fn release_claim(db: &PgPool, organization_id: &Uuid) {
    let result = sqlx::query!(
        "UPDATE iam.organization SET matomo_activation_emitted_at = NULL WHERE organization__id = $1",
        organization_id,
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        error!(
            target: "api::matomo",
            error = %e,
            "failed to release Matomo activation claim"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_ads::test_support::{
        FakeGoogleAds, seed_event, seed_org, seed_request_attempt, seed_user,
    };

    const TEST_SITE_ID: u16 = 2;

    /// Build an enabled Matomo tracking client pointing at `base_url`.
    fn enabled_client(base_url: &str) -> Arc<MatomoTrackingClient> {
        let base_url = Url::parse(base_url).expect("parse matomo base url");
        MatomoTrackingClient::new(Some(MatomoTrackingConfig {
            base_url,
            site_id: TEST_SITE_ID,
            token_auth: "test-token-auth".to_string(),
        }))
        .expect("build matomo client")
    }

    /// Whether an org's activation marker is still NULL (not emitted / released).
    async fn activation_marker_is_null(pool: &PgPool, org: Uuid) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT matomo_activation_emitted_at IS NULL FROM iam.organization WHERE organization__id = $1",
        )
        .bind(org)
        .fetch_one(pool)
        .await
        .expect("read activation marker");
        row.0
    }

    /// Seed an org that has delivered at least one webhook successfully.
    async fn seed_org_with_delivery(pool: &PgPool) -> Uuid {
        let user = seed_user(pool).await;
        let org = seed_org(pool, user).await;
        let (application_id, event_id) = seed_event(pool, org).await;
        seed_request_attempt(pool, application_id, event_id, true).await;
        org
    }

    /// Dark by default: with no Matomo config, the pass emits nothing and never
    /// stamps the marker, even for an org with a successful delivery.
    #[sqlx::test]
    async fn dark_when_matomo_unset(pool: PgPool) {
        let client = MatomoTrackingClient::new(None).expect("build disabled client");
        let org = seed_org_with_delivery(&pool).await;

        let emitted = emit_pending_activation_events(&pool, &client, 500)
            .await
            .expect("pass ok");

        assert_eq!(emitted, 0, "nothing emitted while feature dark");
        assert!(
            activation_marker_is_null(&pool, org).await,
            "the marker stays NULL — nothing claimed while dark"
        );
    }

    /// A qualifying org gets exactly one activation event emitted and its marker
    /// stamped.
    #[sqlx::test]
    async fn emits_once_and_marks_for_eligible_org(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = enabled_client(&fake.base_url);
        let org = seed_org_with_delivery(&pool).await;

        let emitted = emit_pending_activation_events(&pool, &client, 500)
            .await
            .expect("pass ok");

        assert_eq!(emitted, 1);
        let reqs = fake.requests();
        assert_eq!(reqs.len(), 1, "exactly one activation event emitted");
        assert_eq!(
            reqs[0].path, "/matomo.php",
            "hits the Matomo tracking endpoint"
        );
        assert!(
            !activation_marker_is_null(&pool, org).await,
            "the marker is stamped after a successful emission"
        );
    }

    /// An org whose activation event was already emitted is never re-emitted.
    #[sqlx::test]
    async fn does_not_reemit_for_already_stamped_org(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = enabled_client(&fake.base_url);
        let org = seed_org_with_delivery(&pool).await;

        // Pre-stamp the marker as if a previous pass already emitted it.
        sqlx::query("UPDATE iam.organization SET matomo_activation_emitted_at = statement_timestamp() WHERE organization__id = $1")
            .bind(org)
            .execute(&pool)
            .await
            .expect("pre-stamp marker");

        let emitted = emit_pending_activation_events(&pool, &client, 500)
            .await
            .expect("pass ok");

        assert_eq!(emitted, 0, "already-stamped org is not re-emitted");
        assert!(
            fake.requests().is_empty(),
            "no HTTP request for an already-stamped org"
        );
    }

    /// When the send fails (unreachable Matomo endpoint), the claim is released
    /// (marker back to NULL) so the org stays eligible; a later working pass
    /// then emits it.
    #[sqlx::test]
    async fn send_failure_releases_claim(pool: PgPool) {
        // Enabled but pointing at an unreachable endpoint (connection refused).
        let failing_client = enabled_client("http://127.0.0.1:1/");
        let org = seed_org_with_delivery(&pool).await;

        let emitted = emit_pending_activation_events(&pool, &failing_client, 500)
            .await
            .expect("pass ok");

        assert_eq!(emitted, 0, "a failed send counts as zero emitted");
        assert!(
            activation_marker_is_null(&pool, org).await,
            "the claim is released after a failed send, keeping the org eligible"
        );

        // A subsequent working pass emits the event now that the org is eligible.
        let fake = FakeGoogleAds::start(200, "{}");
        let working_client = enabled_client(&fake.base_url);
        let emitted = emit_pending_activation_events(&pool, &working_client, 500)
            .await
            .expect("pass ok");

        assert_eq!(emitted, 1, "the released org is re-processed next pass");
        assert!(
            !activation_marker_is_null(&pool, org).await,
            "the marker is stamped after the retry succeeds"
        );
    }

    /// The tracking request carries the correct event and a pseudonymous, org-
    /// derived visitor id — and no PII (no email).
    #[sqlx::test]
    async fn builds_tracking_request_without_pii(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = enabled_client(&fake.base_url);
        let org = seed_org_with_delivery(&pool).await;

        emit_pending_activation_events(&pool, &client, 500)
            .await
            .expect("pass ok");

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 1);
        let body = &reqs[0].body;

        assert!(body.contains("idsite=2"), "targets the configured site id");
        assert!(body.contains("rec=1"), "is a tracking request");
        assert!(
            body.contains(&format!("e_c={ACTIVATION_EVENT_CATEGORY}")),
            "carries the activation event category"
        );
        assert!(
            body.contains(&format!("e_a={ACTIVATION_EVENT_ACTION}")),
            "carries the first-webhook-delivered event action"
        );
        let expected_visitor_id = MatomoTrackingClient::visitor_id(&org);
        assert!(
            body.contains(&format!("_id={expected_visitor_id}")),
            "carries the org-derived pseudonymous visitor id"
        );
        assert!(
            body.contains("token_auth="),
            "authenticates the server-time override"
        );
        assert!(body.contains("cdt="), "overrides the event datetime");

        // No PII: the seeded org owner's email must never appear in the request.
        assert!(
            !body.contains("%40example.com")
                && !body.contains("@example.com")
                && !body.contains("e2e-"),
            "no email or other PII leaks into the tracking request"
        );
    }
}
