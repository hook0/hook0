# MR !233 — Code Review Consolidation

Retours de **CodeRabbit** (CR) et **David** (DS) avec propositions de corrections.

---

## 1. [DS] Limite `max_retries <= 25` dans la DB — trop rigide

**Fichier:** `api/migrations/20260325120000_add_retry_schedule.up.sql:11`
**Commentaire:** "C'est un peu rude de forcer une limite à 25 dans la DB. On a envie d'une limite mais c'est peut-être au niveau de l'API qu'on la voudrait ?"

**Analyse:** La contrainte CHECK en DB est irréversible sans migration. La validation API est plus flexible (changeable sans downtime).

**Proposition:** Retirer le `<= 25` du CHECK DB, garder uniquement `> 0`. Ajouter la validation `max_retries <= 25` côté API (handler Rust + schéma Zod frontend). Ça permet d'ajuster la limite par plan/tier sans migration.

```sql
-- Avant
max_retries integer not null check (max_retries > 0 and max_retries <= 25),
-- Après
max_retries integer not null check (max_retries > 0),
```

---

## 2. [DS] Nommage des index incohérent

**Fichier:** Toutes les migrations Phase 1-3
**Commentaire:** "le nommage des index a l'air de ne pas être cohérent avec les migrations précédentes"

**Analyse:** Convention historique = `{table}_{column}_idx` (ex: `request_attempt_event__id_idx`). Les nouvelles migrations utilisent le préfixe `idx_` (ex: `idx_subscription_health_event_sub_id`).

**Proposition:** Renommer tous les index des nouvelles migrations pour suivre la convention `{table}_{column}_idx` :

| Actuel | Proposé |
|--------|---------|
| `subscription_retry_schedule__id_idx` | OK — déjà conforme |
| `idx_subscription_health_event_sub_id` | `subscription_health_event_subscription__id_idx` |
| `idx_subscription_health_bucket_start` | `subscription_health_bucket_bucket_start_idx` |
| `idx_subscription_health_bucket_open` | `subscription_health_bucket_open_idx` |
| `idx_request_attempt_completed_at` | `request_attempt_completed_at_idx` |
| `idx_subscription_health_event_cleanup` | `subscription_health_event_cleanup_idx` |
| `idx_request_attempt_triggered_by` | `request_attempt_triggered_by_idx` |

---

## 3. [DS] SQL en minuscules dans les migrations

**Fichier:** `20260326120000_add_subscription_health.up.sql:68`, `20260403130000_add_attempt_trigger.up.sql:4-26`
**Commentaire:** "on est parti (à tord ou à raison) sur du SQL en minuscules pour les migrations (mais pas dans le code)"

**Analyse:** La convention historique est lowercase pour le DDL (`create table`, `alter table`, `create index`). Les instructions de session (`SET`/`RESET lock_timeout`) apparaissent en uppercase même dans les fichiers historiques — c'est un usage accepté.

**Proposition:** Passer le DDL en lowercase. Scope = DDL uniquement, `SET`/`RESET` restent en uppercase (convention établie).
- `20260326120000`: lignes 68-70 `CREATE INDEX IF NOT EXISTS` → `create index if not exists`, `ON` → `on`, `WHERE` → `where`
- `20260403130000`: lignes 4-5 `ALTER TABLE` → `alter table`, lignes 7-11 `ADD CONSTRAINT`/`CHECK`/`NOT VALID` → lowercase, ligne 14 `VALIDATE CONSTRAINT` → lowercase, lignes 18-19 `ADD COLUMN`/`REFERENCES` → lowercase, lignes 24-26 `CREATE INDEX`/`ON`/`WHERE` → lowercase

---

## 4. [DS] Requêtes dynamiques — préférer les macros SQLx

**Commentaire:** "j'aimerais beaucoup éviter de lancer des requêtes qui ne sont pas statiques/vérifiées à la compilation (via les macros SQLx)"

**Fichiers concernés:**
- `api/src/handlers/subscriptions.rs:812-818` — `query_scalar("SELECT is_enabled...")` dynamique
- `api/src/handlers/subscriptions.rs:973-982` — `query("INSERT INTO ... subscription_health_event...")` dynamique
- `api/src/handlers/subscriptions.rs:985-991` — `query("UPDATE ... SET failure_percent = NULL...")` dynamique

**Proposition:** Remplacer les `query()` / `query_scalar()` par `query!()` / `query_scalar!()` pour bénéficier de la vérification au compile-time.

**Prérequis:** Les macros `query!()` nécessitent soit un `DATABASE_URL` au compile-time, soit un cache offline (`.sqlx/`). L'infrastructure est déjà en place — le même fichier utilise `query!()` dès la ligne 663 et le dossier `.sqlx/` existe avec des query metadata.

---

## 5. [CR Critical] Pulsar failure ne doit pas remonter en erreur après commit

**Fichier:** `api/src/handlers/request_attempts.rs:693`
**Commentaire:** L'attempt est déjà committed en DB (ligne 677). Si Pulsar échoue, retourner une erreur pousse le client à re-tenter un endpoint non-idempotent, ce qui crée des duplicats.

**Proposition:** Remplacer `.await?` par un `match` qui log l'erreur et continue vers `202 Accepted` :

```rust
// Avant
.await?;

// Après
.await
{
    Ok(()) => {},
    Err(e) => {
        error!("Failed to dispatch manual retry {new_attempt_id} to Pulsar: {e}. The PG worker will pick it up via polling.");
    }
};
```

---

## 6. [CR Critical] Lock row avant toggle `is_enabled` — race condition avec health monitor

**Fichier:** `api/src/handlers/subscriptions.rs:812-818`
**Commentaire:** `previous_is_enabled` est lu sans lock. Le health monitor peut flipper la subscription entre le SELECT et l'INSERT du health event, ce qui produit un bogus event.

**Analyse:** Le `FOR UPDATE` fonctionne ici parce que le code tourne dans une transaction (`&mut *tx`). Le health monitor utilise aussi un pattern atomique dans `state_machine.rs:173-210`.

**Proposition:** Ajouter `FOR UPDATE` au SELECT :

```sql
-- Avant
SELECT is_enabled FROM webhook.subscription WHERE subscription__id = $1 AND deleted_at IS NULL
-- Après
SELECT is_enabled FROM webhook.subscription WHERE subscription__id = $1 AND application__id = $2 AND deleted_at IS NULL FOR UPDATE
```

Et passer `body.application_id` en bind supplémentaire.

---

## 7. [CR Major] `failure_percent` stale dans la réponse edit

**Fichier:** `api/src/handlers/subscriptions.rs:1042`
**Commentaire:** Après reset de `failure_percent = NULL` (lignes 985-991), la réponse sérialise encore `s.failure_percent` du RETURNING initial (ligne 828).

**Proposition:** Tracker si un toggle a eu lieu, et forcer `None` dans la réponse :

```rust
let toggled = body.is_enabled != previous_is_enabled.unwrap_or(body.is_enabled);
// ... plus tard dans la construction de Subscription :
failure_percent: if toggled { None } else { s.failure_percent },
```

---

## 8. [CR Major] Payload nullable — `query_scalar` peut fail sur NULL

**Fichier:** `api/src/handlers/request_attempts.rs:643-648`
**Commentaire:** `event.event.payload` est nullable (migration `20251013`), mais `query_scalar` + `fetch_optional` retourne `Option<T>` — si la row existe et payload est NULL, SQLx tente de décoder NULL dans `Vec<u8>`, ce qui produit une erreur de décodage.

**Proposition:** Utiliser le type hint explicite :

```rust
let db_payload: Option<Vec<u8>> =
    sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT payload FROM event.event WHERE event__id = $1")
        .bind(&source.event_id)
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?
        .flatten();
```

---

## 9. [CR Major] `user__id` non renseigné sur health events manuels

**Fichier:** `api/src/handlers/subscriptions.rs:979`
**Commentaire:** Le INSERT du health event passe `None::<Uuid>` pour `user__id`, même quand c'est une action utilisateur (`source = 'user'`). L'auth token est disponible dans le handler.

**Proposition:** Passer le `user_id` extrait de l'auth context au lieu de `None::<Uuid>`.

---

## 10. [CR Minor] Commentaire stale — sentinel retention

**Fichier:** `frontend/src/pages/organizations/applications/logs/LogList.vue:39`

**Proposition:**
```typescript
// Avant
// API uses INT32_MAX (2147483647) as sentinel for "unlimited retention"
// Après
// Treat 10+ years (3650 days) as "unlimited retention"
```

---

## 11. [CR Minor] JSDoc stale sur `formatRelativeTime`

**Fichier:** `frontend/src/utils/formatDate.ts:21`

**Proposition:**
```typescript
// Avant
* Format a future date string as a human-readable countdown (e.g. "5m", "1h30m").
// Après
* Format a future date string as a human-readable countdown (e.g. "5min", "1h 30min").
```

---

## 12. [CR] Placeholder templates email — faux positif, MAIS `{ $application_url }` jamais remplacé

**Fichier:** `api/src/mail_templates/subscriptions/disabled.mjml:51` + `warning.mjml` + `recovered.mjml`

**Analyse CodeRabbit originale (faux positif) :** Le mix `{ $logo_url }` / `{{ $subscription_description }}` est **intentionnel** — `mailer.rs` fait deux passes de remplacement :
1. **Pass 1** (ligne 325) : `{{ $key }}` pour les variables domaine (via `self.variables()`)
2. **Pass 2** (lignes 328-335) : `{ $key }` pour les variables infra hardcodées (`logo_url`, `subscription_url`, `website_url`, `app_url`, `support_email_address`)

**Bug réel découvert :** `{ $application_url }` apparaît dans les 3 templates (lien sur le nom de l'app), mais **n'existe dans aucune des deux passes** de `render()`. Le lien `<a href="{ $application_url }">` reste en dur dans le HTML envoyé — **lien cassé dans tous les emails de subscription**.

**Sévérité : Critical**

**Proposition:** Ajouter le remplacement dans `mailer.rs:render()`, entre les lignes 330-331 :

```rust
if let Some(application_url) = self.application_url(app_url) {
    mjml = mjml.replace("{ $application_url }", &application_url);
}
```

Et ajouter la méthode `application_url()` sur `Mail` qui construit l'URL comme `{app_url}/organizations/{org_id}/applications/{app_id}`.

---

## 13. [Review #1] DELETE FROM subscription__worker dupliqué

**Fichier:** `api/src/handlers/subscriptions.rs:896-905` et `943-952`

**Analyse:** Deux blocs identiques `DELETE FROM webhook.subscription__worker WHERE subscription__id = $1`. Le premier (ligne 896) est toujours écrasé par le second (ligne 943) juste avant le re-INSERT des workers. Le premier est du dead code.

**Sévérité : Minor** (code quality, pas de bug)

**Proposition:** Supprimer le premier DELETE (lignes 896-905). Seul le second (avant le re-INSERT) est nécessaire.

---

## 14. [DS] Questions design (pas de correction code, à discuter)

1. **Schedule par défaut application-level ?** — Pas implémenté. Ajouter un champ `default_retry_schedule__id` sur `event.application` ?
2. **Protection contre schedules dangereux ?** — Le CHECK DB empêche `linear_delay < 1s`. Côté API, ajouter un minimum plus agressif (ex: 60s min pour custom, 30s min pour linear).
3. **Volume d'emails pour gros clients ?** — Implémenter un rate-limit / digest (1 email max par subscription par heure ? quotidien ?).
4. **Limite des manual retries ?** — Ajouter un rate-limit (ex: max 10 retries manuels par subscription par heure).
5. **Utilisateurs finaux et schedules ?** — Hors scope Phase 1. Dashboard embeddings créeraient les schedules via l'API du client, pas directement.

---

## Résumé des actions

| # | Sévérité | Type | Status |
|---|----------|------|--------|
| 1 | Major | Migration | À corriger |
| 2 | Minor | Migration | À corriger |
| 3 | Minor | Migration | À corriger |
| 4 | Major | Rust | À corriger |
| 5 | Critical | Rust | À corriger |
| 6 | Critical | Rust | À corriger |
| 7 | Major | Rust | À corriger |
| 8 | Major | Rust | À corriger |
| 9 | Major | Rust | À corriger |
| 10 | Minor | Vue | À corriger |
| 11 | Minor | TS | À corriger |
| 12 | Critical | Rust/MJML | À corriger |
| 13 | Minor | Rust | À corriger |
| 14 | — | Design | Discussion |
