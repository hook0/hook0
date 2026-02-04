<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import {
  Webhook,
  Key,
  FolderTree,
  FileText,
  Link,
  FileCheck,
  Settings,
  ExternalLink,
  KeyRound,
  BookOpen,
  Sun,
  Moon,
  LogOut,
  User,
  PanelLeftClose,
  PanelLeft,
} from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { InstanceConfig, getInstanceConfig } from '@/utils/biscuit_auth';
import Hook0Button from '@/components/Hook0Button.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const route = useRoute();
const authStore = useAuthStore();
const contextStore = useContextStore();
const uiStore = useUiStore();

const instanceConfig = ref<InstanceConfig | null>(null);

onMounted(() => {
  getInstanceConfig()
    .then((config) => {
      instanceConfig.value = config;
    })
    .catch(console.error);
});

interface NavItem {
  name: string;
  icon: typeof Key;
  to?: { name: string; params?: Record<string, string> };
  href?: string;
  active: boolean;
}

const navItems = computed<NavItem[]>(() => {
  const orgId = contextStore.organizationId;
  const appId = contextStore.applicationId;
  const appSecretCompat = instanceConfig.value?.application_secret_compatibility ?? true;

  if (orgId && appId) {
    const params = { organization_id: orgId, application_id: appId };
    return [
      appSecretCompat
        ? {
            name: 'API Keys',
            icon: Key,
            to: { name: routes.ApplicationSecretsList, params },
            active: route.name === routes.ApplicationSecretsList,
          }
        : null,
      {
        name: 'Event Types',
        icon: FolderTree,
        to: { name: routes.EventTypesList, params },
        active: route.name === routes.EventTypesList || route.name === routes.EventTypesNew,
      },
      {
        name: 'Events',
        icon: FileText,
        to: { name: routes.EventsList, params },
        active: route.name === routes.EventsList || route.name === routes.EventsDetail,
      },
      {
        name: 'Subscriptions',
        icon: Link,
        to: { name: routes.SubscriptionsList, params },
        active:
          route.name === routes.SubscriptionsList ||
          route.name === routes.SubscriptionsNew ||
          route.name === routes.SubscriptionsDetail,
      },
      {
        name: 'Logs',
        icon: FileCheck,
        to: { name: routes.LogsList, params },
        active: route.name === routes.LogsList,
      },
      {
        name: 'Settings',
        icon: Settings,
        to: { name: routes.ApplicationsDashboard, params },
        active:
          route.name === routes.ApplicationsDashboard || route.name === routes.ApplicationsDetail,
      },
      {
        name: 'API Docs',
        icon: ExternalLink,
        href: 'https://documentation.hook0.com/',
        active: false,
      },
    ].filter(Boolean) as NavItem[];
  }

  if (orgId) {
    const params = { organization_id: orgId };
    return [
      {
        name: 'Applications',
        icon: FolderTree,
        to: { name: routes.ApplicationsList, params },
        active: route.name === routes.ApplicationsList,
      },
      {
        name: 'Service Tokens',
        icon: KeyRound,
        to: { name: routes.ServicesTokenList, params },
        active: route.name === routes.ServicesTokenList || route.name === routes.ServiceTokenView,
      },
      {
        name: 'API Docs',
        icon: BookOpen,
        href: 'https://documentation.hook0.com/',
        active: false,
      },
    ];
  }

  return [
    {
      name: 'API Docs',
      icon: BookOpen,
      href: 'https://documentation.hook0.com/',
      active: false,
    },
  ];
});

const sidebarCollapsed = ref(false);

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

function toggleColorMode() {
  uiStore.toggleColorMode();
}
</script>

<template>
  <aside
    class="hook0-sidebar"
    :class="{ collapsed: sidebarCollapsed }"
    aria-label="Sidebar navigation"
  >
    <!-- Logo -->
    <div class="hook0-sidebar-logo">
      <Hook0Button :to="{ name: routes.Home }" variant="ghost" class="hook0-sidebar-logo-link">
        <Webhook :size="24" aria-hidden="true" />
        <span v-if="!sidebarCollapsed" class="hook0-sidebar-logo-text">Hook0</span>
      </Hook0Button>
      <button class="hook0-sidebar-collapse" aria-label="Toggle sidebar" @click="toggleSidebar">
        <PanelLeftClose v-if="!sidebarCollapsed" :size="18" aria-hidden="true" />
        <PanelLeft v-else :size="18" aria-hidden="true" />
      </button>
    </div>

    <!-- Navigation -->
    <nav class="hook0-sidebar-nav">
      <ul class="hook0-sidebar-nav-list">
        <li v-for="item in navItems" :key="item.name">
          <a
            v-if="item.href"
            :href="item.href"
            target="_blank"
            rel="noopener noreferrer"
            class="hook0-sidebar-nav-item"
            :class="{ active: item.active }"
          >
            <component :is="item.icon" :size="20" aria-hidden="true" />
            <span v-if="!sidebarCollapsed" class="hook0-sidebar-nav-label">{{ item.name }}</span>
            <ExternalLink
              v-if="!sidebarCollapsed"
              :size="14"
              class="hook0-sidebar-external"
              aria-hidden="true"
            />
          </a>
          <router-link
            v-else-if="item.to"
            :to="item.to"
            class="hook0-sidebar-nav-item"
            :class="{ active: item.active }"
          >
            <component :is="item.icon" :size="20" aria-hidden="true" />
            <span v-if="!sidebarCollapsed" class="hook0-sidebar-nav-label">{{ item.name }}</span>
          </router-link>
        </li>
      </ul>
    </nav>

    <!-- Footer -->
    <div class="hook0-sidebar-footer">
      <button
        class="hook0-sidebar-footer-item"
        aria-label="Toggle dark mode"
        @click="toggleColorMode"
      >
        <Sun v-if="uiStore.effectiveColorMode === 'dark'" :size="18" aria-hidden="true" />
        <Moon v-else :size="18" aria-hidden="true" />
        <span v-if="!sidebarCollapsed" class="hook0-sidebar-nav-label">
          {{ uiStore.effectiveColorMode === 'dark' ? 'Light mode' : 'Dark mode' }}
        </span>
      </button>

      <router-link :to="{ name: routes.UserSettings }" class="hook0-sidebar-footer-item">
        <User :size="18" aria-hidden="true" />
        <span v-if="!sidebarCollapsed" class="hook0-sidebar-nav-label">
          {{ authStore.userInfo?.email ?? 'Settings' }}
        </span>
      </router-link>

      <button class="hook0-sidebar-footer-item" @click="authStore.logout()">
        <LogOut :size="18" aria-hidden="true" />
        <span v-if="!sidebarCollapsed" class="hook0-sidebar-nav-label">{{
          t('sidebar.logout')
        }}</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.hook0-sidebar {
  display: none;
  flex-direction: column;
  width: 16rem;
  background-color: var(--color-bg-primary);
  border-right: 1px solid var(--color-border);
  transition: width 0.2s ease;
  overflow: hidden;
}

@media (min-width: 768px) {
  .hook0-sidebar {
    display: flex;
  }
}

.hook0-sidebar.collapsed {
  width: 4rem;
}

.hook0-sidebar-logo {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  height: 4rem;
  flex-shrink: 0;
  border-bottom: 1px solid var(--color-border);
}

.hook0-sidebar-logo-link {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-primary);
  text-decoration: none;
  border: none;
  padding: 0;
}

.hook0-sidebar-logo-text {
  font-weight: 700;
  font-size: 1.125rem;
  letter-spacing: -0.025em;
  color: var(--color-text-primary);
}

.hook0-sidebar-collapse {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.hook0-sidebar-collapse:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
}

.hook0-sidebar-nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.hook0-sidebar-nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.15s ease;
  cursor: pointer;
}

.hook0-sidebar-nav-item:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-sidebar-nav-item.active {
  background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.hook0-sidebar-nav-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hook0-sidebar-external {
  margin-left: auto;
  opacity: 0.5;
}

.hook0-sidebar-footer {
  padding: 0.75rem;
  border-top: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.hook0-sidebar-footer-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: 0.875rem;
  font-weight: 500;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: all 0.15s ease;
  width: 100%;
  text-align: left;
}

.hook0-sidebar-footer-item:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}
</style>
