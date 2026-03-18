# Frontend Development Guidelines

## Development Server

**Port:** The frontend MUST always run on port `3000` (configured in `vite.config.ts` with `strictPort: true`).

**Start command:**
```bash
source .envrc && npm run dev
```

**URLs:**
- Local: http://localhost:3000/
- API endpoint is configured in `.envrc` (production: `https://app.hook0.com/api/v1`)

## Architecture (post-redesign)

### State Management
- **Pinia**: ONLY for client state (auth, UI preferences, org/app context). Maximum 3 stores.
- **TanStack Query**: ALL server/API state. Pinia stores NEVER cache API responses.
- Service layer adapters: existing `*Service.ts` files are the fetch layer, wrapped by TanStack Query composables in `features/*/queries/`.

### Data Fetching
- NEVER use raw Axios in page components. Always use TanStack Query hooks.
- Query keys follow factory pattern per domain (see spec section 8.3).
- NEVER use async/await with try/catch. Use `.then()` and `.catch()` in service files.

### Forms
- ALL forms use VeeValidate + Zod schemas.
- Validation errors (422) mapped inline to form fields.
- Server errors (500) shown as toast.

### Components
- All UI components use `Hook0*` prefix.
- Built from scratch (no headless library like Radix).
- WCAG 2.1 AA compliance required (aria attributes, keyboard nav, focus management).

### Page Convention (ESLint-enforced)
Every page component follows this structure:
1. Imports
2. Route params
3. Permissions check (`usePermissions()`)
4. Data queries (TanStack Query composables)
5. Skeleton while loading
6. ErrorCard on error with retry
7. Content when data is available
8. Wrapped in `Hook0PageLayout`

### File Organization
- Hybrid: shared UI in `components/`, features in `features/` with colocated queries/schemas/sub-components.
- Stores and composables at root level.

## Design System

### CSS Architecture
- **Scoped CSS only** — every component/page uses `<style scoped>`.
- **BEM-style class naming** — `.block__element--modifier` (e.g. `.org-card__header`, `.app-item--create`).
- **CSS custom properties** for ALL colors, spacing, radii, shadows, fonts. NEVER hardcode hex/rgb values.
- **No Tailwind utility classes in templates** — use scoped CSS classes instead. Tailwind is available as a utility layer but page-level styling MUST use scoped CSS with BEM naming.

### CSS Custom Properties Reference
```css
/* Backgrounds */
--color-bg-primary, --color-bg-secondary, --color-bg-tertiary, --color-bg-elevated

/* Text */
--color-text-primary, --color-text-secondary, --color-text-tertiary

/* Borders */
--color-border, --color-border-strong

/* Brand */
--color-primary, --color-primary-hover, --color-primary-light, --color-primary-text

/* Semantic */
--color-success, --color-success-light, --color-warning, --color-warning-light
--color-error, --color-error-light, --color-info, --color-info-light

/* Typography */
--font-sans, --font-mono

/* Shadows */
--shadow-sm, --shadow-md, --shadow-lg, --shadow-xl

/* Radii */
--radius-sm (4px), --radius-md (6px), --radius-lg (8px), --radius-xl (12px), --radius-full (9999px)
```

### Icons
- **Use Lucide icons** (`lucide-vue-next`) exclusively: `import { Building2, Box, Plus } from 'lucide-vue-next'`
- **NEVER use `Hook0Icon`** (FontAwesome-based legacy component). It is deprecated.
- Always set `aria-hidden="true"` on decorative icons.
- Standard sizes: `:size="14"` small, `:size="16"` default, `:size="18"` medium, `:size="20"` large, `:size="24"` XL.

### Typography & Text
- **NEVER use `Hook0Text`** (legacy component). Use direct `<span>`, `<p>`, `<h1>`–`<h6>` elements.
- Apply typography through scoped CSS classes:
  - `.some-block__title` → `font-weight: 600; color: var(--color-text-primary);`
  - `.some-block__subtitle` → `font-size: 0.875rem; color: var(--color-text-secondary);`
  - `.some-block__code` → `font-family: var(--font-mono); font-size: 0.8125rem;`

### Visual Hierarchy Rules
1. **Page-level** — Use `Hook0PageLayout` for title/breadcrumbs. Pages are NOT wrapped in a card.
2. **Section-level** — Use `Hook0Card` + `Hook0CardHeader` for distinct sections within a page.
3. **Content-level** — Use `Hook0CardContent` + `Hook0CardContentLine` for key-value pairs and form fields.
4. **List-level** — Use `Hook0Table` for tabular data. Use custom styled lists for non-tabular items.

### Hover Effects & Transitions
Every interactive element MUST have visible hover/focus feedback:
```css
.some-item {
  transition: all 0.15s ease;
}
.some-item:hover {
  background-color: var(--color-bg-secondary);
}
.some-item:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}
```
- Cards: `border-color` change + subtle `box-shadow` on hover.
- Buttons: background shift, subtle `translateY(-1px)` lift.
- List items: background color change, chevron/icon color shift.
- Links: color transition.

### Badges & Pills
Use scoped CSS for status badges, NOT inline Tailwind utilities:
```css
.badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}
```

### Avatars
For entities (organizations, users), generate initials-based avatars with gradient backgrounds:
```css
.avatar {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #ffffff;
  font-size: 0.8125rem;
  font-weight: 700;
}
```

### Empty States
Always provide meaningful empty states with:
1. A muted icon (from Lucide, `:size="24"` or larger, `opacity: 0.5`)
2. A title explaining the state
3. A description with guidance
4. A CTA button when applicable

### Danger Zones (delete sections)
Destructive actions use `--color-error` / `--color-error-light` for visual distinction:
- Red-tinted icon or badge in the section header
- Danger-styled button: `background-color: var(--color-error); color: #ffffff;`
- Confirmation dialogs before destructive mutations

## Functional Components (keep using)

These are properly styled, functional UI components. Use them as building blocks:

| Component | Purpose |
|-----------|---------|
| `Hook0PageLayout` | Page wrapper with title, breadcrumbs |
| `Hook0Card` | Card container |
| `Hook0CardHeader` | Card header with `#header`, `#subtitle`, `#actions` slots |
| `Hook0CardContent` | Card content wrapper |
| `Hook0CardContentLine` | Key-value pair layout (types: `split`, `full-width`, `stacked`) |
| `Hook0CardFooter` | Card footer with action buttons |
| `Hook0Table` | TanStack Table wrapper |
| `Hook0Button` | Button with variants (`primary`, `secondary`, `danger`, `ghost`) |
| `Hook0Input` | Text input with label, error, help text |
| `Hook0Select` | Select dropdown |
| `Hook0Textarea` | Textarea with label, error |
| `Hook0Checkbox` | Checkbox input |
| `Hook0Skeleton` | Loading skeleton placeholder |
| `Hook0CardSkeleton` | Full card skeleton |
| `Hook0ErrorCard` | Error display with retry button |
| `Hook0EmptyState` | Empty state display |
| `Hook0Code` | Code block with syntax highlighting |
| `Hook0DateTime` | Date/time formatting |
| `Hook0Consumption` | Quota/consumption meter |
| `Hook0TutorialWidget` | Tutorial progress widget |

## Deprecated Components (NEVER use)

| Component | Replacement |
|-----------|------------|
| `Hook0Text` | Direct `<span>`, `<p>`, `<code>` elements with scoped CSS |
| `Hook0Icon` | Lucide icons: `import { IconName } from 'lucide-vue-next'` |
| `Hook0List` / `Hook0ListItem` | Scoped CSS styled `<ul>/<li>` or custom layouts |
| `Hook0CardContentLines` | Direct content in `Hook0CardContent` (the `<dl>` wrapper is rarely needed) |

## Hard Requirements

### E2E Test Preservation
- All `data-test-*` attributes from current components MUST be preserved on equivalent elements.
- Existing E2E tests in `tests-e2e/` must continue working WITHOUT modification.
- If a `data-test-*` attribute cannot be placed on an equivalent element, you MUST ask for explicit permission BEFORE modifying any E2E test. Explain WHY the test needs to change.

### i18n
- ALL UI strings go through `vue-i18n` (`$t()` or `t()`).
- No hardcoded strings in templates.
- English only for now, but architecture supports adding locales.
- Locale file: `src/locales/en.json`. Keys are organized by domain (e.g. `organizations.*`, `applications.*`, `events.*`).
- For strings with interpolation containing literal braces (like JSON examples), escape with `{'{'}` and `{'}'}` to prevent vue-i18n parsing errors.

### No Hardcoded Lists
- NEVER write hardcoded lists/arrays (e.g., list of pages, navigation items).
- ALWAYS prefer dynamic discovery (scanning directories, reading from generated files, parsing data).

### Dark Mode
- ALL components use CSS custom properties (not hardcoded colors).
- Light theme and dark theme must both work correctly.
- Use `.dark` class on `<html>` for dark mode.

### Permissions
- Use `usePermissions()` composable for all action visibility (create, edit, delete).
- Currently returns `true` for everything (RBAC-ready, backend not implemented yet).

### Loading States
- Skeleton screens for ALL data-dependent components.
- TanStack Query `isLoading` drives skeleton display.
- Background refetches show subtle progress bar, not skeleton.

### TanStack Query Conditional Rendering (CRITICAL)

**Root cause of blank pages:** When TanStack Query's `enabled` condition is `false` (e.g., `applicationId` is empty during navigation), the query returns:
- `isLoading = false`
- `error = undefined`
- `data = undefined`

If the template uses `v-if="isLoading"` → `v-else-if="error"` → `v-else-if="data"`, ALL conditions can be false → **blank page**.

**CORRECT pattern:**
```vue
<template>
  <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
  <Hook0Card v-if="isLoading || !data">
    <Hook0SkeletonGroup :count="3" />
  </Hook0Card>

  <!-- Error state -->
  <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />

  <!-- Data loaded (data is guaranteed to be defined here) -->
  <template v-else-if="data">
    <Hook0Card>
      <Hook0CardContent v-if="data.length > 0">
        <!-- Table/content -->
      </Hook0CardContent>
      <Hook0CardContent v-else>
        <Hook0EmptyState />
      </Hook0CardContent>
    </Hook0Card>
  </template>
</template>
```

**Rules:**
1. Use `v-if="isLoading || !data"` for skeleton - handles both loading AND disabled query states
2. Use `v-else-if="data"` for data template - guarantees data is defined inside
3. Use explicit `data.length > 0` (NOT `data?.length`) for content checks
4. NEVER use optional chaining (`?.`) in Vue templates for array length checks

### Error Handling
- Error boundaries at layout level (safety net).
- Contextual inline errors per query (ErrorCard with retry).
- Mutation validation errors inline in forms.
- Server/network errors as toast.

### Promise Handling
- NEVER use `async/await` with `try/catch`. Use `.then()` and `.catch()`.
- Add `void` before all unhandled promise calls (`void router.push(...)`, `void someQuery.refetch()`).
- ESLint enforces `@typescript-eslint/no-floating-promises`.

### ESLint & Formatting
- All optional props with non-primitive types MUST have explicit `undefined` default in `withDefaults()`.
- Prettier formatting enforced via `prettier-vue/prettier` ESLint rule.
- Run `npx eslint --fix` after modifying files.
- Run `npx vue-tsc --noEmit` to verify no type errors.

### Breadcrumbs
- All pages under `/organizations/*` MUST have detailed breadcrumbs.
- Breadcrumb format example:
  ```
  [Organizations] / [org-icon] [Org Name] / [app-icon] [App Name] / [Current Page] / [Detail Name]
  ```
- Each breadcrumb segment except the last one MUST be a clickable link.
- Organization and application names come from `useContextStore` (Pinia store).
- Use Lucide icons: `Building2` for organizations, `Box` for applications.
- The breadcrumb adapts based on route depth:
  - Organization level: `Organizations / [Building2] Org Name`
  - Application level: `Organizations / [Building2] Org Name / [Box] App Name`
  - Feature level: `Organizations / [Building2] Org Name / [Box] App Name / Feature Page`
  - Detail level: `Organizations / [Building2] Org Name / [Box] App Name / Feature Page / Detail Name`

## Stack Reference
- Vue 3.5+ / Vue Router 4.6+ / Vite 7+
- Tailwind CSS v4 (CSS-first config, no tailwind.config.js)
- TanStack Query + TanStack Table
- Pinia 3 (client state only)
- VeeValidate + Zod 4
- vue-i18n 11
- Lucide icons
- Inter + JetBrains Mono fonts
- @vueuse/core + @vueuse/motion
- CodeMirror 6 (enhanced)
- vue-sonner (toasts)
