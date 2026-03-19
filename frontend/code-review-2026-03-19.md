# Code Review — 2026-03-19 (10 commits, 16 files, ~1900 LOC)

Aggregated from 10 independent reviewers. Findings ranked by consensus (number of agents that flagged it).

---

## CRITICAL (fix before merge)

### 1. `Hook0EventsPerDayChart.vue:110` — `computed` with zero reactive deps never recomputes (10/10 agents)

```ts
const themeColors = computed<ThemeColors>(() => getThemeColors());
```

`getComputedStyle(document.documentElement)` is a DOM read, not reactive. The computed caches once and never invalidates. Dark mode toggle = stale chart colors.

**Fix:** Remove the `computed` wrapper. Call `getThemeColors()` directly inside `chartOption` computed (which already reruns on prop changes). For true dark-mode reactivity, add a reactive signal.

---

### 2. `LogList.vue:61,108,140,169` — Unsafe `as unknown as Record<string, unknown>` casts (10/10)

5 occurrences of `(row as unknown as Record<string, unknown>).someField`. This is `any` under a different name — zero compiler protection.

**Fix:** Extend `RequestAttemptTypeFixed` to include the missing fields (`http_response_status`, `retry_count`, `succeeded_at`, `failed_at`, `picked_at`, `delay_until`, `completed_at`, `created_at`, `event_type_name`). Remove all casts.

---

### 3. `LogList.vue:127-136` — Inline styles in render functions bypass scoped CSS (10/10)

200+ char inline style strings in `renderStatusPill` and event_id cell. The same file defines `.log-status--*` scoped CSS classes (lines 312-362) that are **never used** (dead CSS).

**Fix:** Use the existing scoped CSS classes via `class` in `h()` calls. Delete inline style strings. For hover effects, use CSS `:hover` instead of `onMouseenter`/`onMouseleave` JS handlers.

---

### 4. `LogList.vue:65-80` — Hardcoded English strings bypass i18n (8/10)

```ts
case RequestAttemptStatusType.Successful: return 'Sent';
```

**Fix:** Use `t('logs.statusSent')`, `t('logs.statusFailed')`, etc.

---

### 5. `Hook0EventsPerDayChart.vue:193-212` — HTML string concatenation in tooltip (XSS risk) (8/10)

`p.seriesName` comes from `application_name` (user input) and is interpolated raw into HTML. ECharts renders tooltips via `innerHTML`.

**Fix:** Add `escapeHtml()` utility. Apply to all interpolated values.

---

## IMPORTANT (fix soon)

### 6. `Hook0EventsPerDayChart.vue:215-321` — `buildStackedChartOption` is 70-107 lines (10/10)

Too long, too many responsibilities (app grouping, top-series detection, series building, markLine, tooltip, axes).

**Fix:** Extract `groupEntriesByApp()`, `findTopSeriesPerDate()`, `buildStackedSeries()` as pure functions.

---

### 7. `LogList.vue:311-362` — 50+ lines of dead CSS (10/10)

`.log-status`, `.log-event-cell`, `.log-duration` classes defined but never referenced.

**Fix:** Use them (see #3) or delete them.

---

### 8. `OrganizationsDashboard.vue:174-241` — Quota card template repeated 4x (9/10)

Four near-identical blocks differing only in icon/value/label.

**Fix:** `v-for` over a computed `quotaCards` array, or extract a `QuotaCard` component.

---

### 9. `Hook0TopNavContextBar.vue:138` — `<template v-if="true">` no-op wrapper (9/10)

Dead logic, adds noise.

**Fix:** Remove it.

---

### 10. `useEventsPerDayQuery.ts:31-37` — `from`/`to` should be computed, not watch+ref (7/10)

Derived state via `ref` + `watch` instead of `computed`. Stale on midnight rollover.

**Fix:** Replace with `const from = computed(() => ...)`. Eliminate the watch.

---

### 11. `Hook0EventsPerDayChart.vue:165-172,299-305` — Tooltip config duplicated between builders (7/10)

Identical tooltip objects. Also hardcoded colors that won't adapt to dark mode.

**Fix:** Extract `BASE_TOOLTIP_CONFIG` constant.

---

### 12. `ApplicationsDashboard.vue:59` — Missing blank-page guard (6/10)

Should be `v-if="appLoading || (!application && !appError)"` per CLAUDE.md pattern.

---

### 13. `Hook0Tooltip.vue:86` — Global keydown listener on every instance (5/10)

N tooltips = N global listeners on every keypress.

**Fix:** Only register when `visible` is true.

---

### 14. `Hook0TopNavContextBar.vue:262,264` — Inline `style` on router-link (6/10)

Duplicate `style="color: var(--color-text-muted)"` on parent and child.

**Fix:** Use a scoped CSS class.

---

## MINOR

| # | Issue | File | Agents |
|---|-------|------|--------|
| M1 | `dayPresets` not `as const` | Hook0EventsPerDayChart.vue:83 | 7/10 |
| M2 | `cssVar()` called 12x — cache `getComputedStyle` once | Hook0EventsPerDayChart.vue:87 | 7/10 |
| M3 | Magic `z-index: 9999` — use token | Hook0Tooltip.vue:150 | 7/10 |
| M4 | `else` after `return` in formatDate | Hook0TableCellDate.vue:15 | 5/10 |
| M5 | Whitespace hack `</strong>{{ ' ' }}` | Hook0Consumption.vue:96 | 5/10 |
| M6 | `ConsumptionQuota` type exported from .vue file | Hook0Consumption.vue:11 | 4/10 |
| M7 | `as const` on string literals inside `Record<string, unknown>` is noise | Hook0EventsPerDayChart.vue | 4/10 |
| M8 | Dead CSS `.hook0-topnav__ghost-icon-btn` | Hook0TopNavContextBar.vue:645 | 3/10 |
| M9 | Hardcoded `'en'` locale in `Intl.DateTimeFormat` | LogList.vue:95 | 2/10 |
| M10 | `routeTitles` in router.ts is a hardcoded map | router.ts:10 | 1/10 |

---

## POSITIVE (unanimous praise)

| What | Where | Agents |
|------|-------|--------|
| Clean TanStack Query composable + key factory | useEventsPerDayQuery.ts, keys.ts | 10/10 |
| Excellent accessibility (ARIA, keyboard, reduced-motion) | Hook0TopNavContextBar.vue | 10/10 |
| DRY service layer with shared `fetchEventsPerDay` | EventsPerDayService.ts | 10/10 |
| Well-factored `Hook0Consumption.vue` (pure fns, BEM, tokens) | Hook0Consumption.vue | 10/10 |
| Consistent CSS custom property usage | All files | 10/10 |
| Safe redirect validation (`isValidRedirectPath`) | LoginPage.vue:49 | 6/10 |
| Good `Teleport` usage for tooltips | Hook0Tooltip.vue | 5/10 |
| `sortCurrentFirst` is a clean pure utility | Hook0TopNavContextBar.vue | 3/10 |
