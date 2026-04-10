# Manual Retry — Design Spec v2

**Date**: 2026-04-10
**Feature**: Manual retry of webhook deliveries (phase 3, standalone MR)
**Branch**: à créer depuis `master`
**Supersedes**: `2026-04-03-manual-retry-design.md`

---

## Contexte

Split de la MR !233 en 3 branches distinctes (weekly du 8 avril). Cette spec couvre **uniquement** la phase 3 (manual retry), sans dépendance aux phases 1 (retry schedules) ni 2 (health monitor).

## Problème

Quand une livraison webhook échoue, l'utilisateur ne peut pas la relancer manuellement. C'est le #1 des demandes utilisateurs et le cas le plus simple à shipper en premier.

## Solution

Endpoint `POST /request_attempts/{id}/retry` qui crée une nouvelle tentative one-shot. Bouton retry dans la page logs existante (`DeliverySplitView`).

---

## Scope

### Inclus
- Migration `attempt_trigger` + `triggered_by`
- Handler `retry` dans `request_attempts.rs`
- Cooldown configurable par event (1 retry/min par défaut)
- Protobuf `AttemptTrigger` (champ 15)
- Worker one-shot (PG + Pulsar)
- Extract `fetch_event_payload` (refacto replay)
- IAM action `RequestAttemptRetry`
- Error variant `EventPayloadUnavailable` (410 Gone)
- Frontend : bouton retry dans `DeliverySplitView`, badge "Retry", mutation
- Tests k6 + unit Rust

### Exclus
- Retry schedules (phase 1)
- Health monitor / auto-deactivation (phase 2)
- Subscription detail page
- Health badges / health timeline
- Email templates de santé

---

## API

### Endpoint

```
POST /api/v1/request_attempts/{request_attempt_id}/retry
```

### Guards (dans l'ordre)

1. Fetch attempt metadata (event_id, subscription_id, application_id) — **sans** payload
2. Authorize `RequestAttemptRetry` scoped à l'application
3. **404 uniforme** pour not-found ET forbidden (anti-énumération)
4. Subscription non supprimée (`deleted_at IS NULL`)
5. **Cooldown** : pas de manual_retry sur le même **event** depuis moins de `MANUAL_RETRY_COOLDOWN_SECONDS`
6. Payload disponible (DB ou S3) — sinon **410 Gone**

### Pas de guard sur

- **Status de l'attempt** : retry sur failed ET success (re-livraison volontaire)
- **Attempts in-flight** : le cooldown suffit, pas de blocage sur pending
- **Subscription désactivée** : l'attempt restera "Pending" jusqu'à ré-activation

### Cooldown

```sql
SELECT EXISTS(
  SELECT 1 FROM webhook.request_attempt
  WHERE event__id = $1
    AND attempt_trigger = 'manual_retry'
    AND created_at > now() - make_interval(secs => $2)
)
```

- `$2` = `MANUAL_RETRY_COOLDOWN_SECONDS` (env var, défaut `60`, `0` = désactivé)
- Si exists → **429 Too Many Requests** avec message : "A manual retry for this event was already triggered less than {N}s ago"
- Le check est par **event** (pas par attempt) : empêche le contournement en retryant chaque nouveau retry

### Comportement

1. Fetch source attempt (metadata only)
2. Authorize (404 on failure)
3. Check subscription non supprimée
4. Check cooldown par event
5. Fetch event payload (DB ou S3 fallback via `fetch_event_payload`)
6. Extract `user_id` du token (`Some` pour user, `None` pour service token)
7. INSERT new `request_attempt` :
   - `retry_count = 0`, `delay_until = NULL` (immédiat)
   - `attempt_trigger = 'manual_retry'`
   - `triggered_by = user_id`
8. Dispatch Pulsar (optionnel, hors transaction — PG poller = filet de sécurité)
9. **202 Accepted** dans tous les cas si INSERT réussi (même si Pulsar down)

### Response

```json
{ "request_attempt_id": "<uuid>" }
```

### Codes d'erreur

| Code | Cas |
|------|-----|
| 202 | Retry créé |
| 404 | Attempt/subscription introuvable ou non autorisé |
| 410 | Payload expiré / purgé |
| 429 | Cooldown pas expiré |

---

## Configuration

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `MANUAL_RETRY_COOLDOWN_SECONDS` | u64 | `60` | Cooldown entre deux manual retry du même event. `0` = désactivé. |

Vit dans les args CLI de l'API (struct `ApiArgs` dans `main.rs`), à côté des autres configs.

---

## Migration

### `20260403130000_add_attempt_trigger.up.sql`

```sql
SET lock_timeout = '5s';

-- Column avec DEFAULT (instant PG 11+, catalog-only)
ALTER TABLE webhook.request_attempt
  ADD COLUMN attempt_trigger TEXT NOT NULL DEFAULT 'dispatch';

-- CHECK NOT VALID (pas de table scan)
ALTER TABLE webhook.request_attempt
  ADD CONSTRAINT request_attempt_trigger_check
    CHECK (attempt_trigger IN ('dispatch', 'auto_retry', 'manual_retry'))
    NOT VALID;

-- Validation séparée (SHARE UPDATE EXCLUSIVE, DML concurrent OK)
ALTER TABLE webhook.request_attempt
  VALIDATE CONSTRAINT request_attempt_trigger_check;

-- FK nullable pour audit (NULL = system/service token)
ALTER TABLE webhook.request_attempt
  ADD COLUMN triggered_by UUID REFERENCES iam.user ON DELETE SET NULL;

-- Index partiel : évite full scan sur DELETE iam.user
CREATE INDEX idx_request_attempt_triggered_by
  ON webhook.request_attempt (triggered_by)
  WHERE triggered_by IS NOT NULL;

RESET lock_timeout;
```

### Down

```sql
SET lock_timeout = '5s';
DROP INDEX IF EXISTS webhook.idx_request_attempt_triggered_by;
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS triggered_by;
ALTER TABLE webhook.request_attempt DROP CONSTRAINT IF EXISTS request_attempt_trigger_check;
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS attempt_trigger;
RESET lock_timeout;
```

---

## Déploiement

**Worker + API + migration ensemble** (même `kubectl apply`).

Raison : si un vieux worker reçoit un message manual_retry, il ne voit pas `attempt_trigger` (proto3 ignore unknown fields), le traite comme `Dispatch`, et crée des auto-retry successors → pas one-shot. Pas catastrophique mais pas le comportement voulu.

---

## Worker

### One-shot

Dans `pg.rs` et `pulsar.rs`, **avant** `compute_next_retry` :

```rust
if attempt.attempt_trigger == AttemptTrigger::ManualRetry {
    info!("Manual retry failed; not re-queuing (one-shot)");
} else if let Some(retry_in) = compute_next_retry(...) {
    // INSERT successor avec attempt_trigger = 'auto_retry'
}
```

### Successor INSERT

Ajouter `attempt_trigger = 'auto_retry'` dans l'INSERT des successors (PG et Pulsar paths).

---

## Protobuf

Champ `string attempt_trigger = 15` dans `RequestAttempt`. Forward-compatible :
- Vieux worker sans champ 15 → prost ignore → default `Dispatch`
- Valeur inconnue → `FromStr` fallback `Dispatch`
- Champ vide → `is_empty()` → default `Dispatch`

---

## Refacto : extract `fetch_event_payload`

Extract depuis `events.rs` (handler replay) vers `api/src/event_payload.rs`. Réutilisé par replay (refactoré) et retry (nouveau). Commit séparé.

---

## SQLX : macros obligatoires

**Toutes** les requêtes du handler retry doivent utiliser `sqlx::query!` / `sqlx::query_as!` / `sqlx::query_scalar!` (macros avec `!`). Pas de `sqlx::query()` / `sqlx::query_as::<_, T>()` (fonctions runtime).

Raison : vérification à la compilation, prévention injection SQL (demande explicite de David, weekly 8 avril).

---

## Frontend

### Bouton retry

- **Où** : dans `DeliverySplitView.vue` (page logs existante, pas subscription detail)
- **Icône** : `RefreshCw` (lucide-vue-next)
- **Visible sur** : toutes les tentatives (failed ET success)
- **Disabled** : pendant `retryMutation.isPending` (anti spam-clic)
- **On success** : toast + `invalidateQueries`
- **On 410** : toast error "payload expiré"
- **On 429** : toast error "cooldown"

### Badge "Retry"

Sur les lignes avec `attempt_trigger === 'manual_retry'` : `Hook0Badge variant="info" size="sm"` avec texte "Retry".

### i18n

```json
"retryDelivery": "Retry this delivery",
"retryQueued": "Retry queued",
"payloadExpired": "Cannot retry — event payload has expired",
"retryCooldown": "Please wait before retrying this event again",
"manualRetryBadge": "Retry"
```

---

## Tests

### k6 (intégration API)

| # | Scénario | Assertion |
|---|----------|-----------|
| 1 | Retry un attempt failed | 202, nouveau attempt avec `attempt_trigger = 'manual_retry'` |
| 2 | Retry 2x le même event en < 1 min | 1er → 202, 2ème → 429 |
| 3 | Retry avec payload purgé | 410 Gone |
| 4 | Retry un attempt d'une autre application | 404 |
| 5 | Retry un attempt avec subscription supprimée | 404 |

### Unit Rust (output-worker)

| # | Scénario | Assertion |
|---|----------|-----------|
| 6 | Worker reçoit manual_retry failed | Pas de successor INSERT (one-shot) |

---

## Decisions log

| # | Décision | Choix | Raison |
|---|----------|-------|--------|
| 1 | Scope | Phase 3 seule, sans detail page | Split MR (weekly 8 avril) |
| 2 | Cooldown | 1x/min par event, configurable env var | David (weekly) : "limiter à 1x/min par tentative de requête" |
| 3 | Config | `MANUAL_RETRY_COOLDOWN_SECONDS`, défaut 60, 0 = off | Simple, cohérent avec les autres config |
| 4 | SQLX | Macros uniquement (`!`) | David (weekly) : vérification compilation, anti-injection |
| 5 | Déploiement | Worker + API ensemble | Proto forward-compatible mais one-shot perdu si vieux worker |
| 6 | Payload expiré | 410 Gone, bouton visible, erreur parle | Pas d'info expiration côté frontend sans requête extra |
| 7 | In-flight | Pas de blocage | Cooldown suffit, blocage fragile si worker mort |
| 8 | Retry tout état | Oui (failed + success) | Re-livraison volontaire légitime |
| 9 | Frontend | Disable pendant isPending | Anti double-clic, 1 ligne |
| 10 | event_payload extract | Inclus, commit séparé | Dédup propre, David appréciera |
| 11 | Tests | k6 (5 cas) + unit Rust (1 cas) | Couverture complète du happy path et edge cases |
| 12 | Retry button placement | DeliverySplitView (logs) | Pas de subscription detail page dans cette MR |

---

## Risques

Aucune question ouverte identifiée lors du design review (2026-04-10).
