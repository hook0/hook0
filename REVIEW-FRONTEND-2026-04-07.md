# Frontend Code Review — `feat/retry-schedule` vs `master`

**Date:** 2026-04-07
**Branch:** feat/retry-schedule
**Scope:** 31 fichiers frontend modifiés (~2652 lignes ajoutées, 96 supprimées)
**Reviewers:** 5 agents indépendants, résultats agrégés et dédupliqués

---

## Statistiques

| Sévérité | Count |
|----------|-------|
| BUG      | 5     |
| FIX      | 12    |
| CONSIDER | 14    |
| NIT      | 13    |
| QUESTION | 1     |

---

## BUG — Bloquants

### B1. Hook0Slider: pas de `@blur` sur l'input d'édition inline
**Fichier:** `frontend/src/components/Hook0Slider.vue:93-98`
**Détecté par:** Agent 4

L'input d'édition inline n'a pas de handler `@blur`. Si l'utilisateur clique ailleurs sans appuyer Enter ou Escape, l'input reste ouvert indéfiniment (`isEditing` reste `true`).

**Fix:** Ajouter `@blur="confirmEdit"` sur l'input d'édition.

---

### B2. RetrySchedulesEdit: bouton submit sans loading ni disabled pendant mutation
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue:484`
**Détecté par:** Agents 1, 2, 3, 4, 5 (unanime)

Le bouton submit a `:disabled="hasExceedingRetries"` mais ne vérifie pas `isPending` des mutations. Risque de double-soumission.

**Fix:** Ajouter `:loading="createMutation.isPending.value || updateMutation.isPending.value"` et inclure `isPending` dans `:disabled`.

---

### B3. RetrySchedulesEdit: soumission via Enter bypass le guard `hasExceedingRetries`
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue:174-209`
**Détecté par:** Agent 4

Le bouton est disabled quand des délais dépassent le max, mais la soumission via Enter key bypass ce guard. Le schema Zod valide `base_delay` et `wait_factor` individuellement, pas les délais calculés (`bd * wf^i`).

**Fix:** Ajouter un guard en haut de `onSubmit`: `if (hasExceedingRetries.value) return;` ou valider les délais calculés dans le `superRefine` Zod.

---

### B4. cleanPayload envoie `organization_id` dans le body PUT (type mismatch)
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue:138-172`
**Détecté par:** Agents 3, 5

`cleanPayload()` inclut toujours `organization_id`. Pour le create (`RetrySchedulePost`), c'est correct. Pour l'update, c'est passé comme `schedule: RetrySchedulePut` qui n'a PAS `organization_id`. TypeScript ne catch pas ça car le payload est passé comme variable (pas d'excess property check).

**Fix:** Séparer les payloads create/update, ou destructurer `organization_id` avant de passer au update mutation.

---

### B5. Hook0Slider: `parseDuration` est toujours utilisé pour l'édition, même pour des sliders non-durée
**Fichier:** `frontend/src/components/Hook0Slider.vue:65`
**Détecté par:** Agents 1, 3, 4

`confirmEdit()` appelle toujours `parseDuration()` quel que soit le type de slider. Actuellement `editable=true` n'est set que sur les sliders de durée, donc c'est un bug latent.

**Fix:** Accepter un prop optionnel `parseFn` (default: `parseDuration`) pour rendre le composant générique. Ou documenter que `editable` ne fonctionne que pour les sliders de durée.

---

## FIX — À corriger avant merge

### F1. Hook0Slider: span éditable non accessible au clavier
**Fichier:** `frontend/src/components/Hook0Slider.vue:107`
**Détecté par:** Agents 2, 4

Le `<span>` avec `@click="startEditing"` n'a ni `tabindex`, ni `role="button"`, ni `@keydown.enter`. Les utilisateurs clavier ne peuvent pas activer l'édition inline.

**Fix:** Ajouter `tabindex="0"`, `role="button"`, `@keydown.enter="startEditing"`, et `:aria-label`.

---

### F2. Hook0Slider: input d'édition sans label accessible
**Fichier:** `frontend/src/components/Hook0Slider.vue:92-99`
**Détecté par:** Agents 2, 3

L'input text n'a pas d'`aria-label`. Les lecteurs d'écran ne sauront pas à quoi sert cet input.

**Fix:** Ajouter `:aria-label="label"`.

---

### F3. Hook0HealthBadge: concaténation de string au lieu d'interpolation i18n
**Fichier:** `frontend/src/components/Hook0HealthBadge.vue:50`
**Détecté par:** Agents 1, 2

```js
Math.round(props.failurePercent) + '% ' + t('health.failureRate')
```

Non localisable — certaines locales placent le pourcentage différemment.

**Fix:** Utiliser une clé i18n avec interpolation: `t('health.failureRateTooltip', { percent: Math.round(props.failurePercent) })`.

---

### F4. RetrySchedulesEdit: bouton supprimer intervalle sans aria-label
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue:455`
**Détecté par:** Agents 2, 4

Le bouton trash icon n'a que l'icône avec `aria-hidden="true"`. Les lecteurs d'écran n'annoncent rien.

**Fix:** Ajouter `:aria-label="t('retrySchedules.fields.removeInterval', { number: index + 1 })"`.

---

### F5. targetIsHttp dupliqué entre SubscriptionsDetail et SubscriptionsList
**Fichier:** `SubscriptionsDetail.vue:107` et `SubscriptionsList.vue:55`
**Détecté par:** Agents 1, 2, 3, 5 (unanime)

Type guard identique défini dans deux fichiers. Si le union target change, les deux doivent être mis à jour.

**Fix:** Extraire dans `SubscriptionService.ts`.

---

### F6. Styles CSS `:deep(.log-status-*)` dupliqués (~75 lignes)
**Fichier:** `SubscriptionsDetail.vue:266-340`
**Détecté par:** Agents 1, 2, 3, 5 (unanime)

Les styles de statut des logs sont dupliqués entre LogList et SubscriptionsDetail. Tout changement doit être fait deux fois.

**Fix:** Extraire dans un fichier CSS partagé ou un composant `LogStatusPill.vue`.

---

### F7. RetrySchedulesEdit: pas de mapping d'erreurs 422 sur les champs
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue:174-210`
**Détecté par:** Agent 5

Contrairement à SubscriptionsEdit qui mappe les erreurs 422 sur les champs inline, toutes les erreurs API sont envoyées en toast générique.

**Fix:** Ajouter un `handleValidationError` équivalent pour les champs du formulaire retry schedule.

---

### F8. Hook0Slider: message d'erreur de plage affiche des secondes brutes
**Fichier:** `frontend/src/components/Hook0Slider.vue:73`
**Détecté par:** Agents 3, 5

`t('slider.rangeError', { min: props.min, max: props.max })` affiche "Must be between 1 and 86400", ce qui est incompréhensible pour l'utilisateur.

**Fix:** Utiliser `props.formatValue` pour formater min/max: `{ min: props.formatValue?.(props.min) ?? props.min, max: props.formatValue?.(props.max) ?? props.max }`.

---

### F9. Groupe radio des stratégies sans fieldset/legend
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue:317-342`
**Détecté par:** Agent 4

Trois `SelectableCard` radio inputs avec le même `name="strategy"` sans `<fieldset>/<legend>`. Les lecteurs d'écran ne peuvent pas annoncer le label du groupe.

**Fix:** Wrapper dans `<fieldset>` avec `<legend class="sr-only">{{ t('retrySchedules.fields.strategy') }}</legend>`.

---

### F10. retrySchedule.schema.ts: `.max(MAX_NAME_LENGTH)` sans message i18n
**Fichier:** `frontend/src/pages/organizations/retry_schedules/retrySchedule.schema.ts:125`
**Détecté par:** Agent 3

`.min()` a un message traduit, mais `.max()` utilise le message par défaut de Zod en anglais.

**Fix:** Ajouter un message i18n et ajouter `validation.maxLength` à en.json.

---

### F11. RetrySchedulesList: `v-else-if="schedules"` devrait être `v-else`
**Fichier:** `frontend/src/pages/organizations/retry_schedules/RetrySchedulesList.vue:138`
**Détecté par:** Agent 4

Après `v-else-if="isLoading || !schedules"`, la condition `v-else-if="schedules"` est toujours vraie. Trompeur.

**Fix:** Changer en simple `v-else`.

---

### F12. SubscriptionsDetail: pas d'error handling pour la query health events
**Fichier:** `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue:54-57`
**Détecté par:** Agents 1, 5

Si la requête health events échoue, la timeline disparaît silencieusement. Aucun feedback utilisateur.

**Fix:** Destructurer `error: healthError` et afficher une erreur inline pour la section timeline.

---

## CONSIDER — Améliorations recommandées

### C1. Fichiers dépassant 200 lignes
**Détecté par:** Agents 1, 2, 4

| Fichier | Lignes | Recommandation |
|---------|--------|----------------|
| RetrySchedulesEdit.vue | 629 | Extraire PreviewChips, CustomIntervalsEditor |
| SubscriptionsEdit.vue | 658 | Extraire useSubscriptionForm composable |
| SubscriptionsList.vue | 396 | Extraire useSubscriptionColumns |
| SubscriptionsDetail.vue | 342 | Extraire column overrides + styles partagés |
| Hook0Slider.vue | 248 | Borderline — CSS scoped est la cause |

### C2. SubscriptionHealthTimeline: `Intl.RelativeTimeFormat` recréé à chaque appel
**Fichier:** `SubscriptionHealthTimeline.vue:21-23`
**Détecté par:** Agents 1, 2, 5

Chaque appel à `relativeDate()` crée un nouveau formatter. Pour une timeline avec N events, c'est N instances.

**Fix:** Cacher le formatter dans un `computed` ou au niveau module, indexé par locale.

---

### C3. SubscriptionHealthTimeline: import direct du plugin i18n
**Fichier:** `SubscriptionHealthTimeline.vue:3`
**Détecté par:** Agents 1, 5

Utilise `import i18n from '@/plugins/i18n'` au lieu de `useI18n()`.

**Fix:** `const { locale } = useI18n()` et passer `locale.value` au formatter.

---

### C4. SubscriptionHealthTimeline: timeline sans markup sémantique
**Fichier:** `SubscriptionHealthTimeline.vue:39-58`
**Détecté par:** Agents 2, 4

Les items de timeline sont des `<div>` au lieu de `<ol>/<li>`.

**Fix:** Utiliser `<ol>` pour le container et `<li>` pour chaque event.

---

### C5. RetrySchedulesEdit: pas de navigation après update réussi
**Fichier:** `RetrySchedulesEdit.vue:192-208`
**Détecté par:** Agent 5

Après create, navigation vers la liste. Après update, seulement un toast. Incohérent.

**Fix:** Ajouter navigation vers la liste après update, comme pour create.

---

### C6. SubscriptionsEdit: `type="button"` sur le submit non-tutorial
**Fichier:** `SubscriptionsEdit.vue:524`
**Détecté par:** Agents 3, 5

Le bouton non-tutorial a `type="button"` (pas de soumission via Enter), tandis que le tutorial a `type="submit"`. Incohérent.

**Fix:** Utiliser `type="submit"` partout et supprimer le `@click="onSubmit"` redondant.

---

### C7. retrySchedule.schema.ts: slider et text input ont des max différents
**Fichier:** `retrySchedule.schema.ts:33` / `RetrySchedulesEdit.vue`
**Détecté par:** Agent 3

Le slider a `max=SLIDER_MAX_BASE_DELAY (300)`, mais le text input autorise jusqu'à `MAX_BASE_DELAY (3600)`. Le slider ne peut pas représenter une valeur tapée au-delà de 300.

**Fix:** Clamper le slider pour afficher la position correcte même au-delà de SLIDER_MAX_BASE_DELAY, ou limiter le text input à SLIDER_MAX_BASE_DELAY.

---

### C8. LogService: `listBySubscription` sans pagination ni limit
**Fichier:** `frontend/src/pages/organizations/applications/logs/LogService.ts:58-71`
**Détecté par:** Agent 5

Fetch ALL les deliveries d'une subscription dans les 7 derniers jours sans `limit`. Un abonnement à fort trafic peut retourner des milliers de lignes.

**Fix:** Ajouter un paramètre `limit` (ex: 100) et pagination/load more.

---

### C9. HealthEvent query key n'inclut pas organizationId
**Fichier:** `frontend/src/queries/keys.ts:94-96`
**Détecté par:** Agent 5

`healthEventKeys.list(subscriptionId)` n'inclut pas `organizationId`, contrairement à `retryScheduleKeys.detail` qui l'inclut. Incohérent.

**Fix:** Aligner avec le pattern des retry schedule keys.

---

### C10. updateInterval sans clamp de borne supérieure
**Fichier:** `RetrySchedulesEdit.vue:224-228`
**Détecté par:** Agent 1

`updateInterval` parse le string en Number avec fallback à 1 sur NaN, mais pas de clamp supérieur. Le `max` HTML est advisory.

**Fix:** `current[index] = Math.min(Math.max(Number(value) || 1, 1), MAX_INTERVAL_SECONDS)`.

---

### C11. SubscriptionsEdit: 10+ refs individuels hors VeeValidate
**Fichier:** `SubscriptionsEdit.vue:126-146`
**Détecté par:** Agent 2

Le composant gère deux systèmes d'état parallèles. Beaucoup de state manuel.

**Fix:** Grouper les refs non-validés dans un seul `reactive()` pour simplifier.

---

### C12. Seuils de santé hardcodés (80/95) dupliqués entre front et back
**Fichier:** `Hook0HealthBadge.vue:14-16`
**Détecté par:** Agents 1, 3, 4

`WARNING_THRESHOLD = 80` et `CRITICAL_THRESHOLD = 95` avec commentaire "must match health_monitor defaults in api/src/main.rs". Risque de drift.

**Fix:** Servir les seuils depuis l'API de config, ou au minimum ajouter le chemin exact du fichier backend dans le commentaire.

---

### C13. Vérifier la pluralisation vue-i18n v11 pour `delayCustom`
**Fichier:** `en.json:1034`
**Détecté par:** Agents 1, 2, 5

Le key utilise la syntaxe pipe `{count} custom interval | {count} custom intervals`. Vérifier que `t('retrySchedules.delayCustom', { count: N })` déclenche correctement la pluralisation en vue-i18n v11.

---

### C14. SubscriptionsEdit: retry_schedule_id falsy coercion
**Fichier:** `SubscriptionsEdit.vue:297`
**Détecté par:** Agent 5

`retry_schedule_id: selectedRetryScheduleId.value || null` — le `||` convertit toute valeur falsy en null.

**Fix:** Utiliser une vérification explicite: `selectedRetryScheduleId.value === '' ? null : selectedRetryScheduleId.value`.

---

## NIT — Style et petites améliorations

| # | Fichier | Description |
|---|---------|-------------|
| N1 | `SubscriptionHealthTimeline.vue:51` | Concaténation de string pour clés i18n dynamiques — rend l'analyse statique impossible |
| N2 | `retryScheduleFormatters.ts:10` | `TranslateFn` re-déclaré manuellement au lieu d'utiliser le type vue-i18n |
| N3 | `LogService.ts:26-38` | TODO pour `RequestAttemptExtended` — créer une issue GitHub au lieu d'un commentaire |
| N4 | `SubscriptionsEdit.vue:65` | `defineEmits` utilise la syntaxe array au lieu du type-based |
| N5 | `Hook0Slider.vue:134` | `gap: 0.375rem` (6px) — hors de l'échelle de spacing (4px/8px) |
| N6 | `RetrySchedulesEdit.vue` | Preview chips sans `:focus-visible` ring |
| N7 | `SubscriptionsList.vue:389-394` | `:deep(nth-child)` fragile pour le responsive — utiliser CSS class ou column visibility API |
| N8 | `SubscriptionsDetail.vue:168` | `JSON.stringify` brut pour les targets non-HTTP — pretty-print avec indentation |
| N9 | `SubscriptionsEdit.vue:99-101` | Double cast `as unknown as Record<string, never>` — fixer le spec OpenAPI |
| N10 | `SubscriptionsList.vue:107-121` | Spread conditionnel pour colonne delete évalué une seule fois (pas reactif pour RBAC futur) |
| N11 | `formatDuration.ts` + `parseDuration.ts` | Pas de tests unitaires dans le diff — fonctions critiques avec edge cases |
| N12 | `SubscriptionSectionAdvanced.vue:144` | `--color-text-muted` — vérifier qu'il est défini dans le thème |
| N13 | `formatDuration.ts:7-28` | Abbreviations d'unités hardcodées en anglais — dette i18n connue |

---

## QUESTION

### Q1. SubscriptionsEdit: comportement double-submit entre tutorial et non-tutorial
**Fichier:** `SubscriptionsEdit.vue:521-540`

Le bouton tutorial est `type="submit"` + `@click="onSubmit"`. Le `Hook0Form` a `@submit="onSubmit"`. Est-ce que ça cause un double-fire? Le bouton non-tutorial est `type="button"` + `@click` (pas de soumission via Enter). L'incohérence est-elle intentionnelle?

---

## Points forts

Les 5 reviewers s'accordent sur les qualités suivantes:

- **Architecture propre**: séparation Service / Query / Schema / Formatter / Component bien respectée
- **i18n complet**: aucun texte hardcodé trouvé (hors formatDuration qui est une dette connue)
- **TanStack Query**: patterns de query keys factory, invalidation, et cache bien utilisés
- **Zod validation**: schemas avec cross-field validation via superRefine, bonne couverture
- **Accessibilité partielle**: icônes avec `aria-hidden`, couleur jamais seul indicateur d'état
- **cleanPayload pattern**: intelligent — empêche les 422 en nettoyant les champs de stratégie inactive

---

## Priorités d'action

### Avant merge (bloquant)
1. **B2** — Submit button loading/disabled (unanime 5/5)
2. **B3** — Guard Enter-key submission quand delays > max
3. **B1** — `@blur` handler sur l'input d'édition du slider

### Avant merge (recommandé)
4. **B4** — `organization_id` dans le PUT body
5. **F1** — Keyboard accessibility du slider éditable
6. **F3** — i18n interpolation dans HealthBadge tooltip
7. **F5** — Extraire `targetIsHttp` (duplication)
8. **F6** — Extraire styles CSS log-status (duplication ~75 lignes)
9. **F7** — Mapping erreurs 422 champs retry schedule
10. **F8** — Range error message formatée

### Post-merge (améliorations)
11. **C1** — Découper les fichiers > 200 lignes
12. **C5** — Navigation après update retry schedule
13. **C8** — Pagination deliveries par subscription
14. **N11** — Tests unitaires formatDuration/parseDuration
