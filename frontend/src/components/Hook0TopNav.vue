<script setup lang="ts">
/**
 * Hook0TopNav - Clerk-style 2-level navigation bar
 *
 * Row 1: Context bar (logo, org/app switcher, right actions)
 * Row 2: Tab bar (navigation tabs from useNavigationTabs)
 *
 * @example
 * <Hook0TopNav />
 */
import { ref, watch, onMounted, onBeforeUnmount, onUnmounted, computed, nextTick } from 'vue';
import type { ComponentPublicInstance } from 'vue';
import { useRouter } from 'vue-router';
import {
  ChevronsUpDown,
  Box,
  Search,
  BookOpen,
  Code2,
  ExternalLink,
  Settings,
  LogOut,
  Sun,
  Moon,
  Menu,
  Plus,
} from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Avatar from '@/components/Hook0Avatar.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import { useNavigationTabs } from '@/composables/useNavigationTabs';
import { useOrgAppSwitcher } from '@/composables/useOrgAppSwitcher';

const { t } = useI18n();
const router = useRouter();
const authStore = useAuthStore();
const uiStore = useUiStore();

const { navTabs } = useNavigationTabs();

const isMac = computed(() => {
  const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
  return /mac/i.test(nav.userAgentData?.platform ?? navigator.platform);
});

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

/** Sort a list so the item matching currentId comes first. */
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

const userTriggerRef = ref<HTMLButtonElement | null>(null);

// Tab indicator sliding
const TAB_PADDING_X = 12;
const tabsNavRef = ref<HTMLElement | null>(null);
const activeTabEl = ref<HTMLElement | null>(null);
const tabIndicatorStyle = ref<Record<string, string>>({ opacity: '0' });

function updateTabIndicator() {
  void nextTick(() => {
    if (!activeTabEl.value || !tabsNavRef.value) {
      tabIndicatorStyle.value = { opacity: '0' };
      return;
    }
    const navRect = tabsNavRef.value.getBoundingClientRect();
    const tabRect = activeTabEl.value.getBoundingClientRect();
    tabIndicatorStyle.value = {
      width: `${tabRect.width - TAB_PADDING_X * 2}px`,
      transform: `translateX(${tabRect.left - navRect.left + TAB_PADDING_X}px)`,
      opacity: '1',
    };
  });
}

watch(() => navTabs.value.find((tab) => tab.active)?.id, updateTabIndicator);

let tabsResizeObserver: ResizeObserver | null = null;

onMounted(() => {
  tabsResizeObserver = new ResizeObserver(() => {
    updateTabIndicator();
  });
  if (tabsNavRef.value) {
    tabsResizeObserver.observe(tabsNavRef.value);
  }
});

watch(tabsNavRef, (el) => {
  if (el && tabsResizeObserver) {
    tabsResizeObserver.observe(el);
  }
});

// Dropdown state machine — only one dropdown open at a time
type DropdownId = 'org' | 'app' | 'user';
const activeDropdown = ref<DropdownId | null>(null);

const orgDropdownOpen = computed(() => activeDropdown.value === 'org');
const appDropdownOpen = computed(() => activeDropdown.value === 'app');
const userDropdownOpen = computed(() => activeDropdown.value === 'user');
const anyDropdownOpen = computed(() => activeDropdown.value !== null);

/** Close all open dropdowns. */
function closeDropdowns(): void {
  activeDropdown.value = null;
}

/** Toggle a specific dropdown, closing others. */
function toggleDropdown(id: DropdownId): void {
  activeDropdown.value = activeDropdown.value === id ? null : id;
}

/** Handle org item click — switch only if not already the current org. */
function handleOrgItemClick(orgId: string): void {
  if (orgId !== currentOrgId.value) {
    switchOrg(orgId);
  }
}

const orgTriggerRef = ref<HTMLButtonElement | null>(null);
const appTriggerRef = ref<HTMLButtonElement | null>(null);

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && anyDropdownOpen.value) {
    event.preventDefault();
    const triggerMap: Record<DropdownId, HTMLButtonElement | null> = {
      org: orgTriggerRef.value,
      app: appTriggerRef.value,
      user: userTriggerRef.value,
    };
    const trigger = activeDropdown.value ? triggerMap[activeDropdown.value] : null;
    closeDropdowns();
    trigger?.focus();
  }
}

/** Close dropdowns when clicking outside. */
function onDocumentClick(event: MouseEvent): void {
  const target = event.target as HTMLElement;
  if (!target.closest('.hook0-topnav__dropdown-anchor')) {
    closeDropdowns();
  }
}

watch(anyDropdownOpen, (isOpen) => {
  if (isOpen) {
    document.addEventListener('click', onDocumentClick, { capture: true });
  } else {
    document.removeEventListener('click', onDocumentClick, { capture: true });
  }
});

onMounted(() => {
  document.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown);
  document.removeEventListener('click', onDocumentClick, { capture: true });
  tabsResizeObserver?.disconnect();
  tabsResizeObserver = null;
});

// Close dropdowns on route change
const removeAfterEach = router.afterEach(() => {
  closeDropdowns();
});
onUnmounted(removeAfterEach);
</script>

<template>
  <header class="hook0-topnav">
    <!-- Row 1: Context Bar -->
    <div class="hook0-topnav__context-bar">
      <!-- Mobile menu button -->
      <button
        class="hook0-topnav__mobile-toggle"
        :aria-label="t('nav.openMenu')"
        @click.stop="uiStore.toggleMobileDrawer()"
      >
        <Menu :size="20" aria-hidden="true" />
      </button>

      <!-- Logo -->
      <router-link :to="{ name: routes.Home }" class="hook0-topnav__logo-section">
        <Hook0Logo variant="image" size="sm" />
      </router-link>

      <!-- Org/App context (hidden on mobile) -->
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

      <!-- Right section -->
      <div class="hook0-topnav__right">
        <!-- Search -->
        <button
          class="hook0-topnav__search"
          :aria-label="t('nav.search')"
          @click="uiStore.openCommandPalette()"
        >
          <Search :size="16" aria-hidden="true" />
          <span class="hook0-topnav__search-text">{{ t('nav.search') }}</span>
          <kbd class="hook0-topnav__search-kbd">{{ isMac ? '\u2318' : 'Ctrl+' }}K</kbd>
        </button>

        <!-- Documentation -->
        <a
          href="https://documentation.hook0.com/"
          target="_blank"
          rel="noopener noreferrer"
          class="hook0-topnav__nav-link"
        >
          <BookOpen :size="16" aria-hidden="true" />
          <span class="hook0-topnav__nav-link-text">{{ t('nav.documentation') }}</span>
          <ExternalLink :size="12" class="hook0-topnav__nav-link-external" aria-hidden="true" />
        </a>

        <!-- API Reference -->
        <a
          href="https://documentation.hook0.com/api"
          target="_blank"
          rel="noopener noreferrer"
          class="hook0-topnav__nav-link"
        >
          <Code2 :size="16" aria-hidden="true" />
          <span class="hook0-topnav__nav-link-text">{{ t('nav.apiReference') }}</span>
          <ExternalLink :size="12" class="hook0-topnav__nav-link-external" aria-hidden="true" />
        </a>

        <!-- User Menu -->
        <div class="hook0-topnav__dropdown-anchor">
          <button
            ref="userTriggerRef"
            class="hook0-topnav__user-trigger"
            :aria-expanded="userDropdownOpen"
            aria-haspopup="true"
            :aria-label="t('nav.userMenu')"
            @click.stop="toggleDropdown('user')"
          >
            <div class="hook0-topnav__user-avatar">
              {{ authStore.userInfo?.email?.charAt(0)?.toUpperCase() ?? '?' }}
            </div>
          </button>

          <Transition name="dropdown">
            <div
              v-if="userDropdownOpen"
              class="hook0-topnav__dropdown hook0-topnav__user-dropdown"
              role="menu"
              aria-orientation="vertical"
            >
              <div class="hook0-topnav__dropdown-user-info">
                <div class="hook0-topnav__dropdown-user-email">
                  {{ authStore.userInfo?.email }}
                </div>
              </div>
              <div class="hook0-topnav__dropdown-separator" />
              <router-link
                :to="{ name: routes.UserSettings }"
                class="hook0-topnav__dropdown-item"
                role="menuitem"
              >
                <Settings :size="16" aria-hidden="true" />
                {{ t('nav.settings') }}
              </router-link>
              <Hook0Button
                variant="ghost"
                class="hook0-topnav__dropdown-item"
                role="menuitem"
                @click="uiStore.toggleColorMode()"
              >
                <Sun v-if="uiStore.effectiveColorMode === 'dark'" :size="16" aria-hidden="true" />
                <Moon v-else :size="16" aria-hidden="true" />
                {{ uiStore.effectiveColorMode === 'dark' ? t('nav.lightMode') : t('nav.darkMode') }}
              </Hook0Button>
              <div class="hook0-topnav__dropdown-separator" />
              <Hook0Button
                variant="ghost"
                class="hook0-topnav__dropdown-item hook0-topnav__dropdown-item--danger"
                role="menuitem"
                @click="void authStore.logout()"
              >
                <LogOut :size="16" aria-hidden="true" />
                {{ t('nav.logout') }}
              </Hook0Button>
            </div>
          </Transition>
        </div>
      </div>
    </div>

    <!-- Row 2: Tab Bar -->
    <nav
      v-if="navTabs.length > 0"
      ref="tabsNavRef"
      class="hook0-topnav__tabs"
      :aria-label="t('nav.tabBar')"
    >
      <router-link
        v-for="tab in navTabs"
        :key="tab.id"
        :ref="
          (el) => {
            if (tab.active && el)
              activeTabEl = (el as ComponentPublicInstance)?.$el ?? (el as HTMLElement);
          }
        "
        :to="tab.to"
        class="hook0-topnav__tab"
        :class="{ 'hook0-topnav__tab--active': tab.active }"
        :title="tab.label"
      >
        <component :is="tab.icon" :size="16" class="hook0-topnav__tab-icon" aria-hidden="true" />
        <span class="hook0-topnav__tab-label">{{ tab.label }}</span>
        <span v-if="tab.badge" class="hook0-topnav__tab-badge">{{ tab.badge }}</span>
      </router-link>
      <div class="hook0-topnav__tab-indicator" :style="tabIndicatorStyle" />
    </nav>
  </header>
</template>

<style scoped>
/* ==========================================================================
   Hook0TopNav — 2-level navigation
   ========================================================================== */

.hook0-topnav {
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  position: sticky;
  top: 0;
  z-index: var(--z-topnav, 30);
}

/* --------------------------------------------------------------------------
   Row 1: Context Bar
   -------------------------------------------------------------------------- */

.hook0-topnav__context-bar {
  display: flex;
  align-items: center;
  height: 3rem;
  padding: 0 1rem;
  gap: 0.5rem;
}

/* Mobile toggle */
.hook0-topnav__mobile-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: none;
  background: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.hook0-topnav__mobile-toggle:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-topnav__mobile-toggle:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

@media (min-width: 768px) {
  .hook0-topnav__mobile-toggle {
    display: none;
  }
}

/* Logo section */
.hook0-topnav__logo-section {
  display: flex;
  align-items: center;
  text-decoration: none;
  flex-shrink: 0;
}

.hook0-topnav__logo-section:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-md);
}

/* Vertical separator after logo */
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

/* Right section */
.hook0-topnav__right {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  margin-left: auto;
}

/* Search */
.hook0-topnav__search {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-secondary);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: border-color 0.15s ease;
  font-size: 0.8125rem;
}

.hook0-topnav__search:hover {
  border-color: var(--color-border-strong);
}

.hook0-topnav__search:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-topnav__search-text {
  display: none;
}

@media (min-width: 640px) {
  .hook0-topnav__search-text {
    display: inline;
  }

  .hook0-topnav__search {
    min-width: 10rem;
  }
}

.hook0-topnav__search-kbd {
  display: none;
  padding: 0.125rem 0.375rem;
  font-size: 0.6875rem;
  font-family: var(--font-mono);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  margin-left: auto;
}

@media (min-width: 768px) {
  .hook0-topnav__search-kbd {
    display: inline;
  }
}

/* Nav links (Documentation, API Reference) */
.hook0-topnav__nav-link {
  display: none;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.625rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  text-decoration: none;
  border-radius: var(--radius-md);
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
  white-space: nowrap;
  flex-shrink: 0;
}

@media (min-width: 768px) {
  .hook0-topnav__nav-link {
    display: flex;
  }
}

.hook0-topnav__nav-link:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-tertiary);
}

.hook0-topnav__nav-link:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-topnav__nav-link-text {
  display: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (min-width: 1280px) {
  .hook0-topnav__nav-link-text {
    display: inline;
  }
}

.hook0-topnav__nav-link-external {
  color: var(--color-text-muted);
  display: none;
}

@media (min-width: 1280px) {
  .hook0-topnav__nav-link-external {
    display: inline;
  }
}

/* User avatar trigger */
.hook0-topnav__user-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  cursor: pointer;
  padding: 0;
}

.hook0-topnav__user-trigger:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-full);
}

.hook0-topnav__user-avatar {
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--radius-full);
  background: linear-gradient(
    135deg,
    var(--color-primary),
    color-mix(in srgb, var(--color-primary) 70%, var(--color-text-primary))
  );
  color: var(--color-primary-text, #fff);
  font-size: 0.6875rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: box-shadow 0.15s ease;
}

.hook0-topnav__user-trigger:hover .hook0-topnav__user-avatar {
  box-shadow:
    0 0 0 2px var(--color-bg-primary),
    0 0 0 4px var(--color-primary);
}

/* --------------------------------------------------------------------------
   Dropdown shared styles
   -------------------------------------------------------------------------- */

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

.hook0-topnav__user-dropdown {
  left: auto;
  right: 0;
  min-width: 12rem;
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

.hook0-topnav__dropdown-item--danger {
  color: var(--color-error);
}

.hook0-topnav__dropdown-item--danger:hover {
  background-color: var(--color-error-light);
  color: var(--color-error);
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

.hook0-topnav__dropdown-user-info {
  padding: 0.625rem 0.75rem;
}

.hook0-topnav__dropdown-user-email {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* --------------------------------------------------------------------------
   Row 2: Tab Bar
   -------------------------------------------------------------------------- */

.hook0-topnav__tabs {
  display: none;
  align-items: center;
  gap: 0.125rem;
  padding: 0 1rem;
  border-top: 1px solid var(--color-border);
  position: relative;
}

@media (min-width: 768px) {
  .hook0-topnav__tabs {
    display: flex;
  }
}

.hook0-topnav__tab {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.625rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  text-decoration: none;
  position: relative;
  transition: color 0.15s ease;
  white-space: nowrap;
}

.hook0-topnav__tab:hover {
  color: var(--color-text-primary);
}

.hook0-topnav__tab:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
  border-radius: var(--radius-md);
}

/* Active tab */
.hook0-topnav__tab--active {
  color: var(--color-primary);
}

/* Sliding indicator */
.hook0-topnav__tab-indicator {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 2px;
  background-color: var(--color-primary);
  border-radius: 1px;
  transition:
    transform 0.25s cubic-bezier(0.4, 0, 0.2, 1),
    width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

@media (prefers-reduced-motion: reduce) {
  .hook0-topnav__tab-indicator {
    transition: none;
  }
}

.hook0-topnav__tab-icon {
  flex-shrink: 0;
}

.hook0-topnav__tab-label {
  display: none;
}

@media (min-width: 1280px) {
  .hook0-topnav__tab-label {
    display: inline;
  }
}

.hook0-topnav__tab-badge {
  padding: 0.0625rem 0.375rem;
  font-size: 0.6875rem;
  font-weight: 600;
  border-radius: var(--radius-full);
  background-color: var(--color-primary);
  color: var(--color-primary-text);
}

/* --------------------------------------------------------------------------
   Dropdown animation
   -------------------------------------------------------------------------- */

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

  .hook0-topnav__mobile-toggle,
  .hook0-topnav__org-name,
  .hook0-topnav__switcher-btn,
  .hook0-topnav__search,
  .hook0-topnav__nav-link,
  .hook0-topnav__user-avatar,
  .hook0-topnav__dropdown-item,
  .hook0-topnav__tab {
    transition: none;
  }
}
</style>
