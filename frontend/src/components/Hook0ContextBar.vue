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
import { ChevronsUpDown, Box, Plus } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Avatar from '@/components/Hook0Avatar.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
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
  goToOrgDashboard,
  goToOrgSettings,
  goToAppSettings,
  goToCreateOrg,
  goToCreateApp,
} = useOrgAppSwitcher();

/**
 * Sort a list so the item matching currentId comes first, preserving order of the rest.
 * Used to show the active org/app at the top of switcher dropdowns.
 *
 * @example
 * sortCurrentFirst([{id:'a'}, {id:'b'}], x => x.id, 'b') // [{id:'b'}, {id:'a'}]
 */
function sortCurrentFirst<T>(list: T[], getId: (item: T) => string, currentId: string | null): T[] {
  return [...list].sort((a, b) => {
    if (getId(a) === currentId) return -1;
    if (getId(b) === currentId) return 1;
    return 0;
  });
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
}

const orgTriggerRef = ref<HTMLButtonElement | null>(null);
const appTriggerRef = ref<HTMLButtonElement | null>(null);

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

defineExpose({ closeDropdowns, focusActiveTrigger });
</script>

<template>
  <template v-if="currentOrgId">
    <span class="hook0-topnav__logo-separator" aria-hidden="true" />

    <!-- Org section -->
    <div class="hook0-topnav__org-section hook0-topnav__dropdown-anchor">
      <Hook0Avatar :name="currentOrgName ?? '?'" size="sm" variant="square" />
      <button
        class="hook0-topnav__org-name"
        data-test="context-bar-org-name"
        @click="goToOrgDashboard()"
      >
        <span>{{ currentOrgName ?? '...' }}</span>
        <Hook0Badge variant="primary" size="sm">{{
          currentOrgPlan ? currentOrgPlan.label : t('orgAppSelector.developer')
        }}</Hook0Badge>
      </button>
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
      <Transition name="dropdown">
        <div
          v-if="orgDropdownOpen"
          class="hook0-topnav__dropdown"
          role="menu"
          aria-orientation="vertical"
        >
          <button
            v-for="org in sortedOrgs"
            :key="org.organization_id"
            class="hook0-topnav__dropdown-item"
            :class="{
              'hook0-topnav__dropdown-item--active': org.organization_id === currentOrgId,
            }"
            role="menuitem"
            @click="handleOrgItemClick(org.organization_id)"
          >
            <Hook0Avatar :name="org.name" size="sm" variant="square" />
            <div class="hook0-topnav__dropdown-item-content">
              <span class="hook0-topnav__dropdown-item-name">
                <span class="hook0-topnav__dropdown-item-name-text">{{ org.name }}</span>
                <Hook0Badge v-if="org.plan" variant="primary" size="sm">{{
                  org.plan.label
                }}</Hook0Badge>
              </span>
              <span
                v-if="org.organization_id === currentOrgId"
                class="hook0-topnav__dropdown-item-meta"
              >
                {{ t('common.current') }}
              </span>
            </div>
            <Hook0Button
              v-if="org.organization_id === currentOrgId"
              variant="secondary"
              size="xs"
              :aria-label="`${t('nav.settings')} ${org.name}`"
              @click.stop="goToOrgSettings(org.organization_id)"
            >
              {{ t('nav.settings') }}
            </Hook0Button>
          </button>

          <div class="hook0-topnav__dropdown-separator" />

          <button
            class="hook0-topnav__dropdown-item hook0-topnav__dropdown-item--create"
            role="menuitem"
            @click="goToCreateOrg()"
          >
            <Plus :size="16" aria-hidden="true" />
            {{ t('nav.createOrganization') }}
          </button>
        </div>
      </Transition>
    </div>

    <!-- App section (when in app context) -->
    <template v-if="isAppLevel">
      <span class="hook0-topnav__path-separator" aria-hidden="true">/</span>

      <div class="hook0-topnav__app-section hook0-topnav__dropdown-anchor">
        <Box :size="16" class="hook0-topnav__app-icon" aria-hidden="true" />
        <button
          class="hook0-topnav__app-name"
          data-test="context-bar-app-name"
          @click="switchApp(currentOrgId!, currentAppId!)"
        >
          {{ currentAppName ?? '...' }}
        </button>
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
        <Transition name="dropdown">
          <div
            v-if="appDropdownOpen"
            class="hook0-topnav__dropdown"
            role="menu"
            aria-orientation="vertical"
          >
            <button
              v-for="app in sortedApps"
              :key="app.application_id"
              class="hook0-topnav__dropdown-item"
              :class="{
                'hook0-topnav__dropdown-item--active': app.application_id === currentAppId,
              }"
              role="menuitem"
              @click="
                app.application_id !== currentAppId
                  ? switchApp(app.organization_id, app.application_id)
                  : undefined
              "
            >
              <Box :size="16" aria-hidden="true" />
              <div class="hook0-topnav__dropdown-item-content">
                <span class="hook0-topnav__dropdown-item-name">{{ app.name }}</span>
                <span
                  v-if="app.application_id === currentAppId"
                  class="hook0-topnav__dropdown-item-meta"
                >
                  {{ t('common.current') }}
                </span>
              </div>
              <Hook0Button
                v-if="app.application_id === currentAppId"
                variant="secondary"
                size="xs"
                :aria-label="`${t('nav.settings')} ${app.name}`"
                @click.stop="goToAppSettings(app.organization_id, app.application_id)"
              >
                {{ t('nav.settings') }}
              </Hook0Button>
            </button>

            <div class="hook0-topnav__dropdown-separator" />

            <button
              class="hook0-topnav__dropdown-item hook0-topnav__dropdown-item--create"
              role="menuitem"
              @click="goToCreateApp()"
            >
              <Plus :size="16" aria-hidden="true" />
              {{ t('nav.createApplication') }}
            </button>
          </div>
        </Transition>
      </div>
    </template>
  </template>
</template>

<style scoped>
/* Logo separator (rendered here because it's contextual to org presence) */
.hook0-topnav__logo-separator {
  display: none;
  width: 1px;
  height: 1.25rem;
  background-color: var(--color-border);
  flex-shrink: 0;
}

@media (min-width: 768px) {
  .hook0-topnav__logo-separator {
    display: block;
  }
}

/* Org section */
.hook0-topnav__org-section {
  display: none;
  align-items: center;
  gap: 0.375rem;
  position: relative;
}

@media (min-width: 768px) {
  .hook0-topnav__org-section {
    display: flex;
  }
}

.hook0-topnav__org-name {
  border: none;
  background: none;
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
  max-width: 14rem;
  overflow: hidden;
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

.hook0-topnav__org-name:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
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

.hook0-topnav__switcher-btn:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Path separator between org and app */
.hook0-topnav__path-separator {
  display: none;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  user-select: none;
  flex-shrink: 0;
}

@media (min-width: 768px) {
  .hook0-topnav__path-separator {
    display: block;
  }
}

/* App section */
.hook0-topnav__app-section {
  display: none;
  align-items: center;
  gap: 0.375rem;
  position: relative;
}

@media (min-width: 768px) {
  .hook0-topnav__app-section {
    display: flex;
  }
}

.hook0-topnav__app-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.hook0-topnav__app-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
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

.hook0-topnav__app-name:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Dropdown shared styles */
.hook0-topnav__dropdown-anchor {
  position: relative;
}

.hook0-topnav__dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  left: 0;
  min-width: 16rem;
  max-width: 20rem;
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: 0.375rem;
  z-index: var(--z-dropdown, 50);
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.hook0-topnav__dropdown-item {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  justify-content: flex-start;
  gap: 0.625rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  text-decoration: none;
  border: none;
  background: none;
  cursor: pointer;
  border-bottom: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
  width: 100%;
  text-align: left;
  white-space: nowrap;
}

.hook0-topnav__dropdown-item :deep(svg) {
  flex-shrink: 0;
}

.hook0-topnav__dropdown-item:not(.hook0-topnav__dropdown-item--active):hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-topnav__dropdown-item:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.hook0-topnav__dropdown-item--active {
  background-color: transparent;
  border-radius: 0;
  cursor: default;
}

.hook0-topnav__dropdown-item--create {
  color: var(--color-text-muted);
  font-weight: 400;
  border-bottom: none;
}

.hook0-topnav__dropdown-item:has(+ .hook0-topnav__dropdown-separator) {
  border-bottom: none;
}

.hook0-topnav__dropdown-item--create :deep(svg) {
  border: 1.5px dashed var(--color-border-strong);
  border-radius: var(--radius-sm);
  padding: 1px;
}

.hook0-topnav__dropdown-item--create:hover {
  color: var(--color-text-primary);
}

.hook0-topnav__dropdown-item-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.0625rem;
}

.hook0-topnav__dropdown-item-name {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-weight: 500;
  color: var(--color-text-primary);
  min-width: 0;
  flex: 1;
}

.hook0-topnav__dropdown-item-name-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.hook0-topnav__dropdown-item-meta {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
}

.hook0-topnav__dropdown-separator {
  height: 1px;
  background-color: var(--color-border);
  margin: 0.125rem 0;
}

/* Dropdown animation */
.dropdown-enter-active,
.dropdown-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-0.25rem);
}

@media (prefers-reduced-motion: reduce) {
  .dropdown-enter-active,
  .dropdown-leave-active {
    transition: none;
  }

  .hook0-topnav__org-name,
  .hook0-topnav__switcher-btn,
  .hook0-topnav__dropdown-item {
    transition: none;
  }
}
</style>
