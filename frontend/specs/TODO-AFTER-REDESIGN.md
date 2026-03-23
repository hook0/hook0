# Frontend Review — Rapport Agrege (4 agents)

> Date: 2026-03-21 | Branche: `feat/frontend-redesign`
> Chaque finding indique le nombre d'agents qui l'ont identifie (consensus).

---

## Verdict global

Le codebase est **en bon etat** pour une branche de redesign. L'architecture (Service → TanStack Query → Page components), le design token system (CSS custom properties light/dark), l'accessibilite (skip link, aria, focus trap, prefers-reduced-motion), et l'usage de Vue 3.5+ (`<script setup>`, `defineProps<T>()`, `defineSlots<T>()`) sont solides.

Les problemes critiques sont concentres sur quelques fichiers specifiques.

---

## 1. CRITICAL

### 1.1 Axios instance re-creee a chaque requete HTTP [4/4 agents]
**Fichier:** `src/http.ts:11-63`

`getAxios()` cree un **nouveau** `axios.create()` + interceptor + `import()` dynamique a chaque appel API. Pour une page avec 5 queries = 5 instances, 5 interceptors.

**Fix:** Creer une instance singleton (lazy). Injecter le token via un request interceptor qui lit le store au moment de l'appel.

---

### 1.2 `@ts-ignore` + code unsafe dans feature-flags.ts [4/4 agents]
**Fichier:** `src/feature-flags.ts:4`

- `@ts-ignore` sur un indexed assignment
- `hasOwnProperty` appele directement (risque prototype pollution)
- `reduce` avec mutation de l'accumulateur

**Fix:** Remplacer tout le fichier par:
```ts
const queryParams = Object.fromEntries(new URLSearchParams(location.search));
export const isFeatureEnabled = (feature: string) => Object.hasOwn(queryParams, feature);
```

---

### 1.3 `v-model` sur computed (store) dans UserSettings [3/4 agents]
**Fichier:** `src/pages/user/UserSettings.vue:111,124,136`

`v-model="currentUser.email"` sur un `computed` derive du auth store. Les inputs sont `disabled` mais c'est fragile — retirer `disabled` causerait une mutation silencieuse du store.

**Fix:** Remplacer `v-model` par `:model-value` (one-way binding) puisque les champs sont read-only.

---

### 1.4 `OrganizationAndApplicationSelector.vue` bypass TanStack Query — N+1 requests [1/4 agents, high impact]
**Fichier:** `src/pages/OrganizationAndApplicationSelector.vue:59-102`

Composant de 656 lignes qui appelle `OrganizationService.list()` + `ApplicationService.list()` pour chaque org dans `onMounted`, bypass TanStack Query. Waterfall de N+1 requetes, zero caching, zero error handling UI.

**Fix:** Rewriter avec `useOrganizationList()` / `useApplicationList()`. Splitter le mega-composant.

---

### 1.5 `deleteUser()` toujours en echec — feature morte avec UI trompeuse [1/4 agents]
**Fichier:** `src/pages/user/UserService.ts:4-11`

`deleteUser()` fait `return Promise.reject(...)` avec un message hardcode. Le bouton "Delete Account" montre un dialog de confirmation puis echoue systematiquement.

**Fix:** Retirer l'UI de suppression ou implementer la feature. Ne pas presenter un flow interactif qui echoue toujours.

---

### 1.6 `JSON.parse(localStorage)` sans validation dans ui.ts [1/4 agents]
**Fichier:** `src/stores/ui.ts:112,124-128`

`JSON.parse` cast directement en `RecentPage[]` / `RecentWorkspace[]` sans try/catch ni validation. Un localStorage corrompu = crash runtime.

**Fix:** Wrapper dans try/catch + validation Zod au boundary.

---

### 1.7 `JSON.parse(atob(payload))` sans try/catch dans EventsService [1/4 agents]
**Fichier:** `src/pages/organizations/applications/events/EventsService.ts:27`

Un payload malformed = exception non geree qui casse la vue event detail.

**Fix:** Wrapper dans try/catch, fallback sur la string decodee brute.

---

## 2. ARCHITECTURE & PATTERNS

### 2.1 Legacy AG-Grid adapter dans Hook0Table [4/4 agents]
**Fichier:** `src/components/Hook0Table.vue:20-122`

~100 lignes d'adapter code "temporary, removed in Phase 2". Si `columnDefs`/`rowData` ne sont plus utilises nulle part, supprimer l'adapter.

---

### 2.2 Mega-composants a splitter [1/4 agents]
- `OrganizationAndApplicationSelector.vue` — 656 lignes
- `EventsList.vue` — 601 lignes (list + form + side panel + replay)
- `SubscriptionsEdit.vue` — 601 lignes (n'utilise pas `useEntityForm`)

---

### 2.3 Service layer couple au auth store [1/4 agents]
**Fichiers:** `OrganizationService.ts:19,41,51`, `ApplicationService.ts:15,42,53`

`create/update/remove` appellent `useAuthStore().refresh()` directement. Ce side-effect devrait etre dans le `onSuccess` de la mutation TanStack Query.

---

### 2.4 3 systemes d'error handling coexistent [1/4 agents]
- `displayError.ts` — gere `Problem`
- `handleMutationError.ts` — gere unknown errors + Axios
- `useAuthErrorHandler.ts` — gere auth errors
- `SubscriptionsEdit.vue` — inline `handleValidationError`

**Fix:** Unifier en un pipeline unique.

---

### 2.5 Duplicate `hasSlot()` helper dans 3 composants [1/4 agents]
`Hook0Input.vue:66`, `Hook0Button.vue:149`, `Hook0Textarea.vue:52`

**Fix:** Extraire en composable `useHasSlot`.

---

## 3. TYPESCRIPT & TYPE SAFETY

### 3.1 `UUID` = simple alias `string` [4/4 agents]
**Fichier:** `src/http.ts:126`

Zero type safety. Un `organizationId` peut etre passe comme `applicationId`.

**Fix:** Branded type via Zod `.brand()` ou nominal type.

---

### 3.2 `defineEmits` non type dans 5 composants [3/4 agents]
`OrganizationsEdit.vue:51`, `ApplicationsEdit.vue:51`, `SubscriptionsEdit.vue:64`, `EventTypesNew.vue:43`, `Hook0DropdownMenuItemLink.vue:7`

**Fix:** Convertir en `defineEmits<{ 'event-name': [payload: Type] }>()`.

---

### 3.3 `as unknown as` casts dans SubscriptionsEdit [2/4 agents]
**Fichier:** `src/pages/.../subscriptions/SubscriptionsEdit.vue:98,170`

Symptome d'un mismatch schema API (`Record<string, never>` vs `Record<string, string>`).

**Fix:** Corriger le schema OpenAPI ou ajouter un type override.

---

### 3.4 `handleError` cast `as Problem` sans validation [2/4 agents]
**Fichier:** `src/http.ts:148`

Cast avant validation. `typeof null === 'object'` n'est pas garde.

**Fix:** Ajouter `typeof data === 'object' && data !== null` avant le cast, ou utiliser le type guard `isProblem` existant dans `handleMutationError.ts`.

---

### 3.5 `async` inutile dans UserService.ts [4/4 agents]
**Fichier:** `src/pages/user/UserService.ts:4,14,22,26,30`

Toutes les fonctions sont `async` mais retournent juste `unwrapResponse(...)`.

**Fix:** Retirer `async`.

---

### 3.6 `isAxiosError` custom reimplemente l'existant [1/4 agents]
**Fichier:** `src/http.ts:167-170`

Axios exporte deja `isAxiosError()`.

**Fix:** Utiliser l'import Axios natif.

---

## 4. VUE BEST PRACTICES

### 4.1 `useSlots()` appele dans des fonctions, pas au top-level setup [4/4 agents]
**Fichiers:** `Hook0Button.vue:150`, `Hook0Input.vue:66`

**Fix:** `const slots = useSlots()` une fois au top-level, puis `!!slots[name]` dans le helper.

---

### 4.2 `route.fullPath` comme cle de transition = re-mount sur query params [4/4 agents]
**Fichier:** `src/App.vue:79`

Chaque changement de query param (pagination, filtres) detruit et recree le composant page entier.

**Fix:** Utiliser `route.path` ou `route.name`.

---

### 4.3 `onMounted` + `onUpdated` pour promise tracking dans Hook0Button [3/4 agents]
**Fichier:** `src/components/Hook0Button.vue:153-159`

**Fix:** Remplacer par `watch(() => props.loading, forwardPromiseState, { immediate: true })`.

---

### 4.4 `watch(accessToken, ..., { deep: true })` sur un string [1/4 agents]
**Fichier:** `src/pages/OrganizationAndApplicationSelector.vue:56`

`deep: true` n'a aucun effet sur une primitive.

**Fix:** Retirer `{ deep: true }`.

---

## 5. TAILWIND & STYLING

### 5.1 Breakpoints inconsistants [1/4 agents]
Mix de `max-width: 639px` vs `max-width: 640px` (off-by-one) et mobile-first vs desktop-first.

**Fix:** Standardiser sur mobile-first (`min-width`). Fixer le `640px` → `639px`.

---

### 5.2 `color-scheme: dark` manquant [2/4 agents]
Les controles natifs (scrollbars, `<select>`, date pickers) ne s'adaptent pas au dark mode.

**Fix:** Ajouter `:root.dark { color-scheme: dark; }` dans `tailwind.css`.

---

### 5.3 Hardcoded `rgba()` dans plusieurs composants [3/4 agents]
`Hook0Dropdown.vue:169-172`, `Hook0Popover.vue:198,217,236,255`, `Hook0Error404Background.vue:87-88`

**Fix:** Utiliser les shadow tokens existants (`var(--shadow-lg)`) ou `color-mix()`.

---

### 5.4 CSS mort `.sub-secret__warning` [1/4 agents]
**Fichier:** `src/pages/.../subscriptions/SubscriptionsEdit.vue:525-530`

**Fix:** Supprimer.

---

### 5.5 Double chargement de fonts [1/4 agents]
`@fontsource-variable/inter` importe en JS dans `main.ts` ET declare via `@font-face` dans `tailwind.css`.

**Fix:** Garder un seul mecanisme (preferer `@font-face` CSS).

---

## 6. UI/UX

### 6.1 `autocomplete` manquant sur le form de changement de mot de passe [1/4 agents]
**Fichier:** `src/pages/user/UserSettings.vue:204-226`

**Fix:** Ajouter `autocomplete="new-password"` sur les deux inputs.

---

### 6.2 Dates brutes ISO dans le side panel events [1/4 agents]
**Fichier:** `src/pages/.../events/EventsList.vue:479,483`

`{{ selectedEventData.received_at }}` affiche un timestamp ISO brut.

**Fix:** Utiliser `Hook0DateTime` ou `Intl.DateTimeFormat`.

---

### 6.3 `Hook0DateTime` non locale-aware [2/4 agents]
**Fichier:** `src/components/Hook0DateTime.vue:13`

Utilise `formatISO9075` (format fixe) au lieu de `Intl.DateTimeFormat`.

---

### 6.4 `prefers-reduced-motion` non respecte sur les transitions de page [1/4 agents]
**Fichier:** `src/App.vue` + `src/assets/styles/transitions.css`

**Fix:** Ajouter `@media (prefers-reduced-motion: reduce) { transition-duration: 0s }`.

---

### 6.5 `setTimeout` sans cleanup pour logout apres suppression [2/4 agents]
**Fichier:** `src/pages/user/UserSettings.vue:83-85`

Timer jamais nettoye. Si le composant unmount, le logout fire quand meme.

**Fix:** Utiliser `useTimeoutFn` de VueUse ou nettoyer dans `onBeforeUnmount`.

---

## 7. PERFORMANCE

### 7.1 Double librairie de charts [4/4 agents]
`chart.js` + `vue-chartjs` ET `echarts` + `vue-echarts` dans `package.json`. ~400KB+ combines.

**Fix:** Standardiser sur une seule librairie, supprimer l'autre.

---

### 7.2 `echarts` non inclus dans `manualChunks` [2/4 agents]
**Fichier:** `vite.config.ts:18-33`

**Fix:** Ajouter `echarts` + `vue-echarts` au chunk splitting.

---

### 7.3 Pas de virtualisation pour les listes longues [2/4 agents]
Events et logs rendent tous les rows. Si la liste grandit au-dela de ~100 rows, le DOM sera lourd.

**Fix:** Monitorer. Ajouter `@tanstack/vue-virtual` si necessaire.

---

### 7.4 Variables d'environnement non validees au demarrage [1/4 agents]
**Fichier:** `src/http.ts:32-38`

`VITE_API_ENDPOINT` fallback silencieux sur string vide = requetes vers l'origin courant = 404 confus.

**Fix:** Valider avec un schema Zod au demarrage. Fail loud.

---

## 8. QUICK WINS — Top 15 par priorite

| # | Finding | Fichier | Consensus | Effort |
|---|---------|---------|-----------|--------|
| 1 | Singleton Axios au lieu de creation per-request | `http.ts` | 4/4 | ~30min |
| 2 | Rewrite `feature-flags.ts` (supprimer `@ts-ignore`) | `feature-flags.ts` | 4/4 | 5min |
| 3 | `:model-value` au lieu de `v-model` sur inputs disabled | `UserSettings.vue` | 3/4 | 2min |
| 4 | `route.path` au lieu de `route.fullPath` pour transition key | `App.vue:79` | 4/4 | 1min |
| 5 | Retirer `async` de `UserService.ts` | `UserService.ts` | 4/4 | 2min |
| 6 | try/catch sur `JSON.parse(localStorage)` dans ui.ts | `stores/ui.ts` | 1/4 | 5min |
| 7 | try/catch sur `JSON.parse(atob())` dans EventsService | `EventsService.ts:27` | 1/4 | 3min |
| 8 | Ajouter `color-scheme: dark` | `tailwind.css` | 2/4 | 1min |
| 9 | Consolider sur 1 seule librairie de charts | `package.json` | 4/4 | ~1h |
| 10 | Typed `defineEmits<{}>()` dans 5 composants | Multiple | 3/4 | 10min |
| 11 | `useSlots()` au top-level setup | `Hook0Button.vue`, `Hook0Input.vue` | 4/4 | 5min |
| 12 | Ajouter `autocomplete="new-password"` | `UserSettings.vue` | 1/4 | 1min |
| 13 | Supprimer CSS mort `.sub-secret__warning` | `SubscriptionsEdit.vue` | 1/4 | 1min |
| 14 | Ajouter `prefers-reduced-motion` sur page transitions | `transitions.css` | 1/4 | 2min |
| 15 | Supprimer double chargement fonts | `main.ts` | 1/4 | 5min |

---

## Points positifs unanimes (4/4 agents)

- Architecture Service → TanStack Query → Page propre et bien separee
- Design token system complet (light + dark) avec CSS custom properties
- Accessibilite au-dessus de la moyenne (skip link, aria, focus trap, reduced motion)
- Vue 3.5+ idioms corrects partout (`<script setup>`, typed props/slots/models)
- BEM naming coherent avec prefix `hook0-*`
- Lazy-loading de toutes les routes
- `manualChunks` pour les deps lourdes
- Pinia limite au client state, TanStack Query pour le server state
