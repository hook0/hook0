<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { Home, FolderTree, FileText, Link, Settings, Menu } from 'lucide-vue-next';
import { routes } from '@/routes';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';

const route = useRoute();
const contextStore = useContextStore();
const uiStore = useUiStore();
const { t } = useI18n();

interface TabItem {
  name: string;
  icon: typeof Home;
  to: { name: string; params?: Record<string, string> };
  active: boolean;
}

const tabs = computed<TabItem[]>(() => {
  const orgId = contextStore.organizationId;
  const appId = contextStore.applicationId;

  if (orgId && appId) {
    const params = { organization_id: orgId, application_id: appId };
    return [
      {
        name: 'Events',
        icon: FileText,
        to: { name: routes.EventsList, params },
        active: route.name === routes.EventsList || route.name === routes.EventsDetail,
      },
      {
        name: 'Types',
        icon: FolderTree,
        to: { name: routes.EventTypesList, params },
        active: route.name === routes.EventTypesList,
      },
      {
        name: 'Webhooks',
        icon: Link,
        to: { name: routes.SubscriptionsList, params },
        active:
          route.name === routes.SubscriptionsList ||
          route.name === routes.SubscriptionsNew ||
          route.name === routes.SubscriptionsDetail,
      },
      {
        name: 'Settings',
        icon: Settings,
        to: { name: routes.ApplicationsDashboard, params },
        active: route.name === routes.ApplicationsDashboard,
      },
    ];
  }

  if (orgId) {
    const params = { organization_id: orgId };
    return [
      {
        name: 'Home',
        icon: Home,
        to: { name: routes.OrganizationsDashboard, params },
        active: route.name === routes.OrganizationsDashboard,
      },
      {
        name: 'Apps',
        icon: FolderTree,
        to: { name: routes.ApplicationsList, params },
        active: route.name === routes.ApplicationsList,
      },
      {
        name: 'Settings',
        icon: Settings,
        to: { name: routes.OrganizationsDetail, params },
        active: route.name === routes.OrganizationsDetail,
      },
    ];
  }

  return [
    {
      name: 'Home',
      icon: Home,
      to: { name: routes.Home },
      active: route.name === routes.Home,
    },
  ];
});
</script>

<template>
  <nav class="hook0-mobile-tab-bar" aria-label="Mobile navigation">
    <div class="hook0-mobile-tab-list">
      <router-link
        v-for="tab in tabs"
        :key="tab.name"
        :to="tab.to"
        class="hook0-mobile-tab"
        :class="{ active: tab.active }"
        :aria-current="tab.active ? 'page' : undefined"
      >
        <component :is="tab.icon" :size="20" aria-hidden="true" />
        <span class="hook0-mobile-tab-label">{{ tab.name }}</span>
      </router-link>
      <button
        v-if="contextStore.organizationId && contextStore.applicationId"
        class="hook0-mobile-tab"
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
  z-index: 40;
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
