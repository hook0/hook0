# Manual Retry — Implementation Plan v2

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extraire la phase 3 (manual retry) de `feat/retry-schedule` vers une branche standalone prête à merger.

**Spec:** `docs/superpowers/specs/2026-04-10-manual-retry-v2.md`

**Tech Stack:** Rust (actix-web, sqlx macros, prost), PostgreSQL, Protobuf, Vue 3, TanStack Query, k6

**Contrainte SQLX** : toutes les requêtes SQL DOIVENT utiliser les macros `sqlx::query!` / `sqlx::query_as!` / `sqlx::query_scalar!`. Pas de fonctions runtime.

---

## Préparation : créer la branche

```bash
git checkout master
git pull
git checkout -b feat/manual-retry
```

Cherry-pick ou ré-implémenter depuis `feat/retry-schedule`. Les commits source sont identifiés par tâche ci-dessous.

---

## Task 1: Migration `attempt_trigger`

**Files:**
- Create: `api/migrations/20260403130000_add_attempt_trigger.up.sql`
- Create: `api/migrations/20260403130000_add_attempt_trigger.down.sql`

**Commits source:** `3877215e`

- [ ] **Step 1:** Écrire la migration up (voir spec — colonnes `attempt_trigger`, `triggered_by`, index partiel)
- [ ] **Step 2:** Écrire la migration down
- [ ] **Step 3:** Commit: `feat(db): add attempt_trigger and triggered_by columns to request_attempt`

---

## Task 2: Protobuf `AttemptTrigger`

**Files:**
- Modify: `protobuf/proto/request_attempt.proto`
- Modify: `protobuf/src/request_attempt.rs`

**Commits source:** `57a207c2`, `58b295e7`

- [ ] **Step 1:** Ajouter `string attempt_trigger = 15;` dans le .proto
- [ ] **Step 2:** Ajouter l'enum `AttemptTrigger` (Dispatch/AutoRetry/ManualRetry) avec `Display`/`FromStr` et fallback `Dispatch`
- [ ] **Step 3:** Ajouter le champ `attempt_trigger: AttemptTrigger` au struct `RequestAttempt` + `TryFrom` avec `is_empty()` guard
- [ ] **Step 4:** Propager dans tous les sites de construction du struct (`SerializeMessage`, etc.)
- [ ] **Step 5:** `cargo build -p hook0-protobuf`
- [ ] **Step 6:** Commit: `feat(protobuf): add attempt_trigger field to RequestAttempt message`

---

## Task 3: Extract `fetch_event_payload`

**Files:**
- Create: `api/src/event_payload.rs`
- Modify: `api/src/handlers/events.rs` (refacto replay)
- Modify: `api/src/main.rs` (add `mod event_payload;`)

**Commits source:** `c5eab8d7`

- [ ] **Step 1:** Extraire la logique de fetch payload (DB + S3 fallback) depuis `events.rs` vers `event_payload.rs`
- [ ] **Step 2:** Refactorer le handler `replay` pour utiliser `fetch_event_payload`
- [ ] **Step 3:** Enregistrer le module dans `main.rs`
- [ ] **Step 4:** `cargo build -p hook0-api`
- [ ] **Step 5:** Commit: `refactor(api): extract fetch_event_payload into shared module`

---

## Task 4: IAM + error variants

**Files:**
- Modify: `api/src/iam.rs`
- Modify: `api/src/problems.rs`

**Commits source:** `808c6492`, parties de `43bfff4a`

- [ ] **Step 1:** Ajouter `RequestAttemptRetry { application_id }` dans `Action` enum
- [ ] **Step 2:** Ajouter `action_name()`, role mappings, biscuit policy (suivre pattern `EventReplay`)
- [ ] **Step 3:** Ajouter `EventPayloadUnavailable` dans problems → **410 Gone**
- [ ] **Step 4:** `cargo build -p hook0-api`
- [ ] **Step 5:** Commit: `feat(api): add RequestAttemptRetry IAM action and EventPayloadUnavailable error`

---

## Task 5: Configuration cooldown

**Files:**
- Modify: `api/src/main.rs` (struct `ApiArgs`)

- [ ] **Step 1:** Ajouter le champ dans la struct de config API :
  ```rust
  /// Cooldown in seconds between manual retries of the same event (0 = disabled)
  #[arg(long, env = "MANUAL_RETRY_COOLDOWN_SECONDS", default_value = "60")]
  pub manual_retry_cooldown_seconds: u64,
  ```
- [ ] **Step 2:** `cargo build -p hook0-api`
- [ ] **Step 3:** Commit: `feat(api): add MANUAL_RETRY_COOLDOWN_SECONDS config`

---

## Task 6: Handler `retry`

**Files:**
- Modify: `api/src/handlers/request_attempts.rs`
- Modify: `api/src/main.rs` (route registration)

**Commits source:** `43bfff4a`

**⚠️ SQLX macros obligatoires** — `query_as!`, `query_scalar!`, pas de fonctions runtime.

- [ ] **Step 1:** Ajouter le handler `retry` :
  1. Fetch source attempt metadata (`query_as!`)
  2. Authorize → 404 uniforme
  3. Check subscription non supprimée
  4. Check cooldown par event (`query_scalar!`) :
     ```sql
     SELECT EXISTS(
       SELECT 1 FROM webhook.request_attempt
       WHERE event__id = $1
         AND attempt_trigger = 'manual_retry'
         AND created_at > now() - make_interval(secs => $2::float8)
     )
     ```
     Si exists → 429
  5. Fetch payload via `fetch_event_payload`
  6. Extract `user_id` du token
  7. INSERT new attempt (`query_scalar!`)
  8. Dispatch Pulsar (optionnel, hors tx)
  9. Return 202

- [ ] **Step 2:** Ajouter `attempt_trigger` au SELECT du handler `list` (`query_as!`)
- [ ] **Step 3:** Enregistrer la route dans `main.rs`
- [ ] **Step 4:** Regenerate sqlx offline cache : `cargo sqlx prepare --workspace`
- [ ] **Step 5:** `cargo build -p hook0-api`
- [ ] **Step 6:** Commit: `feat(api): add POST /request_attempts/{id}/retry with per-event cooldown`

---

## Task 7: Worker one-shot + auto_retry

**Files:**
- Modify: `output-worker/src/pg.rs`
- Modify: `output-worker/src/pulsar.rs`

**Commits source:** parties de `43bfff4a`

- [ ] **Step 1:** Ajouter `attempt_trigger` au SELECT de `look_for_work` et au struct
- [ ] **Step 2:** One-shot check **avant** `compute_next_retry` (PG path) :
  ```rust
  if attempt.attempt_trigger == AttemptTrigger::ManualRetry {
      info!("Manual retry failed; not re-queuing (one-shot)");
  } else if let Some(retry_in) = compute_next_retry(...) {
      // INSERT successor avec attempt_trigger = 'auto_retry'
  }
  ```
- [ ] **Step 3:** Même one-shot check dans `pulsar.rs`
- [ ] **Step 4:** Ajouter `attempt_trigger = 'auto_retry'` dans l'INSERT des successors
- [ ] **Step 5:** Propager `attempt_trigger` dans la construction protobuf
- [ ] **Step 6:** `cargo build -p hook0-output-worker && cargo test -p hook0-output-worker`
- [ ] **Step 7:** Commit: `feat(output-worker): one-shot for manual_retry, auto_retry on successors`

---

## Task 8: Frontend — bouton retry + badge

**Files:**
- Modify: `frontend/src/pages/organizations/applications/logs/DeliverySplitView.vue`
- Modify: `frontend/src/pages/organizations/applications/logs/LogService.ts`
- Modify: `frontend/src/pages/organizations/applications/logs/useLogQueries.ts`
- Modify: `frontend/src/pages/organizations/applications/logs/useLogColumns.ts` (badge)
- Modify: `frontend/src/locales/en.json`

**Commits source:** `41f16458`, `300f8b9f`

- [ ] **Step 1:** Ajouter `retry()` dans `LogService.ts`
- [ ] **Step 2:** Ajouter `useRetryDelivery` mutation dans `useLogQueries.ts`
- [ ] **Step 3:** Ajouter le bouton retry dans `DeliverySplitView.vue` :
  - Icône `RefreshCw`
  - `disabled: retryMutation.isPending`
  - Toast success / error (410, 429)
- [ ] **Step 4:** Ajouter le badge "Retry" dans `useLogColumns.ts` pour `attempt_trigger === 'manual_retry'`
- [ ] **Step 5:** Ajouter les clés i18n (retryDelivery, retryQueued, payloadExpired, retryCooldown, manualRetryBadge)
- [ ] **Step 6:** `cd frontend && npx vue-tsc --noEmit`
- [ ] **Step 7:** Commit: `feat(frontend): add manual retry button and badge on deliveries`

---

## Task 9: Tests k6

**Files:**
- Create: `k6/tests/manual-retry.js` (ou intégrer dans l'existant)

- [ ] **Step 1:** Test happy path : retry attempt → 202, vérifier `attempt_trigger = 'manual_retry'`
- [ ] **Step 2:** Test cooldown : retry 2x même event < 1 min → 429
- [ ] **Step 3:** Test payload expiré → 410
- [ ] **Step 4:** Test authz : retry attempt d'une autre app → 404
- [ ] **Step 5:** Test subscription supprimée → 404
- [ ] **Step 6:** Commit: `test: add k6 integration tests for manual retry`

---

## Task 10: Test unitaire Rust — one-shot

**Files:**
- Modify: `output-worker/src/pg.rs` ou `output-worker/src/pulsar.rs` (tests module)

- [ ] **Step 1:** Test : worker reçoit un attempt avec `attempt_trigger = ManualRetry`, échec delivery → pas de successor INSERT
- [ ] **Step 2:** `cargo test -p hook0-output-worker`
- [ ] **Step 3:** Commit: `test(output-worker): verify manual retry is one-shot`

---

## Execution Order

```
Task 1 (migration)
  → Task 2 (protobuf)
    → Task 3 (extract event_payload)
      → Task 4 (IAM + errors)
        → Task 5 (config cooldown)
          → Task 6 (handler retry)
            → Task 7 (worker one-shot)
            → Task 8 (frontend)  ← parallélisable avec Task 7
              → Task 9 (tests k6) 
              → Task 10 (test unit)
```

Avec 2 agents : Agent 1 fait Tasks 1→6→9, Agent 2 fait Tasks 2→7→10 (après Task 2), puis Task 8 après Task 6.

---

## Checklist pré-MR

- [ ] Toutes les requêtes SQL utilisent des macros SQLX (`!`)
- [ ] `cargo build --workspace` passe
- [ ] `cargo test --workspace` passe
- [ ] `npx vue-tsc --noEmit` passe
- [ ] sqlx offline cache régénéré
- [ ] Pas de dépendance sur retry schedules ou health monitor
- [ ] Pas de subscription detail page
- [ ] Variable `MANUAL_RETRY_COOLDOWN_SECONDS` documentée dans docker-compose
