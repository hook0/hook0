# Code Style Review — feat/frontend-redesign (2026-03-16)

**Scope:** 42 commits, 154 fichiers, ~12k additions, ~8k deletions
**Methode:** 10 agents reviewers independants, resultats agreges par consensus
**Notation consensus:** [N/10] = nombre d'agents ayant identifie le probleme

---

## Synthese executive

| Severite | Count | Top categories |
|----------|-------|----------------|
| Critical | 6 | DRY, TypeScript, Component Design |
| High | 12 | DRY, Complexity, Vue, TypeScript |
| Medium | 15 | Complexity, Tailwind, Vue, Functional |
| Low | 10 | Vue, YAGNI, TypeScript |

**Les 3 problemes systemiques majeurs:**
1. **Duplication massive du focus trap** — 3-5 copies identiques (corrige 1 = oublie les 4 autres)
2. **Erosion du type safety** — 22+ casts `as unknown as Problem/Record<string, never>/never`
3. **Composants geants** — SubscriptionsEdit (917L) et ServiceTokenView (920L)

---

## CRITICAL

### C1. Focus trap duplique dans 3-5 composants [10/10]

**Fichiers:**
- `components/Hook0Dialog.vue` (~54-101)
- `components/Hook0SidePanel.vue` (~46-89)
- `components/Hook0MobileDrawer.vue` (~170-196)
- `components/Hook0CommandPalette.vue` (~16-42)
- `pages/tutorial/TutorialWizard.vue` (~258-289)

**Probleme:** Le meme algorithme (query FOCUSABLE_SELECTOR, Tab/Shift+Tab wrapping, Escape close, focus-first-element) est copie-colle dans 3 a 5 composants selon le scope. Les selecteurs divergent subtilement entre les copies. Un fix d'accessibilite dans une copie ne se propage pas aux autres.

**Recommandation:** Extraire un composable `useFocusTrap(containerRef: Ref<HTMLElement | null>)` qui encapsule:
- Query des elements focusables
- Tab/Shift+Tab cycling
- Escape handler
- Focus initial + restauration au close
- Gestion de `document.body.style.overflow`

**Categorie:** DRY
**Effort:** ~2h | **Impact:** Elimine ~200 lignes dupliquees + corrige les bugs de facon centralisee

---

### C2. `displayError(err as unknown as Problem)` — 22+ occurrences [8/10]

**Fichiers:** EventsList, ApplicationsEdit, SubscriptionsList, OrganizationsEdit, ApplicationSecretsList, MembersList, EventTypesList, ServicesTokenList, EventTypesNew, ApplicationsList, OrganizationsRemove, ApplicationsRemove, SubscriptionsRemove

**Probleme:** Chaque mutation `onError` fait un double-cast `err as unknown as Problem`. Cela:
- Defeate totalement TypeScript
- Crashera silencieusement si l'erreur n'est pas un `Problem` (ex: erreur reseau)
- Viole DRY (22 copies du meme pattern)

**Recommandation:** Creer une fonction utilitaire typee:
```ts
function handleMutationError(err: unknown): void {
  if (isAxiosError(err) && isProblem(err.response?.data)) {
    displayError(err.response.data);
  } else {
    push.error({ title: 'Unexpected error', message: String(err) });
  }
}
```

**Categorie:** TypeScript / DRY
**Effort:** ~1h | **Impact:** Securise 22+ handlers d'erreur + ajoute le narrowing runtime

---

### C3. `as unknown as Record<string, never>` — 6 casts dans SubscriptionsEdit [7/10]

**Fichier:** `pages/organizations/applications/subscriptions/SubscriptionsEdit.vue` (~275-312)

**Probleme:** `Record<string, never>` signifie "objet dont les valeurs sont de type `never`" — un type impossible. Le double-cast `as unknown as Record<string, never>` cache un mismatch entre le formulaire (`Record<string, string>`) et les types API.

**Recommandation:** Corriger les types du service/query layer pour accepter `Record<string, string>` pour headers/labels/metadata. Si le type API est genere, corriger le generateur.

**Categorie:** TypeScript
**Effort:** ~30min | **Impact:** Elimine 6 casts dangereux

---

### C4. `handleError(err as never)` — cast le plus dangereux [5/10]

**Fichier:** `SubscriptionsEdit.vue` (~233, 237)

**Probleme:** `as never` est pire que `as any` — il declare qu'une valeur ne peut pas exister. Toute verification de type est eliminee silencieusement.

**Recommandation:** Typer `handleError` pour accepter `unknown` et faire du narrowing interne.

**Categorie:** TypeScript
**Effort:** ~15min | **Impact:** Restaure la securite de type sur le chemin d'erreur

---

### C5. Hook0TutorialWidget utilise des classes Tailwind hardcodees [4/10]

**Fichier:** `components/Hook0TutorialWidget.vue` (~33-78)

**Probleme:** Utilise `bg-indigo-600`, `bg-gray-300`, `text-gray-500`, etc. directement dans le template. Le CLAUDE.md interdit explicitement les classes Tailwind dans les templates et exige des CSS custom properties.

**Recommandation:** Remplacer par du scoped CSS avec `var(--color-primary)`, `var(--color-border)`, etc.

**Categorie:** Tailwind / Design System
**Effort:** ~30min | **Impact:** Compatibilite dark mode

---

### C6. Navigation tabs tripliquees entre TopNav, MobileDrawer, MobileTabBar [5/10]

**Fichiers:**
- `components/Hook0TopNav.vue` (~72-166)
- `components/Hook0MobileDrawer.vue` (~69-156)
- `components/Hook0MobileTabBar.vue` (~22-92)

**Probleme:** Les 3 composants calculent independamment les memes nav items (app-level: events, subscriptions, event-types, logs, api-keys, settings; org-level: applications, service-tokens, settings). Les icones divergent entre desktop et mobile (FileText vs Calendar pour events, Link vs Webhook pour subscriptions) — probablement un bug.

**Recommandation:** Extraire un composable `useNavigationTabs()` consomme par les 3 composants. Corriger les icones incoherentes.

**Categorie:** DRY
**Effort:** ~2h | **Impact:** Elimine ~150 lignes + corrige l'inconsistance d'icones

---

## HIGH

### H1. `ProgressStep` type duplique dans 4 fichiers [9/10]

**Fichiers:** TutorialStepProgress.vue, TutorialWizardStepForm.vue, TutorialWizardStepOrganization.vue, TutorialWizardStepApplication.vue

**Recommandation:** Exporter depuis `TutorialStepProgress.vue` ou creer `pages/tutorial/types.ts`.

**Categorie:** DRY | **Effort:** ~10min

---

### H2. `handleFormAdvance()` — if/else chain avec magic numbers [10/10]

**Fichier:** `TutorialWizard.vue` (~147-192)

**Probleme:** 3 branches `if (step === 3) / else if (step === 4) / else if (step === 5)` avec des corps quasi-identiques.

**Recommandation:** Config map:
```ts
const FORM_STEP_CONFIG: Record<number, { trackLabel: string; toastTitle: string; routeName: string }> = { ... };
const config = FORM_STEP_CONFIG[currentStep.value];
if (config) advanceStep({ ...config, params });
```

**Categorie:** Complexity / Functional | **Effort:** ~20min

---

### H3. `dismiss()` — 5 branches if/else sur step numbers [10/10]

**Fichier:** `TutorialWizard.vue` (~197-223)

**Recommandation:** Meme approche que H2 — object map keye par step number.

**Categorie:** Complexity | **Effort:** ~15min

---

### H4. `groupedCommands` — O(n^2) spread dans reduce [10/10]

**Fichier:** `useCommandPalette.ts` (~338-343)

```ts
return { ...groups, [cmd.category]: [...group, cmd] };
```

**Recommandation:** `Object.groupBy(filteredCommands.value, cmd => cmd.category)` ou reduce mutable.

**Categorie:** Functional / Complexity | **Effort:** ~5min

---

### H5. `useCelebration` — manipulation DOM directe, bypass Vue [9/10]

**Fichier:** `composables/useCelebration.ts`

**Problemes:**
- `document.createElement` + `document.body.appendChild` bypass Vue
- `<style>` injecte dans `<head>` jamais nettoye
- `setTimeout` sans cleanup si le composant unmount avant
- Pas de check `prefers-reduced-motion`
- z-index: 9999 hardcode

**Recommandation:** Recrire comme composant Vue avec `<Teleport to="body">` + scoped CSS + `onUnmounted` cleanup + check `prefers-reduced-motion`.

**Categorie:** Vue / Component Design | **Effort:** ~1h

---

### H6. `ServiceTokenView.vue` — 920 lignes, submitSimple/submitAdvanced 80% identiques [7/10]

**Fichier:** `pages/organizations/services_token/ServiceTokenView.vue`

**Recommandation:**
1. Extraire validation partagee dans `validateBeforeAttenuation()`
2. Extraire `TokenPreviewTabs` child component (template duplique ~50 lignes)
3. Split simple/advanced en sous-composants

**Categorie:** DRY / Complexity / Component Design | **Effort:** ~3h

---

### H7. `SubscriptionsEdit.vue` — 917 lignes, trop de responsabilites [6/10]

**Fichier:** `pages/organizations/applications/subscriptions/SubscriptionsEdit.vue`

**Recommandation:**
1. Extraire `SubscriptionTestEndpoint.vue` (~200 lignes)
2. Extraire le selecteur de types d'evenements
3. Deplacer `toMap`/`fromMap` dans un utilitaire partage

**Categorie:** Complexity / Component Design | **Effort:** ~2h

---

### H8. z-index chaos — pas de scale systematique [6/10]

**Probleme:** Overlays = z-50, Popovers/Tooltips = z-9999, Celebration = z-9999. Un tooltip dans un modal passe au-dessus de tout.

**Recommandation:** Definir une echelle CSS:
```css
--z-dropdown: 40;
--z-overlay: 50;
--z-modal: 60;
--z-popover: 70;
--z-tooltip: 80;
--z-celebration: 90;
```

**Categorie:** Component Design / Tailwind | **Effort:** ~1h

---

### H9. Wizard step header/footer duplique dans 4-5 step components [5/10]

**Fichiers:** TutorialWizardStepIntro, StepOrganization, StepApplication, StepForm, StepSuccess

**Recommandation:** Extraire `WizardStepLayout` avec des slots (`#header-title`, `#default`, `#footer-extra`). Elimine ~30 lignes par step + supprime le couplage `:deep()`.

**Categorie:** DRY / Component Design | **Effort:** ~1.5h

---

### H10. Keydown listeners globaux dans 6+ composants sans coordination [6/10]

**Fichiers:** TutorialWizard, useKeyboardShortcuts, Hook0TopNav, Hook0Popover, Hook0Dropdown, Hook0Tooltip

**Probleme:** Quand plusieurs composants sont montes, chaque keypress Escape declenche 4+ handlers — race condition sur `stopPropagation`.

**Recommandation:** Centraliser dans `useKeyboardShortcuts` avec une pile de priorite. Les composants s'enregistrent/desenregistrent; seul le handler le plus haut fire.

**Categorie:** Complexity / Component Design | **Effort:** ~2h

---

### H11. `displayError` redefini localement dans 3 composants Remove [4/10]

**Fichiers:** OrganizationsRemove, ApplicationsRemove, SubscriptionsRemove

**Probleme:** Fonction identique definie localement alors qu'un utilitaire partage existe deja a `utils/displayError.ts`.

**Recommandation:** `import { displayError } from '@/utils/displayError'`

**Categorie:** DRY | **Effort:** ~5min

---

### H12. `router.afterEach()` dans Hook0TopNav — guard jamais deregister [3/10]

**Fichier:** `Hook0TopNav.vue` (~196-198)

**Recommandation:**
```ts
const removeGuard = router.afterEach(() => { closeDropdowns(); });
onUnmounted(removeGuard);
```

**Categorie:** Vue | **Effort:** ~2min

---

## MEDIUM

### M1. Hardcoded `rgba(0, 0, 0, 0.5)` dans 6-8 overlays [7/10]

**Fichiers:** Hook0Dialog, Hook0ShortcutsCheatSheet, Hook0MobileDrawer, Hook0CommandPalette, Hook0SidePanel, TutorialWizard (seul ce dernier utilise `var(--color-overlay)`)

**Recommandation:** Definir `--color-overlay` dans le theme, utiliser partout.

---

### M2. Hook0Dialog — `id="hook0-dialog-title"` hardcode [6/10]

**Probleme:** IDs en collision si 2 dialogs montes simultanement.

**Recommandation:** Utiliser `useId()` (Vue 3.5+).

---

### M3. `useCommandPalette` — 386 lignes, computed de 215 lignes [5/10]

**Recommandation:** Extraire les builders par categorie en fonctions separees puis composer:
```ts
const commands = computed(() => [
  ...buildRecentItems(), ...buildOrgItems(), ...buildNavItems(), ...buildGlobalItems()
]);
```

---

### M4. `cancel2()` — nom sans signification [7/10]

**Fichier:** `SubscriptionsEdit.vue:193`

**Recommandation:** Renommer en `navigateBack()` ou `goBack()`.

---

### M5. SelectableCard — emit `update:modelValue` sans payload [6/10]

**Probleme:** Declare `'update:modelValue': []` (pas d'argument) mais utilise avec `v-model`.

**Recommandation:** Changer en `'update:modelValue': [value: boolean]` et emettre `true` au click.

---

### M6. `navigator.platform` deprecated [5/10]

**Fichier:** `Hook0ShortcutsCheatSheet.vue:19`

**Recommandation:** `navigator.userAgentData?.platform` avec fallback.

---

### M7. `CustomerLogo.vue` — 295 lignes de SVG inline [5/10]

**Recommandation:** Stocker les SVGs comme fichiers `.svg` separes, import dynamique via Vite.

---

### M8. Hook0Breadcrumbs — computed `crumbs` de ~370 lignes [3/10]

**Recommandation:** Config map declarative au lieu d'un if/else chain de 20+ branches.

---

### M9. Hook0ShortcutsCheatSheet — pas de focus trap ni `aria-modal` [4/10]

**Recommandation:** Utiliser le (futur) composable `useFocusTrap` ou `Hook0Dialog`.

---

### M10. Stagger animation — 10 regles nth-child manuelles [3/10]

**Fichier:** `transitions.css` (~860-893)

**Recommandation:** `animation-delay: calc(var(--stagger-index) * 50ms)` + `:style="{'--stagger-index': index}"`.

---

### M11. Toast `duration: 5000` — magic number repete 36 fois [3/10]

**Recommandation:** Definir un default global dans la config notivue ou extraire `const TOAST_DURATION_MS = 5000`.

---

### M12. `!important` sur placeholder styles globaux [3/10]

**Fichier:** `tailwind.css` (~189-195)

**Recommandation:** Supprimer `!important`, utiliser `@layer` pour le cascade ordering.

---

### M13. `TutorialWizardStepSuccess` — valeurs non-reactives derivees de props [3/10]

**Recommandation:** Wrapper dans `computed()`.

---

### M14. `ComponentShowcase` accessible en production [4/10]

**Fichier:** `routes.ts:265-267`

**Recommandation:** Gater avec `import.meta.env.DEV`.

---

### M15. Hook0TopNav click-outside attache sur le `<header>` [3/10]

**Recommandation:** Utiliser `onClickOutside` de @vueuse/core sur la ref du dropdown.

---

## LOW

### L1. `$emit()` en template au lieu du `emit()` type [5/10]
### L2. `authStore.logout()` floating promise dans TopNav/MobileDrawer [6/10]
### L3. `STEPS` computed appelle `markRaw()` a chaque re-evaluation [5/10]
### L4. Home.vue gradient no-op (meme couleur debut/fin) [3/10]
### L5. `let` utilise la ou `const` + pattern fonctionnel suffirait [3/10]
### L6. Hardcoded `#ffffff` dans SubscriptionsEdit, Hook0Badge, Hook0Avatar [4/10]
### L7. Route params castes avec `as string` sans narrowing [3/10]
### L8. `console.error` dans Hook0Button catch [2/10]
### L9. Deux `onMounted` dans Hook0TopNav [2/10]
### L10. `result.ts` utilise `let` — direct return dans try/catch suffirait [2/10]

---

## Plan d'action recommande (par priorite)

### Sprint 1 — Type Safety + DRY critique (~1 journee)
1. Creer `handleMutationError(err: unknown)` — remplace 22+ `as unknown as Problem` [C2]
2. Fixer les types API pour headers/labels/metadata [C3, C4]
3. Extraire composable `useFocusTrap()` [C1]
4. Exporter `ProgressStep` type une seule fois [H1]

### Sprint 2 — Architecture composants (~1 journee)
5. Extraire `useNavigationTabs()` composable [C6]
6. Extraire `WizardStepLayout` component [H9]
7. Decomposer ServiceTokenView (TokenPreviewTabs, validation) [H6]
8. Decomposer SubscriptionsEdit (TestEndpoint) [H7]

### Sprint 3 — Design System + Polish (~0.5 journee)
9. Definir echelle z-index comme CSS custom properties [H8]
10. Remplacer hardcoded colors par CSS vars [C5, M1, L6]
11. Config maps pour TutorialWizard step routing [H2, H3]
12. Recrire useCelebration en composant Vue [H5]

### Quick wins (< 15min chacun)
- `Object.groupBy` dans useCommandPalette [H4]
- Renommer `cancel2()` [M4]
- Importer displayError partage dans les Remove [H11]
- `void authStore.logout()` [L2]
- `useId()` dans Hook0Dialog [M2]
- Guard `import.meta.env.DEV` sur ComponentShowcase [M14]
- Fix `router.afterEach` cleanup [H12]
