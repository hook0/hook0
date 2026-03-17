<script setup lang="ts">
import { computed } from 'vue';
import { Home, Menu } from 'lucide-vue-next';
import { routes } from '@/routes';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';
import { useNavigationTabs } from '@/composables/useNavigationTabs';
import { useRoute } from 'vue-router';

const route = useRoute();
const uiStore = useUiStore();
const { t } = useI18n();

const { navTabs } = useNavigationTabs();

// Mobile tab bar shows max 4 tabs from the composable.
// When there are more (app-level), show first 3 + "More" button.
// When no context, show a simple Home tab.
const MAX_VISIBLE_TABS = 4;

const tabs = computed(() => {
  if (navTabs.value.length === 0) {
    return [
      {
        id: 'home',
        label: t('nav.home'),
        icon: Home,
        to: { name: routes.Home },
        active: route.name === routes.Home,
      },
    ];
  }

  // If tabs fit, show them all; otherwise truncate to leave room for "More"
  if (navTabs.value.length <= MAX_VISIBLE_TABS) {
    return navTabs.value;
  }
  return navTabs.value.slice(0, MAX_VISIBLE_TABS - 1);
});

const showMoreButton = computed(() => navTabs.value.length > MAX_VISIBLE_TABS);
</script>

<template>
  <nav
    class="hook0-mobile-tab-bar"
    data-test="mobile-tab-bar"
    :aria-label="t('common.mobileNavigation')"
  >
    <div class="hook0-mobile-tab-list">
      <router-link
        v-for="tab in tabs"
        :key="tab.id"
        :to="tab.to"
        class="hook0-mobile-tab"
        :data-test="'mobile-tab-' + tab.id"
        :class="{ active: tab.active }"
        :aria-current="tab.active ? 'page' : undefined"
      >
        <component :is="tab.icon" :size="20" aria-hidden="true" />
        <span class="hook0-mobile-tab-label">{{ tab.label }}</span>
      </router-link>
      <button
        v-if="showMoreButton"
        class="hook0-mobile-tab"
        data-test="mobile-tab-more"
        @click="uiStore.toggleMobileDrawer()"
      >
        <Menu :size="20" aria-hidden="true" />
        <span class="hook0-mobile-tab-label">{{ t('nav.more') }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.hook0-mobile-tab-bar {
  background-color: var(--color-bg-primary);
  border-top: 1px solid var(--color-border);
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: var(--z-tabbar, 40);
}

.hook0-mobile-tab-list {
  display: flex;
  align-items: stretch;
  padding: 0.25rem 0;
}

@media (min-width: 768px) {
  .hook0-mobile-tab-bar {
    display: none;
  }
}

.hook0-mobile-tab {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.125rem;
  padding: 0.5rem 0.25rem;
  color: var(--color-text-muted);
  text-decoration: none;
  font-size: 0.625rem;
  font-weight: 500;
  transition: color 0.15s ease;
  border: none;
  background: none;
  cursor: pointer;
}

.hook0-mobile-tab :deep(svg) {
  flex-shrink: 0;
}

.hook0-mobile-tab:hover {
  color: var(--color-text-secondary);
}

.hook0-mobile-tab.active {
  color: var(--color-primary);
}

.hook0-mobile-tab-label {
  text-align: center;
  line-height: 1;
}
</style>
