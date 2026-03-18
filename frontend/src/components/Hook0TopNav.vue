<script setup lang="ts">
/**
 * Hook0TopNav - Clerk-style 2-level navigation bar (thin shell).
 *
 * Row 1: Context bar (logo, org/app switcher via Hook0ContextBar, right actions, user menu via Hook0UserMenu)
 * Row 2: Tab bar (navigation tabs from useNavigationTabs)
 *
 * Coordinates dropdown closing across children (click-outside, Escape, route changes).
 *
 * @example
 * <Hook0TopNav />
 */
import { ref, watch, onMounted, onBeforeUnmount, onUnmounted, computed, nextTick } from 'vue';
import type { ComponentPublicInstance } from 'vue';
import { useRouter } from 'vue-router';
import { Search, BookOpen, Code2, ExternalLink } from 'lucide-vue-next';
import { routes } from '@/routes';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0ContextBar from '@/components/Hook0TopNavContextBar.vue';
import Hook0UserMenu from '@/components/Hook0TopNavUserMenu.vue';
import { useNavigationTabs } from '@/composables/useNavigationTabs';

const { t } = useI18n();
const router = useRouter();
const uiStore = useUiStore();

const { navTabs } = useNavigationTabs();

const isMac = computed(() => {
  const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
  return /mac/i.test(nav.userAgentData?.platform ?? navigator.platform);
});

// Child component refs for coordinated close
type ContextBarExposed = { closeDropdowns: () => void; focusActiveTrigger: () => void };
type UserMenuExposed = { closeDropdowns: () => void; focusTrigger: () => void };
const contextBarRef = ref<ContextBarExposed | null>(null);
const userMenuRef = ref<UserMenuExposed | null>(null);

// Tab indicator sliding
const TAB_PADDING_X = 12;
const tabsNavRef = ref<HTMLElement | null>(null);
const activeTabEl = ref<HTMLElement | null>(null);
const tabIndicatorStyle = ref<Record<string, string>>({ opacity: '0' });

/**
 * Recalculate the sliding tab indicator position and width based on the active tab's DOM rect.
 * Called on tab change (watcher) and on layout changes (ResizeObserver).
 */
function updateTabIndicator() {
  // nextTick: wait for Vue to flush DOM updates so getBoundingClientRect reads the final position
  void nextTick(() => {
    if (!activeTabEl.value || !tabsNavRef.value) {
      tabIndicatorStyle.value = { opacity: '0' };
      return;
    }
    const navRect = tabsNavRef.value.getBoundingClientRect();
    const tabRect = activeTabEl.value.getBoundingClientRect();
    const scrollLeft = tabsNavRef.value.scrollLeft;
    tabIndicatorStyle.value = {
      width: `${tabRect.width - TAB_PADDING_X * 2}px`,
      transform: `translateX(${tabRect.left - navRect.left + scrollLeft + TAB_PADDING_X}px)`,
      opacity: '1',
    };
  });
}

watch(() => navTabs.value.find((tab) => tab.active)?.id, updateTabIndicator);

let tabsResizeObserver: ResizeObserver | null = null;

function onTabsScroll() {
  updateTabIndicator();
}

onMounted(() => {
  tabsResizeObserver = new ResizeObserver(() => {
    updateTabIndicator();
  });
  if (tabsNavRef.value) {
    tabsResizeObserver.observe(tabsNavRef.value);
    tabsNavRef.value.addEventListener('scroll', onTabsScroll, { passive: true });
  }
});

watch(tabsNavRef, (el) => {
  if (el && tabsResizeObserver) {
    tabsResizeObserver.observe(el);
  }
});

/** Close all dropdowns in both children. */
function closeAll(): void {
  contextBarRef.value?.closeDropdowns();
  userMenuRef.value?.closeDropdowns();
}

/**
 * When a child emits close-dropdowns, close the OTHER child's dropdowns.
 * The emitting child handles its own toggle internally.
 */
function onContextBarCloseDropdowns(): void {
  userMenuRef.value?.closeDropdowns();
}

/** When user menu emits close-dropdowns, close context bar dropdowns. */
function onUserMenuCloseDropdowns(): void {
  contextBarRef.value?.closeDropdowns();
}

/** Handle Escape key — close all dropdowns and refocus the trigger. */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    // Try to focus the active trigger in context bar first, then user menu
    contextBarRef.value?.focusActiveTrigger();
    userMenuRef.value?.focusTrigger();
    closeAll();
    event.preventDefault();
  }
}

/** Close dropdowns when clicking outside any dropdown anchor. */
function onDocumentClick(event: MouseEvent): void {
  const target = event.target as HTMLElement;
  if (!target.closest('.hook0-topnav__dropdown-anchor')) {
    closeAll();
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown);
  document.addEventListener('click', onDocumentClick, { capture: true });
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown);
  document.removeEventListener('click', onDocumentClick, { capture: true });
  tabsNavRef.value?.removeEventListener('scroll', onTabsScroll);
  tabsResizeObserver?.disconnect();
  tabsResizeObserver = null;
});

// Close dropdowns on route change
const removeAfterEach = router.afterEach(() => {
  closeAll();
});
onUnmounted(removeAfterEach);
</script>

<template>
  <header class="hook0-topnav">
    <!-- Row 1: Context Bar -->
    <div class="hook0-topnav__context-bar">
      <!-- Logo -->
      <router-link :to="{ name: routes.Home }" class="hook0-topnav__logo-section">
        <Hook0Logo variant="image" size="sm" />
      </router-link>

      <!-- Org/App context -->
      <Hook0ContextBar ref="contextBarRef" @close-dropdowns="onContextBarCloseDropdowns" />

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
        <Hook0UserMenu ref="userMenuRef" @close-dropdowns="onUserMenuCloseDropdowns" />
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
   Hook0TopNav — 2-level navigation (shell)
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

/* --------------------------------------------------------------------------
   Row 2: Tab Bar
   -------------------------------------------------------------------------- */

.hook0-topnav__tabs {
  display: flex;
  align-items: center;
  gap: 0.125rem;
  padding: 0 1rem;
  border-top: 1px solid var(--color-border);
  position: relative;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none; /* Firefox */
  flex-wrap: nowrap;
}

.hook0-topnav__tabs::-webkit-scrollbar {
  display: none; /* Chrome/Safari */
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
  flex-shrink: 0;
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

@media (min-width: 768px) {
  .hook0-topnav__tab-label {
    display: inline;
  }
}

.hook0-topnav__tab--active .hook0-topnav__tab-label {
  display: inline;
}

.hook0-topnav__tab-badge {
  padding: 0.0625rem 0.375rem;
  font-size: 0.6875rem;
  font-weight: 600;
  border-radius: var(--radius-full);
  background-color: var(--color-primary);
  color: var(--color-primary-text);
}

@media (prefers-reduced-motion: reduce) {
  .hook0-topnav__search,
  .hook0-topnav__nav-link,
  .hook0-topnav__tab {
    transition: none;
  }
}
</style>
