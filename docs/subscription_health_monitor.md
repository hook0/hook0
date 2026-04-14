# Subscription Health Monitor — call graphs

Two recurring jobs live inside the `subscription_health_monitor` module, both
driven by the background loop in [`mod.rs`](../api/src/subscription_health_monitor/mod.rs).
Each graph below shows the functions called in order, the folder they live
in (box label), and a short sentence on what they do.

## 1. `run_health_check` — every `interval` (default 30m), chained while the scan cap is hit

The tick is a **snapshot → plan → apply** loop. The heavy lifting (cursor,
bucket lifecycle, candidate selection, failure-rate compute) is hidden inside
`snapshot_subscription_healths`; the runner only sees three steps. For the
internal pipeline detail, see the appendix below.

```
 ┌────────── subscription_health_monitor/mod.rs ────────────────────────────┐
 │  run_subscription_health_monitor                                         │
 │    └─ wakes on the housekeeping semaphore, loops ticks, sleeps `interval`│
 │       between wake-ups; chains up to MAX_CHAINED_TICKS=10 ticks when     │
 │       run_health_check reports hit_cap=true (backlog catch-up)           │
 └───────────────────────────────┬──────────────────────────────────────────┘
                                 │
                                 ▼
 ┌─────────── subscription_health_monitor/runner.rs ────────────────────────┐
 │  run_health_check                                                        │
 │    ├─ db.begin() + `set local statement_timeout = '5min'`                │
 │    ├─ pg_try_advisory_xact_lock(ADVISORY_LOCK_ID)                        │
 │    │     └─ gates replicas: only one API instance runs the tick at once  │
 │    │                                                                     │
 │    ├─ 1. snapshot_subscription_healths(tx, config)                       │
 │    │     └─ returns Vec<SubscriptionHealth> + hit_cap. Details in the    │
 │    │        "Snapshot pipeline" appendix below                           │
 │    │                                                                     │
 │    ├─ 2. for each subscription:                                          │
 │    │     state_machine::plan_health_actions(subscription, config, now)   │
 │    │       └─ pure function: compares failure rate against thresholds +  │
 │    │          last event, returns Vec<PlannedAction>                     │
 │    │                                                                     │
 │    ├─ 3. for each subscription:                                          │
 │    │     apply_planned_actions(tx, subscription, actions)                │
 │    │       └─ dispatches each PlannedAction to the right queries::* fn   │
 │    │          (update_subscription_failure_percent / insert_health_event │
 │    │          / disable_subscription)                                    │
 │    │                                                                     │
 │    └─ transaction.commit()                                               │
 │          └─ atomically publishes bucket writes, event rows, cursor       │
 │             advance, and failure_percent updates produced by this tick   │
 └──────────────────────────────────────────────────────────────────────────┘
```

## 2. Cleanup — once every `CLEANUP_INTERVAL` (24h) inside the same loop

The cleanup pass runs on a separate timer from the tick: every wake-up, the
loop checks whether 24 hours have passed since the last cleanup and, if so,
fires two independent maintenance queries. These do NOT run inside the tick
transaction — they're pure maintenance queries against a live DB pool.

```
 ┌────── subscription_health_monitor/mod.rs ────────────────────────────────┐
 │  run_subscription_health_monitor (loop body)                             │
 │    └─ checks `last_cleanup.elapsed() > CLEANUP_INTERVAL` (24h)           │
 │       and calls log_cleanup_result around each query below               │
 │                                                                          │
 │  log_cleanup_result                                                      │
 │    └─ uniform info/debug/warn log wrapper around one cleanup result      │
 └───────────────────────────────┬──────────────────────────────────────────┘
                                 │
           ┌─────────────────────┴──────────────────────┐
           │                                            │
           ▼                                            ▼
 ┌── subscription_health_monitor/queries/events.rs ──┐  ┌── queries/buckets.rs ──┐
 │  cleanup_resolved_health_events                   │  │  cleanup_old_buckets    │
 │    └─ deletes rows with status='resolved' older   │  │    └─ deletes every     │
 │       than resolved_event_retention, BUT only if  │  │       bucket (open or   │
 │       a newer event exists for the same           │  │       closed) whose     │
 │       subscription. Warning and disabled events   │  │       bucket_start is   │
 │       are never purged; the latest event per      │  │       older than        │
 │       subscription is always kept so the current  │  │       bucket_retention  │
 │       state stays visible                         │  │                         │
 └───────────────────────────────────────────────────┘  └─────────────────────────┘
```

## State machine transitions (what `plan_health_actions` decides)

```
   ┌─────────┐  failure ≥ warning & < disable   ┌─────────┐
   │  None   │─────────────────────────────────▶│ Warning │
   └────┬────┘                                  └────┬────┘
        │                                            │
        │ failure ≥ disable                          │ failure < warning
        ▼                                            ▼
   ┌──────────┐                                 ┌──────────┐
   │ Disabled │◀──── failure ≥ disable ─────────│ Resolved │
   └──────────┘                                 └──────────┘

   Disabled                    → do nothing (user must re-enable manually).
   Resolved + within anti-flap → do nothing (avoids warning↔resolved
                                 oscillations in the audit trail when the
                                 failure rate hovers around the threshold).
```

## Transaction + concurrency invariants (cross-cutting)

- One `run_health_check` invocation runs inside **one** Postgres transaction
  so a crash at any step rolls the whole tick back (no partial bucket writes,
  no partial event writes, no partial cursor advance).
- `pg_try_advisory_xact_lock` gates replicas: only one API instance executes
  a tick at a given moment. Others return early and retry on their next
  wake-up.
- `housekeeping_semaphore = Semaphore::new(1)` sequentializes **all**
  housekeeping tasks (health monitor + others) in the same process. The
  permit is `drop`'d **before** `sleep(interval)` so peer housekeeping tasks
  can run during the sleep window.
- `set local statement_timeout = '5min'` caps a single tick's DB time so a
  pathological query can never freeze the housekeeping loop indefinitely.

---

## Appendix — Snapshot pipeline internals

`snapshot_subscription_healths` (in `subscription_health_monitor/evaluation/mod.rs`)
is the pre-aggregation layer that makes each tick fast even on instances with
millions of `request_attempt` rows. Rather than scanning the raw table on
every tick, it keeps a **bucketed** rolling aggregate that amortizes the read
cost across ticks.

```
 ┌────── queries/cursor.rs ─────────────────────────────────────────────────┐
 │  read_evaluation_cursor                                                  │
 │    └─ reads the singleton bookmark "where did I stop last tick?"         │
 └───────────────────────────────┬──────────────────────────────────────────┘
                                 ▼
 ┌────── queries/deltas.rs ─────────────────────────────────────────────────┐
 │  aggregate_recent_request_attempts                                       │
 │    └─ scans webhook.request_attempt for rows completed after the cursor, │
 │       capped at max_request_attempts_per_tick, groups them per           │
 │       subscription (total vs failed)                                     │
 └───────────────────────────────┬──────────────────────────────────────────┘
                                 ▼
 ┌────── queries/buckets.rs ────────────────────────────────────────────────┐
 │  upsert_buckets                                                          │
 │    └─ adds the new aggregate counts to each subscription's currently-    │
 │       open bucket (one UNNEST INSERT ... ON CONFLICT DO UPDATE)          │
 │                                                                          │
 │  close_full_buckets                                                      │
 │    └─ freezes buckets that exceeded bucket_duration OR                   │
 │       bucket_max_messages; a closed bucket is immutable                  │
 └───────────────────────────────┬──────────────────────────────────────────┘
                                 ▼
 ┌── queries/subscription_state.rs ─────────────────────────────────────────┐
 │  compute_candidate_healths                                               │
 │    └─ ONE query that picks candidates AND computes their rates:          │
 │         candidates = subs with enough recent failures                    │
 │                    ∪ subs currently in `warning` state                   │
 │       (the warning UNION is critical — a warned sub that stopped         │
 │       failing would otherwise never get a `resolved` event).             │
 │       Returns Vec<SubscriptionHealth> with the latest health event       │
 │       joined in via LEFT JOIN LATERAL.                                   │
 │                                                                          │
 │  reset_healthy_failure_percent                                           │
 │    └─ clears webhook.subscription.failure_percent for every row that     │
 │       is NOT in the current candidate set. Prevents the API's cached     │
 │       rate from going stale on subs that recovered and are no longer     │
 │       evaluated this tick                                                │
 └───────────────────────────────┬──────────────────────────────────────────┘
                                 ▼
 ┌────── queries/cursor.rs ─────────────────────────────────────────────────┐
 │  advance_evaluation_cursor                                               │
 │    └─ moves the singleton bookmark forward so the next tick skips the    │
 │       rows we just processed                                             │
 └──────────────────────────────────────────────────────────────────────────┘
```

### Why the bucket layer exists (for the curious)

The bucket table + cursor combo looks like ceremony, but it exists because
`webhook.request_attempt` is designed to be **massive** — the existing BRIN
index `request_attempt_created_at_idx` (space-optimized for tables with
tens of millions of rows) is the smoking gun. A naive approach of running
`SELECT COUNT(*) FROM request_attempt WHERE created_at > now() - interval
'1h' GROUP BY subscription__id` on every tick would:
- read tens of millions of rows per tick on busy instances;
- fight the existing BRIN index (which is chosen precisely to avoid a huge
  btree), forcing either a seq scan or a new expensive btree;
- miss rows that complete asynchronously — `request_attempt` rows are
  inserted at send time but `succeeded_at` / `failed_at` are set later, so
  filtering on `created_at` misses retries of old rows.

The bucket layer solves all three at once: each `request_attempt` is counted
**exactly once** into its bucket (cursor-based scan), the bucket is read
cheaply on every subsequent tick (bucket_stats query hits a small table),
and `COALESCE(succeeded_at, failed_at) > cursor` captures asynchronous
completions correctly.
