<script setup lang="ts">
/**
 * Hook0ContextBar - Org/App context switching section of the top nav.
 *
 * Displays the current org (avatar + name + plan badge + switcher dropdown)
 * and, when in app context, the current app (icon + name + switcher dropdown).
 * Manages its own dropdown state for org and app switchers.
 *
 * @example
 * <Hook0ContextBar ref="contextBarRef" @close-dropdowns="closeAll" />
 */
import { ref, computed } from 'vue';
import { ChevronsUpDown, Box } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import type { RouteLocationRaw } from 'vue-router';
import { routes } from '@/routes';
import Hook0Avatar from '@/components/Hook0Avatar.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import ContextDropdownMenu from '@/components/ContextDropdownMenu.vue';
import { useOrgAppSwitcher } from '@/composables/useOrgAppSwitcher';

const { t } = useI18n();

const emit = defineEmits<{
  'close-dropdowns': [];
}>();

const {
  currentOrgId,
  currentOrgName,
  currentAppId,
  currentAppName,
  currentOrgPlan,
  orgs,
  apps,
  isAppLevel,
  switchOrg,
  switchApp,
  goToOrgSettings,
  goToAppSettings,
  goToCreateOrg,
  goToCreateApp,
} = useOrgAppSwitcher();

/** Route location for the current organization's dashboard. */
const orgDashboardRoute = computed(() => ({
  name: routes.OrganizationsDashboard,
  params: { organization_id: currentOrgId.value },
}));

/** Route location for the current application's dashboard. */
const appDashboardRoute = computed(() => {
  if (!currentOrgId.value || !currentAppId.value) return undefined;
  return {
    name: routes.ApplicationsDashboard,
    params: { organization_id: currentOrgId.value, application_id: currentAppId.value },
  };
});

/**
 * Sort a list so the item matching currentId comes first, preserving order of the rest.
 * Used to show the active org/app at the top of switcher dropdowns.
 *
 * @example
 * sortCurrentFirst([{id:'a'}, {id:'b'}], x => x.id, 'b') // [{id:'b'}, {id:'a'}]
 */
function sortCurrentFirst<T>(list: T[], getId: (item: T) => string, currentId: string | null): T[] {
  if (!currentId) return list;
  const current: T[] = [];
  const rest: T[] = [];
  for (const item of list) {
    (getId(item) === currentId ? current : rest).push(item);
  }
  return [...current, ...rest];
}

const sortedOrgs = computed(() =>
  sortCurrentFirst(orgs.value ?? [], (o) => o.organization_id, currentOrgId.value)
);

const sortedApps = computed(() =>
  sortCurrentFirst(apps.value ?? [], (a) => a.application_id, currentAppId.value)
);

// Dropdown state — only one of org/app open at a time within this component
type ContextDropdownId = 'org' | 'app';
const activeDropdown = ref<ContextDropdownId | null>(null);

const orgDropdownOpen = computed(() => activeDropdown.value === 'org');
const appDropdownOpen = computed(() => activeDropdown.value === 'app');

/** Close all dropdowns owned by this component. */
function closeDropdowns(): void {
  activeDropdown.value = null;
}

/** Toggle a specific dropdown, closing the other. */
function toggleDropdown(id: ContextDropdownId): void {
  activeDropdown.value = activeDropdown.value === id ? null : id;
  emit('close-dropdowns');
}

/** Handle org item click — switch only if not already the current org. */
function handleOrgItemClick(orgId: string): void {
  if (orgId !== currentOrgId.value) {
    switchOrg(orgId);
  }
  closeDropdowns();
}

const orgTriggerRef = ref<HTMLButtonElement | null>(null);
const appTriggerRef = ref<HTMLButtonElement | null>(null);
const orgDropdownMenuRef = ref<{ dropdownRef: HTMLElement | null } | null>(null);
const appDropdownMenuRef = ref<{ dropdownRef: HTMLElement | null } | null>(null);

/** Extract org plan from an org item. */
function getOrgPlan(item: { name: string; [key: string]: unknown }): { label: string } | null {
  if (item.plan && typeof item.plan === 'object' && 'label' in item.plan) {
    return item.plan as { label: string };
  }
  return null;
}

/**
 * Focus the trigger button that opened the currently active dropdown.
 * Called by the parent when Escape is pressed.
 */
function focusActiveTrigger(): void {
  const triggerMap: Record<ContextDropdownId, HTMLButtonElement | null> = {
    org: orgTriggerRef.value,
    app: appTriggerRef.value,
  };
  if (activeDropdown.value) {
    triggerMap[activeDropdown.value]?.focus();
  }
}

/** Handle app item click — switch only if not already the current app. */
function handleAppItemClick(orgId: string, appId: string): void {
  if (appId !== currentAppId.value) {
    switchApp(orgId, appId);
  }
  closeDropdowns();
}

/** Check whether any dropdown is currently open. */
function hasOpenDropdown(): boolean {
  return activeDropdown.value !== null;
}

/** Build a route for an org dropdown item. */
function orgItemTo(item: { name: string; [key: string]: unknown }): RouteLocationRaw {
  return {
    name: routes.OrganizationsDashboard,
    params: { organization_id: String(item.organization_id) },
  };
}

/** Build a route for an app dropdown item. */
function appItemTo(item: { name: string; [key: string]: unknown }): RouteLocationRaw {
  const orgId = currentOrgId.value;
  if (!orgId) return { name: routes.Home };
  return {
    name: routes.ApplicationsDashboard,
    params: {
      organization_id: orgId,
      application_id: String(item.application_id),
    },
  };
}

defineExpose({ closeDropdowns, focusActiveTrigger, hasOpenDropdown });
</script>

<template>
  <span class="hook0-topnav__logo-separator" aria-hidden="true" />

  <!-- Org section -->
  <div class="hook0-topnav__org-section hook0-topnav__dropdown-anchor">
    <!-- Org selected: link to dashboard + separate switcher button -->
    <template v-if="currentOrgId">
      <router-link
        :to="orgDashboardRoute"
        class="hook0-topnav__context-control hook0-topnav__context-link"
        data-test="context-bar-org-name"
      >
        <Hook0Avatar
          :name="currentOrgName ?? '?'"
          size="sm"
          variant="square"
        />
        <span class="hook0-topnav__context-name">
          {{ currentOrgName ?? '...' }}
        </span>
        <Hook0Badge variant="primary" size="sm" class="hook0-topnav__context-badge">{{
          currentOrgPlan ? currentOrgPlan.label : t('orgAppSelector.developer')
        }}</Hook0Badge>
      </router-link>
      <button
        ref="orgTriggerRef"
        class="hook0-topnav__switcher-btn"
        :aria-label="t('nav.switchOrganization')"
        :aria-expanded="orgDropdownOpen"
        aria-haspopup="true"
        data-test="context-bar-org-switcher"
        @click.stop="toggleDropdown('org')"
      >
        <ChevronsUpDown :size="14" aria-hidden="true" />
      </button>
    </template>
    <!-- No org selected: single unified button -->
    <button
      v-else
      ref="orgTriggerRef"
      class="hook0-topnav__context-control hook0-topnav__context-trigger"
      :aria-label="t('nav.selectOrganization')"
      :aria-expanded="orgDropdownOpen"
      aria-haspopup="true"
      data-test="context-bar-org-switcher"
      @click.stop="toggleDropdown('org')"
    >
      <Hook0Avatar
        name="?"
        size="sm"
        variant="square"
        class="hook0-topnav__ghost-avatar"
      />
      <span class="hook0-topnav__context-ghost-label">
        {{ t('nav.selectOrganization') }}
      </span>
      <ChevronsUpDown :size="14" aria-hidden="true" class="hook0-topnav__context-chevron" />
    </button>

    <!-- Org dropdown -->
    <ContextDropdownMenu
      ref="orgDropdownMenuRef"
      :items="sortedOrgs"
      :current-id="currentOrgId"
      id-key="organization_id"
      :open="orgDropdownOpen"
      :create-label="t('nav.createOrganization')"
      :item-to="orgItemTo"
      @select="handleOrgItemClick"
      @create="goToCreateOrg()"
      @settings="goToOrgSettings"
    >
      <template #icon="{ item }">
        <Hook0Avatar :name="item.name" size="sm" variant="square" />
      </template>
      <template #badge="{ item }">
        <Hook0Badge v-if="getOrgPlan(item)" variant="primary" size="sm">{{
          getOrgPlan(item)?.label
        }}</Hook0Badge>
      </template>
    </ContextDropdownMenu>
  </div>

  <!-- App section (always visible when org is selected) -->
  <template v-if="currentOrgId">
    <span class="hook0-topnav__path-separator" aria-hidden="true">/</span>

    <div class="hook0-topnav__app-section hook0-topnav__dropdown-anchor">
      <!-- App selected: link to dashboard + separate switcher button -->
      <template v-if="isAppLevel && currentAppId">
        <router-link
          :to="appDashboardRoute ?? { name: routes.Home }"
          class="hook0-topnav__context-control hook0-topnav__context-link"
          data-test="context-bar-app-name"
        >
          <Box :size="16" aria-hidden="true" class="hook0-topnav__context-icon--muted" />
          <span class="hook0-topnav__context-name">
            {{ currentAppName ?? '...' }}
          </span>
        </router-link>
        <button
          ref="appTriggerRef"
          class="hook0-topnav__switcher-btn"
          :aria-label="t('nav.switchApplication')"
          :aria-expanded="appDropdownOpen"
          aria-haspopup="true"
          data-test="context-bar-app-switcher"
          @click.stop="toggleDropdown('app')"
        >
          <ChevronsUpDown :size="14" aria-hidden="true" />
        </button>
      </template>
      <!-- No app selected: single unified button -->
      <button
        v-else
        ref="appTriggerRef"
        class="hook0-topnav__context-control hook0-topnav__context-trigger"
        :aria-label="t('nav.selectApplication')"
        :aria-expanded="appDropdownOpen"
        aria-haspopup="true"
        data-test="context-bar-app-switcher"
        @click.stop="toggleDropdown('app')"
      >
        <span class="hook0-topnav__context-ghost-icon">?</span>
        <span class="hook0-topnav__context-ghost-label">
          {{ t('nav.selectApplication') }}
        </span>
        <ChevronsUpDown :size="14" aria-hidden="true" class="hook0-topnav__context-chevron" />
      </button>

      <!-- App dropdown -->
      <ContextDropdownMenu
        ref="appDropdownMenuRef"
        :items="sortedApps"
        :current-id="currentAppId"
        id-key="application_id"
        :open="appDropdownOpen"
        :create-label="t('nav.createApplication')"
        :item-to="appItemTo"
        @select="(id: string) => handleAppItemClick(currentOrgId!, id)"
        @create="goToCreateApp()"
        @settings="(id: string) => goToAppSettings(currentOrgId!, id)"
      >
        <template #icon>
          <Box :size="16" aria-hidden="true" />
        </template>
      </ContextDropdownMenu>
    </div>
  </template>
</template>

<style scoped>
/* Logo separator (rendered here because it's contextual to org presence) */
.hook0-topnav__logo-separator {
  display: block;
  width: 1px;
  height: 1.25rem;
  background-color: var(--color-border);
  flex-shrink: 0;
}

/* Org section */
.hook0-topnav__org-section {
  display: flex;
  align-items: center;
  position: relative;
  min-width: 0;
}

/* Shared base for context link and context trigger */
.hook0-topnav__context-control {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.375rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: none;
  cursor: pointer;
  font-family: inherit;
  min-width: 0;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease;
}

.hook0-topnav__context-control:hover {
  background-color: var(--color-bg-tertiary);
  border-color: var(--color-border-strong);
}

.hook0-topnav__context-control:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Unified context trigger — button-specific reset */
.hook0-topnav__context-trigger {
  appearance: none;
}

/* Name text inside trigger */
.hook0-topnav__context-name {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  max-width: 6rem;
}

@media (min-width: 768px) {
  .hook0-topnav__context-name {
    max-width: 10rem;
  }
}

/* Hide plan badge on mobile to save space */
.hook0-topnav__context-badge {
  display: none;
}

@media (min-width: 768px) {
  .hook0-topnav__context-badge {
    display: inline-flex;
  }
}

/* Context link — link-specific overrides */
.hook0-topnav__context-link {
  text-decoration: none;
  color: inherit;
}

/* Switcher button — standalone chevron to open dropdown, matches link height */
.hook0-topnav__switcher-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  align-self: stretch;
  border: 1px solid var(--color-border);
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  margin-left: 0.25rem;
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease;
  flex-shrink: 0;
}

.hook0-topnav__switcher-btn:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
  border-color: var(--color-border-strong);
}

.hook0-topnav__switcher-btn:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Chevron icon */
.hook0-topnav__context-chevron {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

/* Muted icon (app Box icon) — match avatar height for consistent row sizing */
.hook0-topnav__context-icon--muted {
  color: var(--color-text-muted);
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  padding: 0.1875rem;
}

/* Ghost state (no selection) */
.hook0-topnav__ghost-avatar {
  opacity: 0.35;
}

.hook0-topnav__context-ghost-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  white-space: nowrap;
}

.hook0-topnav__context-ghost-icon {
  color: var(--color-text-muted);
  font-size: 0.875rem;
  font-weight: 500;
  flex-shrink: 0;
}

/* Hide text labels on mobile — keep icons only */
@media (max-width: 767px) {
  .hook0-topnav__context-name,
  .hook0-topnav__context-badge,
  .hook0-topnav__context-ghost-label {
    display: none;
  }
}

/* Path separator between org and app */
.hook0-topnav__path-separator {
  display: block;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  user-select: none;
  flex-shrink: 0;
}

/* App section */
.hook0-topnav__app-section {
  display: flex;
  align-items: center;
  position: relative;
  min-width: 0;
}

@media (prefers-reduced-motion: reduce) {
  .hook0-topnav__context-control,
  .hook0-topnav__switcher-btn {
    transition: none;
  }
}
</style>
