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
//! Blast radius. Only sign-ups younger than `MAX_SIGNUP_AGE_DAYS` are ever
//! selected, so switching the job on nudges recent registrations rather than
//! mailing every dormant account ever created.
//!
//! On by default: the job is spawned unless `ENABLE_REACTIVATION_EMAILS` is set
//! to false, which is the escape hatch for self-hosted instances that do not
//! want Hook0 to email their users.

use actix_web::rt::time::sleep;
use lettre::Address;
use lettre::message::Mailbox;
use sqlx::PgPool;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

use biscuit_auth::PrivateKey;

use crate::mailer::{Mail, Mailer};
use crate::problems::Hook0Problem;

/// Let the API settle (SMTP connection test, migrations) before the first pass.
const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(55);

/// Upper bound on how old a sign-up may be to still enter the sequence.
///
/// This is an onboarding nudge, not a win-back campaign: "your account is ready
/// but no event has come through yet" only makes sense while the sign-up is
/// still fresh in the reader's mind. Without this bound, switching the job on
/// treats every dormant account ever created as if it had just registered, so a
/// years-old address receives a J+1 "5 minutes away" email. Beyond the bad
/// experience, mailing a large backlog of stale addresses earns bounces and spam
/// complaints on the very domain that carries verification and password-reset
/// mail, so the blast radius of enabling this job is capped here by design.
const MAX_SIGNUP_AGE_DAYS: i32 = 30;

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
    /// Base URL of the dashboard, used to build the per-recipient opt-out link.
    pub app_url: Url,
    /// Signs the opt-out token carried by that link.
    pub biscuit_private_key: PrivateKey,
}

/// One recipient selected for a given step (the org's registrant).
#[derive(Debug)]
struct Candidate {
    user_id: Uuid,
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
        if !claim_step(db, &candidate.user_id, candidate.step).await? {
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
                if let Err(release_err) = release_step(db, &candidate.user_id, candidate.step).await
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

    // Claims are per organization, but the mail lands in a person's inbox.
    // Nothing stops one account from owning several organizations, and a
    // dormant one of those is dormant in all of them — so without this the
    // same reader would get the same message once per organization, in the
    // same pass. Keep the first (steps are collected in order, so that is the
    // earliest step this reader is due) and leave the rest for later passes.
    let mut already_planned = HashSet::new();
    all.retain(|candidate| already_planned.insert(candidate.user_id));

    Ok(all)
}

/// Verified registrants whose organization has never ingested an event, are old
/// enough for this step but still within `MAX_SIGNUP_AGE_DAYS`, have this step's
/// predecessor recorded at least `min_days_since_predecessor` days ago (if any),
/// and have not already been sent this step. The spacing on the predecessor's
/// `sent_at` is what keeps the J+1/J+3/J+7 cadence from collapsing for accounts
/// already past every age threshold (the dormant backlog). Bounded by `LIMIT`.
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
                u.user__id AS "user_id!",
                u.email AS "email!",
                u.first_name AS "first_name!"
            FROM iam.user AS u
            WHERE u.email_verified_at IS NOT NULL
              -- Honour the opt-out offered in every reactivation email.
              AND u.reactivation_opted_out_at IS NULL
              AND u.created_at <= statement_timestamp() - MAKE_INTERVAL(days => $1)
              -- Onboarding nudge, not win-back: past this age the sign-up is no
              -- longer fresh and the account is left alone for good.
              AND u.created_at > statement_timestamp() - MAKE_INTERVAL(days => $6)
              -- Registered at least one organization: the mail talks about
              -- sending a first event, which needs somewhere to send it from.
              AND EXISTS (
                  SELECT 1 FROM iam.organization AS o
                  WHERE o.created_by = u.user__id
              )
              -- "This reader has never ingested an event", across every
              -- organization they registered. This NOT EXISTS is the canonical
              -- "event sent" signal, intentionally mirrored inline here for a
              -- set-based batch job rather than calling per-org into
              -- onboarding.rs. It must stay in sync with the `event` projection
              -- of `get_organization_onboarding_steps` in api/src/onboarding.rs
              -- (same event.event ⋈ event.application on organization__id); a
              -- change to that definition is a known sync point for this query.
              AND NOT EXISTS (
                  SELECT 1
                  FROM iam.organization AS o
                  INNER JOIN event.application AS a ON a.organization__id = o.organization__id
                  INNER JOIN event.event AS e ON e.application__id = a.application__id
                  WHERE o.created_by = u.user__id
              )
              AND NOT EXISTS (
                  SELECT 1 FROM iam.reactivation_email AS re
                  WHERE re.user__id = u.user__id AND re.step = $2
              )
              AND (
                  $3::smallint IS NULL
                  OR EXISTS (
                      SELECT 1 FROM iam.reactivation_email AS rp
                      WHERE rp.user__id = u.user__id
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
        MAX_SIGNUP_AGE_DAYS,
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Candidate {
            user_id: r.user_id,
            step,
            email: r.email,
            first_name: r.first_name,
        })
        .collect())
}

/// Atomically claim (organization, step). Returns true only if THIS call
/// inserted the row, i.e. it now owns the send. Concurrent callers get false.
async fn claim_step(db: &PgPool, user_id: &Uuid, step: i16) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"
            INSERT INTO iam.reactivation_email (user__id, step)
            VALUES ($1, $2)
            ON CONFLICT (user__id, step) DO NOTHING
        "#,
        user_id,
        step,
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() == 1)
}

/// Undo a claim after a failed send so a later pass retries the step.
async fn release_step(db: &PgPool, user_id: &Uuid, step: i16) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM iam.reactivation_email WHERE user__id = $1 AND step = $2",
        user_id,
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
    let unsubscribe_url = unsubscribe_url(candidate, config)?;
    let mail = match candidate.step {
        STEP_DAY1 => Mail::ReactivationNoEventDay1 {
            recipient_first_name,
            unsubscribe_url,
        },
        STEP_DAY3 => Mail::ReactivationNoEventDay3 {
            recipient_first_name,
            play_url: config.play_url.clone(),
            unsubscribe_url,
        },
        STEP_DAY7 => Mail::ReactivationNoEventDay7 {
            recipient_first_name,
            discord_url: config.discord_url.clone(),
            unsubscribe_url,
        },
        other => {
            // Unreachable: steps come from STEPS. Fail loud rather than send the wrong mail.
            error!(target: "api::reactivation", step = other, "unknown reactivation step");
            return Err(Hook0Problem::InternalServerError);
        }
    };
    Ok(mail)
}

/// Build the one-click opt-out link for this recipient: the dashboard's
/// unsubscribe page carrying a token that grants nothing except stopping these
/// reminders.
fn unsubscribe_url(
    candidate: &Candidate,
    config: &ReactivationConfig,
) -> Result<Url, Hook0Problem> {
    let token = crate::iam::create_reactivation_unsubscribe_token(
        &config.biscuit_private_key,
        candidate.user_id,
    )
    .map_err(|e| {
        error!(target: "api::reactivation", error = %e, "could not mint unsubscribe token");
        Hook0Problem::InternalServerError
    })?;

    let mut url = config.app_url.join("unsubscribe").map_err(|e| {
        error!(target: "api::reactivation", error = %e, "could not build unsubscribe URL");
        Hook0Problem::InternalServerError
    })?;
    url.query_pairs_mut()
        .append_pair("token", &token.serialized_biscuit);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_ads::test_support::{failing_mailer, seed_event, seed_org, seed_user};
    use biscuit_auth::{Biscuit, KeyPair};

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
    async fn backdate_step_sent(pool: &PgPool, user: Uuid, step: i16, days: i32) {
        sqlx::query(
            r#"UPDATE iam.reactivation_email SET sent_at = statement_timestamp() - MAKE_INTERVAL(days => $1) WHERE user__id = $2 AND step = $3"#,
        )
        .bind(days)
        .bind(user)
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

    /// Record that a user asked to stop receiving these reminders, as the
    /// public unsubscribe endpoint does.
    async fn opt_out(pool: &PgPool, user: Uuid) {
        sqlx::query(
            r#"UPDATE iam."user" SET reactivation_opted_out_at = statement_timestamp() WHERE user__id = $1"#,
        )
        .bind(user)
        .execute(pool)
        .await
        .expect("opt user out");
    }

    /// The steps recorded for an org, ascending.
    async fn steps_sent(pool: &PgPool, user: Uuid) -> Vec<i16> {
        let rows: Vec<(i16,)> = sqlx::query_as(
            "SELECT step FROM iam.reactivation_email WHERE user__id = $1 ORDER BY step",
        )
        .bind(user)
        .fetch_all(pool)
        .await
        .expect("read steps");
        rows.into_iter().map(|r| r.0).collect()
    }

    /// A large enough cap that the LIMIT never interferes with correctness tests.
    const NO_LIMIT: i64 = 1000;

    /// The runtime configuration the job is given in tests. The signing key is
    /// freshly generated so the opt-out links minted here are real, verifiable
    /// biscuits rather than placeholders.
    fn test_config(max_per_step_per_run: i64) -> ReactivationConfig {
        ReactivationConfig {
            play_url: Url::parse("https://play.hook0.com/").expect("play url"),
            discord_url: Url::parse("https://www.hook0.com/community").expect("discord url"),
            max_per_step_per_run,
            app_url: Url::parse("https://app.hook0.com/").expect("app url"),
            biscuit_private_key: KeyPair::new().private(),
        }
    }

    /// A verified registrant whose org has no event and is old enough is picked.
    #[sqlx::test]
    async fn eligible_verified_dormant_org_is_selected(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select");

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].user_id, user);
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

    /// Opting out is honoured by the selection itself, not merely by the mailer:
    /// a reader who clicked "stop these reminders" must never be picked again,
    /// for any step, however eligible they otherwise look.
    #[sqlx::test]
    async fn opted_out_user_is_never_selected(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        // Eligible before the opt-out — otherwise the emptiness below proves
        // nothing.
        let before = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select before opt-out");
        assert_eq!(before.len(), 1, "eligible until the reader opts out");

        opt_out(&pool, user).await;

        for spec in STEPS.iter() {
            let candidates = select_candidates(
                &pool,
                spec.step,
                spec.min_age_days,
                spec.predecessor,
                min_days_since_predecessor(spec),
                NO_LIMIT,
            )
            .await
            .expect("select after opt-out");
            assert!(
                candidates.is_empty(),
                "step {} still targets an opted-out reader",
                spec.step
            );
        }
    }

    /// The opt-out link in the email has to actually work: it must carry a
    /// biscuit that our own authorizer accepts and that resolves to the very
    /// recipient being mailed. A link pointing at the wrong user, or one the API
    /// rejects, is worse than no link at all.
    #[test]
    fn every_step_carries_a_verifiable_opt_out_link_for_its_recipient() {
        let config = test_config(NO_LIMIT);
        let public_key = config.biscuit_private_key.public();

        for spec in STEPS.iter() {
            let candidate = Candidate {
                user_id: Uuid::new_v4(),
                step: spec.step,
                email: "reader@example.com".to_owned(),
                first_name: "Reader".to_owned(),
            };

            let mail = mail_for_step(&candidate, &config).expect("build mail");
            let url = mail
                .unsubscribe_url()
                .expect("every reactivation email carries an opt-out link");

            assert_eq!(url.path(), "/unsubscribe");
            let token = url
                .query_pairs()
                .find(|(k, _)| k == "token")
                .map(|(_, v)| v.into_owned())
                .expect("opt-out link carries a token");

            let biscuit = Biscuit::from_base64(&token, public_key).expect("token is a biscuit");
            let authorized =
                crate::iam::authorize_reactivation_unsubscribe(&biscuit).expect("token authorizes");
            assert_eq!(
                authorized.user_id, candidate.user_id,
                "step {} links to the wrong account",
                spec.step
            );
        }
    }

    /// Sign-ups older than the upper bound are never contacted. Without this,
    /// enabling the job would treat the whole historical backlog of dormant
    /// accounts as fresh registrations and mail every one of them a "your account
    /// is ready" nudge — years after the fact, on the same domain that carries
    /// verification and password-reset mail.
    #[sqlx::test]
    async fn sign_ups_older_than_the_upper_bound_are_left_alone(pool: PgPool) {
        let stale_user = seed_user(&pool).await;
        let _stale_org = seed_org(&pool, stale_user).await;
        backdate_signup(&pool, stale_user, MAX_SIGNUP_AGE_DAYS + 1).await;

        // A recent sign-up in the same state, to prove the query is selecting at
        // all and the emptiness below is really the age bound.
        let fresh_user = seed_user(&pool).await;
        let _fresh_org = seed_org(&pool, fresh_user).await;
        backdate_signup(&pool, fresh_user, 2).await;

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select d1");

        let selected: Vec<_> = candidates.iter().map(|c| c.user_id).collect();
        assert!(
            selected.contains(&fresh_user),
            "a recent dormant sign-up is still nudged"
        );
        assert!(
            !selected.contains(&stale_user),
            "a sign-up past the upper bound must never enter the sequence"
        );
    }

    /// Right at the bound the account is already too old: the comparison is
    /// strict, so `MAX_SIGNUP_AGE_DAYS` is the first age that is excluded.
    #[sqlx::test]
    async fn the_upper_bound_itself_is_excluded(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, MAX_SIGNUP_AGE_DAYS).await;

        let candidates = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select d1");

        assert!(
            !candidates.iter().any(|c| c.user_id == user),
            "the bound is exclusive"
        );
    }

    /// An org that has ingested an event has activated: it drops out of the
    /// series entirely (even a step whose predecessor was already sent).
    #[sqlx::test]
    async fn activation_stops_the_series(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 5).await;

        // J+1 was already sent, long enough ago to clear the min spacing…
        assert!(claim_step(&pool, &user, STEP_DAY1).await.expect("claim d1"));
        backdate_step_sent(&pool, user, STEP_DAY1, 2).await;
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
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 5).await;

        // Without J+1 recorded, J+3 is not offered.
        let before = select_candidates(&pool, STEP_DAY3, 3, Some(STEP_DAY1), 2, NO_LIMIT)
            .await
            .expect("select before");
        assert!(before.is_empty());

        // Record J+1 far enough in the past to clear the min spacing, then J+3
        // becomes eligible.
        assert!(claim_step(&pool, &user, STEP_DAY1).await.expect("claim d1"));
        backdate_step_sent(&pool, user, STEP_DAY1, 2).await;
        let after = select_candidates(&pool, STEP_DAY3, 3, Some(STEP_DAY1), 2, NO_LIMIT)
            .await
            .expect("select after");
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].user_id, user);
    }

    /// A dormant backlog account (old enough for every threshold) does NOT get
    /// the next step on the very next pass: the step waits until the minimum
    /// spacing since the predecessor's send has elapsed, so the J+1/J+3/J+7
    /// cadence holds even for accounts created long ago.
    #[sqlx::test]
    async fn next_step_waits_for_min_spacing_since_predecessor(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        // Past every sign-up-age threshold, still within MAX_SIGNUP_AGE_DAYS.
        backdate_signup(&pool, user, 10).await;

        let config = test_config(NO_LIMIT);

        // J+1 recorded just now.
        assert!(claim_step(&pool, &user, STEP_DAY1).await.expect("claim d1"));

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
        backdate_step_sent(&pool, user, STEP_DAY1, 2).await;
        let ready = collect_pass(&pool, &config).await.expect("collect ready");
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].step, STEP_DAY3);
        assert_eq!(ready[0].user_id, user);
    }

    /// Claiming a step is idempotent: the second attempt reports "not claimed".
    #[sqlx::test]
    async fn claim_is_idempotent(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;

        assert!(claim_step(&pool, &user, STEP_DAY1).await.expect("claim 1"));
        assert!(
            !claim_step(&pool, &user, STEP_DAY1).await.expect("claim 2"),
            "second claim of the same step must be a no-op"
        );
        assert_eq!(steps_sent(&pool, user).await, vec![STEP_DAY1]);
    }

    /// Releasing a claim (after a failed send) lets a later pass retry the step.
    #[sqlx::test]
    async fn release_allows_reselection(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        // Claimed → excluded from selection.
        assert!(claim_step(&pool, &user, STEP_DAY1).await.expect("claim"));
        let claimed = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select claimed");
        assert!(claimed.is_empty());

        // Released → selectable again.
        release_step(&pool, &user, STEP_DAY1)
            .await
            .expect("release");
        let released = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select released");
        assert_eq!(released.len(), 1);
    }

    /// One pass offers at most one step per reader, even when the account is old
    /// enough for every threshold (no predecessor rows exist yet).
    #[sqlx::test]
    async fn collect_pass_offers_one_step_per_reader(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 10).await;

        let config = test_config(NO_LIMIT);
        let planned = collect_pass(&pool, &config).await.expect("collect");

        assert_eq!(planned.len(), 1, "at most one step per reader per pass");
        assert_eq!(planned[0].step, STEP_DAY1);
        assert_eq!(planned[0].user_id, user);
    }

    /// One reader, several organizations: the mail lands in one inbox and never
    /// names an organization, and nothing caps how many an account may create.
    /// The step is claimed for the reader, so the second organization can never
    /// produce a second copy — not in this pass, not in the next one, and not
    /// from another API instance racing this one.
    #[sqlx::test]
    async fn a_reader_with_several_organizations_is_mailed_once(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _first_org = seed_org(&pool, user).await;
        let _second_org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        // The reader is offered once, not once per organization.
        let selected = select_candidates(&pool, STEP_DAY1, 1, None, 0, NO_LIMIT)
            .await
            .expect("select");
        assert_eq!(
            selected.len(),
            1,
            "the selection is per reader, whatever they registered"
        );

        let config = test_config(NO_LIMIT);
        let planned = collect_pass(&pool, &config).await.expect("collect");

        assert_eq!(
            planned.len(),
            1,
            "one reader must receive at most one message per pass"
        );
        assert_eq!(planned[0].user_id, user);

        // The claim is the reader's, so a second attempt at the same step —
        // the next pass, or another API instance mid-pass — is refused rather
        // than landing on the reader's other organization.
        assert!(
            claim_step(&pool, &planned[0].user_id, planned[0].step)
                .await
                .expect("claim")
        );
        assert!(
            !claim_step(&pool, &user, STEP_DAY1)
                .await
                .expect("second claim"),
            "a concurrent instance must not be able to claim the same step again"
        );
        let next = collect_pass(&pool, &config).await.expect("collect next");
        assert!(
            next.is_empty(),
            "a step already sent to this reader must not come back through another organization"
        );
    }

    /// Driving passes manually walks an org J+1 → J+3 → J+7, then stops.
    #[sqlx::test]
    async fn drip_walks_all_three_steps_then_stops(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 10).await;

        let config = test_config(NO_LIMIT);

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
                backdate_step_sent(&pool, user, pred, gap_days).await;
            }
            let planned = collect_pass(&pool, &config).await.expect("collect");
            assert_eq!(planned.len(), 1);
            assert_eq!(planned[0].step, expected_step);
            assert!(
                claim_step(&pool, &planned[0].user_id, planned[0].step)
                    .await
                    .expect("claim")
            );
        }

        assert_eq!(
            steps_sent(&pool, user).await,
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
        let _org = seed_org(&pool, user).await;
        backdate_signup(&pool, user, 2).await;

        let config = test_config(NO_LIMIT);
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
            steps_sent(&pool, user).await.is_empty(),
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
        assert_eq!(next[0].user_id, user);
        assert_eq!(next[0].step, STEP_DAY1);
    }
}
