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
const appDashboardRoute = computed(() => ({
  name: routes.ApplicationsDashboard,
  params: { organization_id: currentOrgId.value, application_id: currentAppId.value },
}));

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

/** Extract org name from a generic item (avoids `as` casts in template). */
function getOrgName(item: { [key: string]: unknown }): string {
  return (item.name as string) ?? '?';
}

/** Extract org plan from a generic item (avoids `as` casts in template). */
function getOrgPlan(item: { [key: string]: unknown }): { label: string } | null {
  return (item.plan as { label: string }) ?? null;
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

defineExpose({ closeDropdowns, focusActiveTrigger, hasOpenDropdown });
</script>

<template>
  <span class="hook0-topnav__logo-separator" aria-hidden="true" />

  <!-- Org section -->
  <div class="hook0-topnav__org-section hook0-topnav__dropdown-anchor">
    <!-- Org selected -->
    <template v-if="currentOrgId">
      <router-link
        :to="orgDashboardRoute"
        class="hook0-topnav__org-avatar-btn"
        :aria-label="t('nav.goToOrgDashboard')"
      >
        <Hook0Avatar
          :name="currentOrgName ?? '?'"
          size="sm"
          variant="square"
          class="hook0-topnav__org-avatar"
        />
      </router-link>
      <router-link
        :to="orgDashboardRoute"
        class="hook0-topnav__org-name"
        data-test="context-bar-org-name"
      >
        <span>{{ currentOrgName ?? '...' }}</span>
        <Hook0Badge variant="primary" size="sm">{{
          currentOrgPlan ? currentOrgPlan.label : t('orgAppSelector.developer')
        }}</Hook0Badge>
      </router-link>
    </template>
    <!-- No org selected -->
    <template v-else>
      <Hook0Avatar
        name="?"
        size="sm"
        variant="square"
        class="hook0-topnav__ghost-avatar"
        @click.stop="toggleDropdown('org')"
      />
      <button
        class="hook0-topnav__ghost-select"
        :aria-label="t('nav.selectOrganization')"
        @click.stop="toggleDropdown('org')"
      >
        {{ t('nav.selectOrganization') }}
      </button>
    </template>
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

    <!-- Org dropdown -->
    <ContextDropdownMenu
      ref="orgDropdownMenuRef"
      :items="sortedOrgs"
      :current-id="currentOrgId"
      id-key="organization_id"
      :open="orgDropdownOpen"
      :create-label="t('nav.createOrganization')"
      @select="handleOrgItemClick"
      @create="goToCreateOrg()"
      @settings="goToOrgSettings"
    >
      <template #icon="{ item }">
        <Hook0Avatar :name="getOrgName(item)" size="sm" variant="square" />
      </template>
      <template #badge="{ item }">
        <Hook0Badge v-if="getOrgPlan(item)" variant="primary" size="sm">{{
          getOrgPlan(item)!.label
        }}</Hook0Badge>
      </template>
    </ContextDropdownMenu>
  </div>

  <!-- App section (always visible when org is selected) -->
  <template v-if="currentOrgId">
    <span class="hook0-topnav__path-separator" aria-hidden="true">/</span>

    <div class="hook0-topnav__app-section hook0-topnav__dropdown-anchor">
      <!-- App selected -->
      <template v-if="isAppLevel && currentAppId">
        <router-link
          :to="appDashboardRoute"
          class="hook0-topnav__app-icon-btn hook0-topnav__app-icon-btn--muted"
          :aria-label="t('nav.goToAppDashboard')"
        >
          <Box :size="16" aria-hidden="true" />
        </router-link>
        <router-link
          :to="appDashboardRoute"
          class="hook0-topnav__app-name"
          data-test="context-bar-app-name"
        >
          {{ currentAppName ?? '...' }}
        </router-link>
      </template>
      <!-- No app selected -->
      <template v-else>
        <button
          class="hook0-topnav__ghost-text"
          :aria-label="t('nav.selectApplication')"
          @click.stop="toggleDropdown('app')"
        >
          ?
        </button>
        <button
          class="hook0-topnav__ghost-select"
          :aria-label="t('nav.selectApplication')"
          @click.stop="toggleDropdown('app')"
        >
          {{ t('nav.selectApplication') }}
        </button>
      </template>
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

      <!-- App dropdown -->
      <ContextDropdownMenu
        ref="appDropdownMenuRef"
        :items="sortedApps"
        :current-id="currentAppId"
        id-key="application_id"
        :open="appDropdownOpen"
        :create-label="t('nav.createApplication')"
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
  gap: 0.375rem;
  position: relative;
  min-width: 0;
}

.hook0-topnav__org-name {
  border: none;
  background: none;
  text-decoration: none;
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-primary);
  cursor: pointer;
  padding: 0.25rem 0.375rem;
  border-radius: var(--radius-md);
  transition: background-color 0.15s ease;
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  max-width: 8rem;
  overflow: hidden;
}

@media (min-width: 768px) {
  .hook0-topnav__org-name {
    max-width: 14rem;
  }
}

/* Hide plan badge on mobile to save space */
.hook0-topnav__org-name :deep(.hook0-badge) {
  display: none;
}

@media (min-width: 768px) {
  .hook0-topnav__org-name :deep(.hook0-badge) {
    display: inline-flex;
  }
}

.hook0-topnav__org-name span:first-child {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.hook0-topnav__org-name:hover {
  background-color: var(--color-bg-tertiary);
}

/* Switcher button (chevrons-up-down) */
.hook0-topnav__switcher-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border: 1px solid var(--color-border);
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
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
  gap: 0.375rem;
  position: relative;
  min-width: 0;
}

.hook0-topnav__org-avatar-btn,
.hook0-topnav__app-icon-btn {
  text-decoration: none;
  color: inherit;
  background: none;
}

.hook0-topnav__app-icon-btn {
  border: none;
  padding: 0;
  font: inherit;
  cursor: pointer;
  display: flex;
  align-items: center;
  border-radius: var(--radius-md);
}

.hook0-topnav__app-icon-btn--muted {
  color: var(--color-text-muted);
}

.hook0-topnav__app-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  text-decoration: none;
  white-space: nowrap;
  max-width: 10rem;
  overflow: hidden;
  text-overflow: ellipsis;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.25rem 0.375rem;
  border-radius: var(--radius-md);
  transition: background-color 0.15s ease;
  font-family: inherit;
}

.hook0-topnav__app-name:hover {
  background-color: var(--color-bg-tertiary);
}

/* Hide org and app text names on mobile — keep icons only */
@media (max-width: 767px) {
  .hook0-topnav__org-name {
    display: none;
  }

  .hook0-topnav__app-name {
    display: none;
  }
}

/* Ghost select button (empty state) */
.hook0-topnav__ghost-select {
  display: flex;
  align-items: center;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: none;
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-family: inherit;
  cursor: pointer;
  white-space: nowrap;
  transition:
    border-color 0.15s ease,
    color 0.15s ease;
}

.hook0-topnav__ghost-avatar {
  opacity: 0.35;
  cursor: pointer;
}

.hook0-topnav__ghost-text {
  all: unset;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  flex-shrink: 0;
}

@media (max-width: 767px) {
  .hook0-topnav__ghost-select {
    display: none;
  }
}

.hook0-topnav__ghost-select:hover {
  border-color: var(--color-border-strong);
  color: var(--color-text-secondary);
}

.hook0-topnav__org-name:focus-visible,
.hook0-topnav__switcher-btn:focus-visible,
.hook0-topnav__org-avatar-btn:focus-visible,
.hook0-topnav__app-icon-btn:focus-visible,
.hook0-topnav__app-name:focus-visible,
.hook0-topnav__ghost-select:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

@media (prefers-reduced-motion: reduce) {
  .hook0-topnav__org-name,
  .hook0-topnav__switcher-btn,
  .hook0-topnav__ghost-select {
    transition: none;
  }
}
</style>
