# Review Aggregee - feat/frontend-redesign (2026-03-16)

**10 agents independants | 10 commits revus | 7 skills charges par agent**
Skills: coding-standards, frontend, language-typescript, tailwind, ui-ux, vue, vue-best-practices

---

## Synthese par consensus

Les findings ci-dessous sont classes par **nombre d'agents ayant identifie le meme probleme** (consensus), puis par severite. Seuls les findings mentionnes par 3+ agents sont inclus dans la section principale.

---

## FINDINGS A HAUTE CONFIANCE (7+ agents sur 10)

### 1. `max-width: 20rem` global sur toutes les cellules de table
**Consensus: 10/10 agents** | **Severite: WARNING**
**Fichier:** `frontend/src/components/Hook0Table.vue:419-422`
**Commit:** 665ca783 + dc39f470

```css
.hook0-table-td {
  max-width: 20rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
```

**Probleme:** Applique une troncature globale a TOUTES les cellules de TOUTES les tables de l'application. Les colonnes URLs, descriptions, UUIDs, JSON seront silencieusement tronquees. Pas de `title` pour voir le contenu complet au hover. Pas de moyen pour les colonnes individuelles de s'en exclure.

**Recommandation unanime:** Rendre la troncature opt-in via une classe CSS (`.hook0-table-td--truncate`) ou une option par colonne dans TanStack Table, plutot qu'un defaut global.

---

### 2. Cle localStorage `'hook0-theme'` dupliquee comme magic string
**Consensus: 10/10 agents** | **Severite: WARNING/CRITICAL**
**Fichiers:** `frontend/src/main.ts:35` + `frontend/src/stores/ui.ts:5`
**Commit:** 665ca783

```ts
// main.ts - magic string
const theme = window.localStorage.getItem('hook0-theme');

// stores/ui.ts - constante
const LOCAL_STORAGE_KEY_THEME = 'hook0-theme';
```

**Probleme:** Si la cle change dans le store, `main.ts` diverge silencieusement et le flash de dark mode revient. Violation du principe Single Source of Truth.

**Recommandation unanime:** Extraire la constante dans un fichier partage (`constants/storage-keys.ts`) importable par les deux fichiers.

---

### 3. Hook0Badge trust variant: `inline-flex` -> `flex` casse potentiellement les usages inline
**Consensus: 10/10 agents** | **Severite: WARNING**
**Fichier:** `frontend/src/components/Hook0Badge.vue:124`
**Commit:** 665ca783

```css
.hook0-badge-trust {
  display: flex;  /* etait inline-flex */
}
```

**Probleme:** Transforme le badge en element block-level. Corrige le chevauchement dans la grille MCP mais casse tout usage inline avec du texte.

**Recommandation:** Appliquer `display: flex` au site d'usage (grille MCP) plutot que de changer le defaut du composant, ou ajouter un modifier class.

---

### 4. Selecteur `:deep(span)` trop large dans Hook0InputRow
**Consensus: 10/10 agents** | **Severite: WARNING/SUGGESTION**
**Fichier:** `frontend/src/components/Hook0InputRow.vue:48-51`
**Commit:** 032d36fe

```css
.hook0-input-row > :deep(span) {
  flex: 0;
  min-width: auto;
}
```

**Probleme:** Cible TOUS les `<span>` enfants directs, pas uniquement les separateurs. Un composant enfant avec un `<span>` racine sera affecte par erreur.

**Recommandation unanime:** Utiliser une classe BEM dediee (`.hook0-input-row__separator`).

---

### 5. OrganizationsDashboard: skeleton infini possible avec query desactivee
**Consensus: 9/10 agents** | **Severite: WARNING/CRITICAL**
**Fichier:** `frontend/src/pages/organizations/OrganizationsDashboard.vue:53`
**Commit:** 665ca783

```vue
<Hook0CardSkeleton v-if="orgLoading || (!organization && !orgError)" :lines="4" />
```

**Probleme:** Si `organizationId` est vide pendant une transition de route, la query reste desactivee indefiniment. Le skeleton s'affiche pour toujours sans moyen pour l'utilisateur d'en sortir.

**Recommandation:** Ajouter un timeout, un redirect quand `organizationId` est falsy, ou un etat "not found" distinct.

---

### 6. `color: white` hardcode dans Hook0SimpleProgressBar
**Consensus: 8/10 agents** | **Severite: WARNING**
**Fichier:** `frontend/src/components/Hook0SimpleProgressBar.vue:53`
**Commit:** 665ca783

```css
.hook0-progress-bar__fill {
  color: white;
}
```

**Probleme:** Viole la regle du CLAUDE.md: "NEVER hardcode hex/rgb values". Le `--empty` variant utilise correctement `var(--color-text-secondary)`, montrant que le pattern est connu.

**Recommandation:** Remplacer par `color: var(--color-bg-primary)` ou un token dedie `--color-on-primary`.

---

### 7. Typos persistants: `ComsumptionQuota` / `comsumption` / `consomptions`
**Consensus: 8/10 agents** | **Severite: SUGGESTION**
**Fichier:** `frontend/src/components/Hook0Consumption.vue:13,16,23`

```ts
export interface ComsumptionQuota {  // -> ConsumptionQuota
  comsumption: number;               // -> consumption
}
consomptions: ComsumptionQuota[];    // -> consumptions
```

**Probleme:** Trois fautes de frappe dans des types exportes. Pre-existant mais le commit i18n a touche ce fichier sans les corriger.

---

### 8. Hook0Popover: `aria-label="Popover"` n'est pas un label utile
**Consensus: 7/10 agents** | **Severite: SUGGESTION**
**Fichier:** `frontend/src/components/Hook0Popover.vue:113`
**Commit:** 7bd4bb0b

**Probleme:** "Popover" decrit le type de composant, pas son contenu. Un utilisateur de lecteur d'ecran n'apprend rien d'utile.

**Recommandation:** Accepter un prop `ariaLabel` depuis le parent avec un fallback generique.

---

### 9. Wizard modal `max-width: 60rem` sans breakpoint responsive
**Consensus: 7/10 agents** | **Severite: SUGGESTION**
**Fichier:** `frontend/src/pages/tutorial/TutorialWizard.vue:353`
**Commit:** 7b1f8cc5

**Probleme:** 60rem (960px) sur un viewport tablette (~1024px) ne laisse que ~64px de marge.

**Recommandation:** Ajouter `@media (max-width: 1024px) { max-width: 90vw; }`.

---

## FINDINGS A CONFIANCE MOYENNE (3-6 agents)

### 10. Hook0KeyValue: boutons icone add/remove sans `aria-label`
**Consensus: 5/10 agents** | **Severite: WARNING**
**Fichier:** `frontend/src/components/Hook0KeyValue.vue:151-169`

Les boutons `<Minus>` et `<Plus>` n'ont pas d'`aria-label`. Violation WCAG dans un fichier touche pour l'i18n.

**Fix:** Ajouter `:aria-label="t('common.remove')"` et `:aria-label="t('common.add')"`.

---

### 11. Hook0SimpleProgressBar: manque `role="progressbar"` et attributs ARIA
**Consensus: 4/10 agents** | **Severite: WARNING**
**Fichier:** `frontend/src/components/Hook0SimpleProgressBar.vue:25-36`

Pas de `role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax`. Les lecteurs d'ecran ne peuvent pas interpreter ce composant.

**Fix:** Ajouter `role="progressbar" :aria-valuenow="clampedPercentage" aria-valuemin="0" aria-valuemax="100"`.

---

### 12. Hook0MobileTabBar: etat `active` org-level Settings inconsistant
**Consensus: 4/10 agents** | **Severite: WARNING**
**Fichier:** `frontend/src/components/Hook0MobileTabBar.vue:79`
**Commit:** dc39f470

```ts
// MobileTabBar (ligne 79) - INCOMPLET
active: route.name === routes.OrganizationsDetail,

// TopNav et MobileDrawer - CORRECT
active: route.name === routes.OrganizationsDashboard || route.name === routes.OrganizationsDetail,
```

**Probleme:** Le fix dc39f470 a corrige TopNav et Drawer mais a oublie d'aligner MobileTabBar pour l'onglet Settings org-level.

**Fix:** Ajouter `|| route.name === routes.OrganizationsDashboard` a la ligne 79.

---

### 13. `v-for :key="index"` anti-pattern dans Hook0KeyValue
**Consensus: 4/10 agents** | **Severite: SUGGESTION**
**Fichier:** `frontend/src/components/Hook0KeyValue.vue:131`

Utiliser l'index comme `:key` cause des problemes de recyclage DOM quand les items sont ajoutes/supprimes. Pre-existant.

**Fix:** Generer un ID unique par paire (`crypto.randomUUID()`).

---

### 14. Division par zero dans Hook0Consumption
**Consensus: 3/10 agents** | **Severite: WARNING (pre-existant)**
**Fichier:** `frontend/src/components/Hook0Consumption.vue:52,58`

```vue
{{ Math.round((quota.comsumption / quota.quota) * 100) }}%
```

Si `quota.quota === 0`, produit `NaN%` ou `Infinity%`.

**Fix:** `quota.quota > 0 ? Math.round(...) : 0`.

---

### 15. Logique de resolution dark mode divergente entre main.ts et ui store
**Consensus: 3/10 agents** | **Severite: WARNING**
**Fichiers:** `main.ts:37` vs `stores/ui.ts:33-34`

```ts
// main.ts - plus permissif
const isDark = theme === 'dark' || (theme !== 'light' && prefersDark);

// store - plus strict
const isDark = colorMode.value === 'dark' || (colorMode.value === 'system' && prefersDark.value);
```

Les deux logiques produisent le meme resultat pour les 3 valeurs connues mais divergent sur les valeurs inattendues. Piege de maintenance.

**Recommandation:** Extraire une fonction pure `resolveIsDark()` partagee.

---

### 16. Perte de `<strong>` sur entityType apres extraction i18n
**Consensus: 2/10 agents** | **Severite: SUGGESTION**
**Fichier:** `frontend/src/components/Hook0Consumption.vue`

Le template original avait `<strong>{{ props.entityType }}</strong>`. L'extraction i18n perd cette mise en gras. Utiliser `<i18n-t>` avec un slot nomme pour preserver le formatage.

---

## RESUME QUANTITATIF

| Finding | Agents | Severite | Action |
|---------|--------|----------|--------|
| #1 Table max-width 20rem global | 10/10 | WARNING | Rendre opt-in |
| #2 Magic string localStorage | 10/10 | WARNING | Extraire constante |
| #3 Badge inline-flex -> flex | 10/10 | WARNING | Fix au site d'usage |
| #4 :deep(span) trop large | 10/10 | WARNING | Utiliser classe BEM |
| #5 Skeleton infini | 9/10 | WARNING | Ajouter fallback |
| #6 color: white hardcode | 8/10 | WARNING | CSS variable |
| #7 Typos ComsumptionQuota | 8/10 | SUGGESTION | Renommer |
| #8 aria-label Popover generique | 7/10 | SUGGESTION | Prop ariaLabel |
| #9 Modal 60rem sans responsive | 7/10 | SUGGESTION | Breakpoint media |
| #10 Boutons KV sans aria-label | 5/10 | WARNING | Ajouter aria-label |
| #11 Progress bar sans ARIA role | 4/10 | WARNING | Ajouter role |
| #12 MobileTabBar active inconsistant | 4/10 | WARNING | Aligner active check |
| #13 v-for :key="index" | 4/10 | SUGGESTION | ID unique |
| #14 Division par zero Consumption | 3/10 | WARNING | Guard |
| #15 Logique dark mode divergente | 3/10 | WARNING | Fonction partagee |
| #16 Perte <strong> i18n | 2/10 | SUGGESTION | i18n-t slot |

---

## TOP 5 ACTIONS PRIORITAIRES

1. **Rendre la troncature table opt-in** (10/10) -- Plus haut risque de regression visuelle sur toute l'app
2. **Extraire `'hook0-theme'` dans une constante partagee** (10/10) -- Fix trivial, previent une regression silencieuse
3. **Corriger l'etat `active` manquant dans MobileTabBar** (4/10) -- Bug reel, fix d'une ligne
4. **Ajouter `role="progressbar"` + ARIA sur SimpleProgressBar** (4/10) -- Violation WCAG, fix rapide
5. **Ajouter `aria-label` sur les boutons add/remove de KeyValue** (5/10) -- Violation WCAG dans un fichier deja touche pour i18n

---

## OBSERVATIONS POSITIVES (consensus)

- **i18n sweep thorough** (10/10) -- Extraction systematique et bien structuree
- **Dark mode flash prevention pattern correct** (10/10) -- `classList.toggle` avant mount
- **Touch handling mobile correct** (8/10) -- `overscroll-behavior: contain` + `touch-action: pan-y`
- **prefers-reduced-motion respecte** (5/10) -- Present dans TutorialStepProgress
- **BEM naming + scoped CSS consistent** (7/10)
- **Composition API + script setup** (6/10) -- Usage coherent partout
- **Commits bien scopes** (8/10) -- Changements chirurgicaux et atomiques
