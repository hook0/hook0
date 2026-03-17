<script setup lang="ts">
/**
 * Hook0TopNav - Stripe-inspired top navigation bar
 *
 * Features:
 * - Horizontal layout with logo, nav tabs, and user menu
 * - Navigation tabs adapt based on context (org-level vs app-level)
 * - Integrated search trigger (Cmd+K)
 * - Clean, minimal design like Stripe Dashboard
 */
import { ref, watch, onMounted, onBeforeUnmount, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import {
  Search,
  BookOpen,
  Code2,
  ExternalLink,
  Settings,
  LogOut,
  Sun,
  Moon,
  Menu,
} from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import { useNavigationTabs } from '@/composables/useNavigationTabs';

const { t } = useI18n();
const router = useRouter();
const authStore = useAuthStore();
const uiStore = useUiStore();

const { navTabs } = useNavigationTabs();

const userTriggerRef = ref<HTMLButtonElement | null>(null);

// Dropdown states
const userDropdownOpen = ref(false);

function closeDropdowns() {
  userDropdownOpen.value = false;
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && userDropdownOpen.value) {
    event.preventDefault();
    closeDropdowns();
    userTriggerRef.value?.focus();
  }
}

// Click-outside detection: attach to document when dropdown opens
function onDocumentClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  if (!target.closest('.hook0-dropdown')) {
    closeDropdowns();
  }
}

watch(userDropdownOpen, (isOpen) => {
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
});

// Close dropdowns on route change
const removeAfterEach = router.afterEach(() => {
  closeDropdowns();
});
onUnmounted(removeAfterEach);
</script>

<template>
  <header class="hook0-topnav">
    <!-- Mobile menu button -->
    <button
      class="hook0-topnav__mobile-toggle"
      :aria-label="t('nav.openMenu')"
      @click.stop="uiStore.toggleMobileDrawer()"
    >
      <Menu :size="20" aria-hidden="true" />
    </button>

    <!-- Logo -->
    <router-link :to="{ name: routes.Home }" class="hook0-topnav__logo">
      <Hook0Logo variant="image" size="sm" />
    </router-link>

    <!-- Navigation Tabs -->
    <nav
      v-if="navTabs.length > 0"
      class="hook0-topnav__tabs"
      :aria-label="t('common.mainNavigation')"
    >
      <router-link
        v-for="tab in navTabs"
        :key="tab.id"
        :to="tab.to"
        class="hook0-topnav__tab"
        :class="{ active: tab.active }"
        :title="tab.label"
      >
        <component :is="tab.icon" :size="16" class="hook0-topnav__tab-icon" aria-hidden="true" />
        <span class="hook0-topnav__tab-label">{{ tab.label }}</span>
        <span v-if="tab.badge" class="hook0-topnav__tab-badge">{{ tab.badge }}</span>
      </router-link>
    </nav>

    <!-- Spacer -->
    <div class="hook0-topnav__spacer" />

    <!-- Search -->
    <button
      class="hook0-topnav__search"
      :aria-label="t('nav.search')"
      @click="uiStore.openCommandPalette()"
    >
      <Search :size="16" aria-hidden="true" />
      <span class="hook0-topnav__search-text">{{ t('nav.search') }}</span>
      <kbd class="hook0-topnav__search-kbd">⌘K</kbd>
    </button>

    <!-- Documentation & API Reference Links -->
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
    <div class="hook0-dropdown hook0-topnav__user">
      <button
        ref="userTriggerRef"
        class="hook0-topnav__user-trigger"
        :aria-expanded="userDropdownOpen"
        aria-haspopup="true"
        :aria-label="t('nav.userMenu')"
        @click.stop="userDropdownOpen = !userDropdownOpen"
      >
        <div class="hook0-topnav__user-avatar">
          {{ authStore.userInfo?.email?.charAt(0)?.toUpperCase() ?? '?' }}
        </div>
      </button>

      <Transition name="dropdown">
        <div
          v-if="userDropdownOpen"
          class="hook0-topnav__dropdown hook0-topnav__dropdown--right hook0-topnav__dropdown--user"
          role="menu"
          aria-orientation="vertical"
        >
          <div class="hook0-topnav__dropdown-user-info">
            <div class="hook0-topnav__dropdown-user-email">{{ authStore.userInfo?.email }}</div>
          </div>
          <div class="hook0-topnav__dropdown-divider" />
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
          <div class="hook0-topnav__dropdown-divider" />
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
  </header>
</template>

<style scoped>
.hook0-topnav {
  display: flex;
  align-items: center;
  height: 3.5rem;
  padding: 0 1rem;
  background-color: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  gap: 0.5rem;
  position: sticky;
  top: 0;
  z-index: var(--z-topnav, 30);
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

@media (min-width: 768px) {
  .hook0-topnav__mobile-toggle {
    display: none;
  }
}

/* Logo */
.hook0-topnav__logo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  text-decoration: none;
  flex-shrink: 0;
}

.hook0-topnav__logo-icon {
  color: var(--color-primary);
}

.hook0-topnav__logo-text {
  font-weight: 700;
  font-size: 1.125rem;
  letter-spacing: -0.025em;
  color: var(--color-text-primary);
}

@media (max-width: 640px) {
  .hook0-topnav__logo-text {
    display: none;
  }
}

/* Navigation Tabs */
.hook0-topnav__tabs {
  display: none;
  align-items: center;
  gap: 0.125rem;
  margin-left: 1.5rem;
}

@media (min-width: 768px) {
  .hook0-topnav__tabs {
    display: flex;
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

.hook0-topnav__tab {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  text-decoration: none;
  border-radius: var(--radius-md);
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
  white-space: nowrap;
}

.hook0-topnav__tab:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-tertiary);
}

.hook0-topnav__tab.active {
  color: var(--color-primary);
  background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
}

.hook0-topnav__tab-badge {
  padding: 0.0625rem 0.375rem;
  font-size: 0.6875rem;
  font-weight: 600;
  border-radius: var(--radius-full);
  background-color: var(--color-primary);
  color: var(--color-primary-text);
}

/* Spacer */
.hook0-topnav__spacer {
  flex: 1;
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

/* User avatar */
.hook0-topnav__user-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  cursor: pointer;
  padding: 0;
}

.hook0-topnav__user-avatar {
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-full);
  background: linear-gradient(
    135deg,
    var(--color-primary),
    color-mix(in srgb, var(--color-primary) 70%, var(--color-text-primary))
  );
  color: var(--color-primary-text, #fff);
  font-size: 0.75rem;
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

/* Dropdown shared styles */
.hook0-dropdown {
  position: relative;
}

.hook0-topnav__dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  left: 0;
  min-width: 14rem;
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: 0.375rem;
  z-index: var(--z-dropdown, 50);
}

.hook0-topnav__dropdown--right {
  left: auto;
  right: 0;
}

.hook0-topnav__dropdown--user {
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

.hook0-topnav__dropdown-item:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-topnav__dropdown-item--danger {
  color: var(--color-error);
}

.hook0-topnav__dropdown-item--danger:hover {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.hook0-topnav__dropdown-divider {
  height: 1px;
  background-color: var(--color-border);
  margin: 0.375rem 0;
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
</style>
