//! Background drip that re-engages verified accounts which never sent an event.
//!
//! A large share of verified sign-ups never ingest a single event and nothing
//! nudges them. This job walks the same "event" onboarding signal computed in
//! `onboarding.rs` (here inlined as a `NOT EXISTS` on `event.event`) and sends a
//! short, well-bounded sequence:
//!   - J+1: send your first webhook (dashboard wizard + curl snippet)
//!   - J+3: lift the #1 blocker — a URL to receive it — via play.hook0.com
//!   - J+7: last nudge, routing to the community (Discord) and human support
//!
//! Best practice honoured: 1 email = 1 job = 1 CTA. The series stops the moment
//! the org sends its first event (activation) or after the last step.
//!
//! Idempotency & bounds. Each (organization, step) is an exclusive claim in
//! `iam.reactivation_email` (unique PK), so the same step is never sent twice —
//! across passes AND across API instances. Every step selection is `LIMIT`-ed,
//! so a pass does bounded work in space (rows) and, transitively, in time. A
//! send failure releases the claim so a later pass retries (claim-first, so the
//! happy path is at-most-once; a duplicate is only possible in the narrow crash
//! window between a successful send and the claim commit — the same trade-off
//! `google_ads::first_event_conversion` already accepts).
//!
//! Sequencing. Candidates for every step are snapshotted at the START of a pass,
//! before any send, so within one pass an org qualifies for at most one step.
//! That alone does not pace the dormant backlog: an account already older than
//! every sign-up-age threshold would otherwise pick up the next step on each
//! (sub-daily) pass and receive all three within hours. So each step past the
//! first also requires its predecessor's `sent_at` to be at least the gap
//! between consecutive day-offsets in the past (J+3 ≥ 2 days after J+1, J+7 ≥ 4
//! days after J+3). This holds the real J+1/J+3/J+7 cadence for brand-new
//! sign-ups AND the long-dormant backlog.
//!
//! On by default: the job is spawned unless `ENABLE_REACTIVATION_EMAILS` is set
//! to false, which is the escape hatch for self-hosted instances that do not
//! want Hook0 to email their users.

use actix_web::rt::time::sleep;
use lettre::Address;
use lettre::message::Mailbox;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

use crate::mailer::{Mail, Mailer};
use crate::problems::Hook0Problem;

/// Let the API settle (SMTP connection test, migrations) before the first pass.
const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(55);

const STEP_DAY1: i16 = 1;
const STEP_DAY3: i16 = 2;
const STEP_DAY7: i16 = 3;

/// One step of the drip: which step number, the minimum sign-up age (in days)
/// that unlocks it, and which step must already have been sent before it.
struct StepSpec {
    step: i16,
    min_age_days: i32,
    predecessor: Option<i16>,
}

/// The J+1 / J+3 / J+7 sequence. Product-fixed cadence, so kept as constants
/// rather than ops knobs.
const STEPS: [StepSpec; 3] = [
    StepSpec {
        step: STEP_DAY1,
        min_age_days: 1,
        predecessor: None,
    },
    StepSpec {
        step: STEP_DAY3,
        min_age_days: 3,
        predecessor: Some(STEP_DAY1),
    },
    StepSpec {
        step: STEP_DAY7,
        min_age_days: 7,
        predecessor: Some(STEP_DAY3),
    },
];

/// Minimum days that must elapse after the predecessor step was sent before this
/// step may fire, derived from the gap between consecutive sign-up-age thresholds
/// (J+3 is 2 days after J+1, J+7 is 4 days after J+3). Without it, an account
/// already past every threshold — the dormant backlog — would pick up the next
/// step on the very next pass and get all three within hours. The first step has
/// no predecessor, hence no spacing constraint.
fn min_days_since_predecessor(spec: &StepSpec) -> i32 {
    match spec.predecessor {
        None => 0,
        Some(pred_step) => STEPS
            .iter()
            .find(|s| s.step == pred_step)
            .map(|pred| spec.min_age_days - pred.min_age_days)
            .unwrap_or(0),
    }
}

/// Runtime configuration for the drip: the per-step CTA URLs and the per-step
/// per-pass row cap (bounds the work of a single pass).
#[derive(Debug, Clone)]
pub struct ReactivationConfig {
    pub play_url: Url,
    pub discord_url: Url,
    pub max_per_step_per_run: i64,
}

/// One recipient selected for a given step (the org's registrant).
#[derive(Debug)]
struct Candidate {
    organization_id: Uuid,
    step: i16,
    email: String,
    first_name: String,
}

pub async fn periodically_send_reactivation_emails(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    mailer: Mailer,
    config: ReactivationConfig,
    period: Duration,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        match run_reactivation_pass(db, &mailer, &config).await {
            Ok(0) => {}
            Ok(sent) => info!(
                target: "api::reactivation",
                sent = sent,
                "sent reactivation emails"
            ),
            Err(e) => error!("Could not run reactivation email pass: {e}"),
        }
        drop(permit);

        sleep(period).await;
    }
}

/// One pass: snapshot recipients for every step up front, then claim-and-send
/// each. Returns how many emails were sent.
async fn run_reactivation_pass(
    db: &PgPool,
    mailer: &Mailer,
    config: &ReactivationConfig,
) -> Result<u64, sqlx::Error> {
    let planned = collect_pass(db, config).await?;

    let mut sent: u64 = 0;
    for candidate in planned {
        // Exclusive claim BEFORE sending. A lost race (another instance) or an
        // already-recorded step yields false → skip, so no step is sent twice.
        if !claim_step(db, &candidate.organization_id, candidate.step).await? {
            continue;
        }

        match send_reactivation_email(mailer, &candidate, config).await {
            Ok(()) => sent += 1,
            Err(e) => {
                warn!(
                    target: "api::reactivation",
                    step = candidate.step,
                    error = %e,
                    "reactivation email send failed; releasing claim for retry"
                );
                if let Err(release_err) =
                    release_step(db, &candidate.organization_id, candidate.step).await
                {
                    error!(
                        target: "api::reactivation",
                        error = %release_err,
                        "failed to release reactivation claim after send failure"
                    );
                }
            }
        }
    }

    Ok(sent)
}

/// Snapshot the eligible recipients for every step, in order, BEFORE any send.
/// Snapshotting up front is what guarantees one email per org per pass.
async fn collect_pass(
    db: &PgPool,
    config: &ReactivationConfig,
) -> Result<Vec<Candidate>, sqlx::Error> {
    let mut all = Vec::new();
    for spec in &STEPS {
        let mut candidates = select_candidates(
            db,
            spec.step,
            spec.min_age_days,
            spec.predecessor,
            min_days_since_predecessor(spec),
            config.max_per_step_per_run,
        )
        .await?;
        all.append(&mut candidates);
    }
    Ok(all)
}

/// Verified registrants whose organization has never ingested an event, are old
/// enough for this step, have this step's predecessor recorded at least
/// `min_days_since_predecessor` days ago (if any), and have not already been sent
/// this step. The spacing on the predecessor's `sent_at` is what keeps the
/// J+1/J+3/J+7 cadence from collapsing for accounts already past every age
/// threshold (the dormant backlog). Bounded by `LIMIT`.
async fn select_candidates(
    db: &PgPool,
    step: i16,
    min_age_days: i32,
    predecessor: Option<i16>,
    min_days_since_predecessor: i32,
    limit: i64,
) -> Result<Vec<Candidate>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
            SELECT
                o.organization__id AS "organization_id!",
                u.email AS "email!",
                u.first_name AS "first_name!"
            FROM iam.organization AS o
            INNER JOIN iam.user AS u ON u.user__id = o.created_by
            WHERE u.email_verified_at IS NOT NULL
              AND u.created_at <= statement_timestamp() - MAKE_INTERVAL(days => $1)
              -- "Org has never ingested an event": this NOT EXISTS is the
              -- canonical "event sent" signal, intentionally mirrored inline
              -- here for a set-based batch job rather than calling per-org into
              -- onboarding.rs. It must stay in sync with the `event` projection
              -- of `get_organization_onboarding_steps` in api/src/onboarding.rs
              -- (same event.event ⋈ event.application on organization__id); a
              -- change to that definition is a known sync point for this query.
              AND NOT EXISTS (
                  SELECT 1
                  FROM event.event AS e
                  INNER JOIN event.application AS a ON e.application__id = a.application__id
                  WHERE a.organization__id = o.organization__id
              )
              AND NOT EXISTS (
                  SELECT 1 FROM iam.reactivation_email AS re
                  WHERE re.organization__id = o.organization__id AND re.step = $2
              )
              AND (
                  $3::smallint IS NULL
                  OR EXISTS (
                      SELECT 1 FROM iam.reactivation_email AS rp
                      WHERE rp.organization__id = o.organization__id
                        AND rp.step = $3
                        AND rp.sent_at <= statement_timestamp() - MAKE_INTERVAL(days => $5)
                  )
              )
            ORDER BY u.created_at
            LIMIT $4
        "#,
        min_age_days,
        step,
        predecessor,
        limit,
        min_days_since_predecessor,
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Candidate {
            organization_id: r.organization_id,
            step,
            email: r.email,
            first_name: r.first_name,
        })
        .collect())
}

/// Atomically claim (organization, step). Returns true only if THIS call
/// inserted the row, i.e. it now owns the send. Concurrent callers get false.
async fn claim_step(db: &PgPool, organization_id: &Uuid, step: i16) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"
            INSERT INTO iam.reactivation_email (organization__id, step)
            VALUES ($1, $2)
            ON CONFLICT (organization__id, step) DO NOTHING
        "#,
        organization_id,
        step,
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() == 1)
}

/// Undo a claim after a failed send so a later pass retries the step.
async fn release_step(db: &PgPool, organization_id: &Uuid, step: i16) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM iam.reactivation_email WHERE organization__id = $1 AND step = $2",
        organization_id,
        step,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn send_reactivation_email(
    mailer: &Mailer,
    candidate: &Candidate,
    config: &ReactivationConfig,
) -> Result<(), Hook0Problem> {
    let address = Address::from_str(&candidate.email).map_err(|e| {
        warn!(target: "api::reactivation", error = %e, "invalid recipient email address");
        Hook0Problem::InternalServerError
    })?;
    let recipient = Mailbox::new(Some(candidate.first_name.clone()), address);

    let mail = mail_for_step(candidate, config)?;
    mailer.send_mail(mail, recipient).await
}

/// Map a step number to its mail variant, wiring in the per-step CTA URL.
fn mail_for_step(candidate: &Candidate, config: &ReactivationConfig) -> Result<Mail, Hook0Problem> {
    let recipient_first_name = Some(candidate.first_name.clone());
    let mail = match candidate.step {
        STEP_DAY1 => Mail::ReactivationNoEventDay1 {
            recipient_first_name,
        },
        STEP_DAY3 => Mail::ReactivationNoEventDay3 {
            recipient_first_name,
            play_url: config.play_url.clone(),
        },
        STEP_DAY7 => Mail::ReactivationNoEventDay7 {
            recipient_first_name,
            discord_url: config.discord_url.clone(),
        },
        other => {
            // Unreachable: steps come from STEPS. Fail loud rather than send the wrong mail.
            error!(target: "api::reactivation", step = other, "unknown reactivation step");
            return Err(Hook0Problem::InternalServerError);
        }
    };
    Ok(mail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_ads::test_support::{failing_mailer, seed_event, seed_org, seed_user};

    /// Move a user's sign-up date `days` into the past so age thresholds unlock.
    async fn backdate_signup(pool: &PgPool, user: Uuid, days: i32) {
        sqlx::query(
            r#"UPDATE iam."user" SET created_at = statement_timestamp() - MAKE_INTERVAL(days => $1) WHERE user__id = $2"#,
        )
        .bind(days)
        .bind(user)
        .execute(pool)
        .await
        .expect("backdate signup");
    }

    /// Push a recorded step's send time `days` into the past so the minimum
    /// spacing before the next step is satisfied.
    async fn backdate_step_sent(pool: &PgPool, org: Uuid, step: i16, days: i32) {
        sqlx::query(
            r#"UPDATE iam.reactivation_email SET sent_at = statement_timestamp() - MAKE_INTERVAL(days => $1) WHERE organization__id = $2 AND step = $3"#,
        )
        .bind(days)
        .bind(org)
        .bind(step)
        .execute(pool)
        .await
        .expect("backdate step sent_at");
    }

    /// Clear a user's verification so they no longer look activable.
    async fn unverify(pool: &PgPool, user: Uuid) {
        sqlx::query(r#"UPDATE iam."user" SET email_verified_at = NULL WHERE user__id = $1"#)
            .bind(user)
            .execute(pool)
            .await
            .expect("unverify user");
    }

    /// The steps recorded for an org, ascending.
    async fn steps_sent(pool: &PgPool, org: Uuid) -> Vec<i16> {
        let rows: Vec<(i16,)> = sqlx::query_as(
            "SELECT step FROM iam.reactivation_email WHERE organization__id = $1 ORDER BY step",
        )
        .bind(org)
        .fetch_all(pool)
        .await
        .expect("read steps");
        rows.into_iter().map(|r| r.0).collect()
    }

    /// A large enough cap that the LIMIT never interferes with correctness tests.
    const NO_LIMIT: i64 = 1000;

    /// A verified registrant whose org has no event and is old enough is picked.
    #[sqlx::test]
    async fn eligible_verified_dormant_org_is_selected(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select");

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].organization_id, org);
        assert_eq!(candidates[0].step, STEP_DAY1);
    }

    /// An unverified account is never targeted (it is not yet activable).
    #[sqlx::test]
    async fn unverified_user_is_excluded(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;
        unverify(&pool, user).await;

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select");

        assert!(candidates.is_empty());
    }

    /// An account younger than the step threshold is not selected yet.
    #[sqlx::test]
    async fn too_recent_signup_is_excluded(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        // created_at defaults to now → age 0 < 1.

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select");

        assert!(candidates.is_empty());
    }

    /// An org that has ingested an event has activated: it drops out of the
    /// series entirely (even a step whose predecessor was already sent).
    #[sqlx::test]
    async fn activation_stops_the_series(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 5).await;

        // J+1 was already sent, long enough ago to clear the min spacing…
        assert!(claim_step(&pool, &org, STEP_DAY1).await.expect("claim d1"));
        backdate_step_sent(&pool, org, STEP_DAY1, 2).await;
        // …then the org sends its first event.
        seed_event(&pool, org).await;

        // No further step is offered, despite the predecessor being present.
        let day3 = select_candidates(&pool, STEP_DAY3, 3, Some(STEP_DAY1), 2, NO_LIMIT)
            .await
            .expect("select d3");
        assert!(day3.is_empty(), "activation must stop the series");
    }

    /// A step is only offered once its predecessor has been recorded (drip).
    #[sqlx::test]
    async fn step_requires_its_predecessor(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 5).await;

        // Without J+1 recorded, J+3 is not offered.
        let before = select_candidates(&pool, STEP_DAY3, 3, Some(STEP_DAY1), 2, NO_LIMIT)
            .await
            .expect("select before");
        assert!(before.is_empty());

        // Record J+1 far enough in the past to clear the min spacing, then J+3
        // becomes eligible.
        assert!(claim_step(&pool, &org, STEP_DAY1).await.expect("claim d1"));
        backdate_step_sent(&pool, org, STEP_DAY1, 2).await;
        let after = select_candidates(&pool, STEP_DAY3, 3, Some(STEP_DAY1), 2, NO_LIMIT)
            .await
            .expect("select after");
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].organization_id, org);
    }

    /// A dormant backlog account (old enough for every threshold) does NOT get
    /// the next step on the very next pass: the step waits until the minimum
    /// spacing since the predecessor's send has elapsed, so the J+1/J+3/J+7
    /// cadence holds even for accounts created long ago.
    #[sqlx::test]
    async fn next_step_waits_for_min_spacing_since_predecessor(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // 30 days old: past every sign-up-age threshold — the dormant backlog case.
        backdate_signup(&pool, user, 30).await;

        let config = ReactivationConfig {
            play_url: Url::parse("https://play.hook0.com/").unwrap(),
            discord_url: Url::parse("https://www.hook0.com/community").unwrap(),
            max_per_step_per_run: NO_LIMIT,
        };

        // J+1 recorded just now.
        assert!(claim_step(&pool, &org, STEP_DAY1).await.expect("claim d1"));

        // J+3 needs 2 days (3 - 1) since J+1 was sent → nothing offered yet, even
        // though the account is old enough for every threshold.
        let too_soon = collect_pass(&pool, &config)
            .await
            .expect("collect too soon");
        assert!(
            too_soon.is_empty(),
            "next step must wait for the min spacing since its predecessor, not fire on the next pass"
        );

        // Once J+1 is 2 days old, J+3 unlocks (and only J+3).
        backdate_step_sent(&pool, org, STEP_DAY1, 2).await;
        let ready = collect_pass(&pool, &config).await.expect("collect ready");
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].step, STEP_DAY3);
        assert_eq!(ready[0].organization_id, org);
    }

    /// Claiming a step is idempotent: the second attempt reports "not claimed".
    #[sqlx::test]
    async fn claim_is_idempotent(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;

        assert!(claim_step(&pool, &org, STEP_DAY1).await.expect("claim 1"));
        assert!(
            !claim_step(&pool, &org, STEP_DAY1).await.expect("claim 2"),
            "second claim of the same step must be a no-op"
        );
        assert_eq!(steps_sent(&pool, org).await, vec![STEP_DAY1]);
    }

    /// Releasing a claim (after a failed send) lets a later pass retry the step.
    #[sqlx::test]
    async fn release_allows_reselection(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        // Claimed → excluded from selection.
        assert!(claim_step(&pool, &org, STEP_DAY1).await.expect("claim"));
        let claimed = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select claimed");
        assert!(claimed.is_empty());

        // Released → selectable again.
        release_step(&pool, &org, STEP_DAY1).await.expect("release");
        let released = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select released");
        assert_eq!(released.len(), 1);
    }

    /// One pass offers at most one step per org, even when the account is old
    /// enough for every threshold (no predecessor rows exist yet).
    #[sqlx::test]
    async fn collect_pass_offers_one_step_per_org(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 30).await;

        let config = ReactivationConfig {
            play_url: Url::parse("https://play.hook0.com/").unwrap(),
            discord_url: Url::parse("https://www.hook0.com/community").unwrap(),
            max_per_step_per_run: NO_LIMIT,
        };
        let planned = collect_pass(&pool, &config).await.expect("collect");

        assert_eq!(planned.len(), 1, "at most one step per org per pass");
        assert_eq!(planned[0].step, STEP_DAY1);
        assert_eq!(planned[0].organization_id, org);
    }

    /// Driving passes manually walks an org J+1 → J+3 → J+7, then stops.
    #[sqlx::test]
    async fn drip_walks_all_three_steps_then_stops(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 30).await;

        let config = ReactivationConfig {
            play_url: Url::parse("https://play.hook0.com/").unwrap(),
            discord_url: Url::parse("https://www.hook0.com/community").unwrap(),
            max_per_step_per_run: NO_LIMIT,
        };

        // Simulate a pass by collecting then claiming (the SMTP send is covered
        // by the mailer's own rendering tests). Each successive step only unlocks
        // once the minimum spacing since its predecessor's send has elapsed (J+3
        // = 2 days after J+1, J+7 = 4 days after J+3), so age the predecessor
        // before collecting the next pass.
        let walk = [
            (STEP_DAY1, None, 0),
            (STEP_DAY3, Some(STEP_DAY1), 2),
            (STEP_DAY7, Some(STEP_DAY3), 4),
        ];
        for (expected_step, predecessor, gap_days) in walk {
            if let Some(pred) = predecessor {
                backdate_step_sent(&pool, org, pred, gap_days).await;
            }
            let planned = collect_pass(&pool, &config).await.expect("collect");
            assert_eq!(planned.len(), 1);
            assert_eq!(planned[0].step, expected_step);
            assert!(
                claim_step(&pool, &planned[0].organization_id, planned[0].step)
                    .await
                    .expect("claim")
            );
        }

        assert_eq!(
            steps_sent(&pool, org).await,
            vec![STEP_DAY1, STEP_DAY3, STEP_DAY7]
        );

        // After the last step, nothing more is offered.
        let empty = collect_pass(&pool, &config).await.expect("collect final");
        assert!(empty.is_empty(), "series stops after the last step");
    }

    /// The per-step LIMIT caps how many candidates a pass returns.
    #[sqlx::test]
    async fn selection_is_bounded_by_limit(pool: PgPool) {
        for _ in 0..3 {
            let user = seed_user(&pool).await;
            let _org = seed_org(&pool, user).await;
            backdate_signup(&pool, user, 2).await;
        }

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, 2)
            .await
            .expect("select");

        assert_eq!(candidates.len(), 2, "LIMIT must cap the selection");
    }

    /// A send failure inside the real pass loop must release the claim so the
    /// org stays eligible on the next pass — the send-failure→release
    /// orchestration, not `release_step` in isolation. The mailer is a real
    /// `Mailer` whose SMTP transport points at an unreachable endpoint, so the
    /// send fails at the boundary (a real failure, not a mock of the code under
    /// test). One eligible org is seeded; after the pass we assert nothing was
    /// counted as sent, no `reactivation_email` row survives (claim released),
    /// and the org is offered the same step again next pass.
    #[sqlx::test]
    async fn send_failure_releases_claim_so_org_stays_eligible(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        let config = ReactivationConfig {
            play_url: Url::parse("https://play.hook0.com/").unwrap(),
            discord_url: Url::parse("https://www.hook0.com/community").unwrap(),
            max_per_step_per_run: NO_LIMIT,
        };
        let mailer = failing_mailer().await;

        // Drive the REAL pass. The SMTP endpoint is unreachable, so the send
        // fails at the boundary and the loop must release the claim.
        let sent = run_reactivation_pass(&pool, &mailer, &config)
            .await
            .expect("pass runs despite send failure");
        assert_eq!(sent, 0, "a failed send must not be counted as sent");

        // Claim released: no row persisted for the org, so it is not marked as
        // already-sent for any step.
        assert!(
            steps_sent(&pool, org).await.is_empty(),
            "the claim must be released on send failure, leaving no persisted step"
        );

        // Org remains eligible for the same step on the next pass.
        let next = collect_pass(&pool, &config)
            .await
            .expect("collect next pass");
        assert_eq!(
            next.len(),
            1,
            "org must remain eligible after a failed send"
        );
        assert_eq!(next[0].organization_id, org);
        assert_eq!(next[0].step, STEP_DAY1);
    }
}
