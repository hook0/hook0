<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import {
  Search,
  FolderTree,
  FileText,
  Link,
  Settings,
  Key,
  LogOut,
  User,
  Home,
  Moon,
  Sun,
  FileCheck,
  KeyRound,
  Building2,
  Box,
  History,
} from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const contextStore = useContextStore();
const uiStore = useUiStore();

// Fetch organizations and applications for switching
const { data: organizations } = useOrganizationList();
const { data: applications } = useApplicationList(
  computed(() => contextStore.organizationId ?? '')
);

const inputRef = ref<HTMLInputElement | null>(null);
const query = ref('');
const selectedIndex = ref(0);

interface CommandItem {
  id: string;
  label: string;
  icon: typeof Home;
  category: string;
  action: () => void;
}

// Map of application-level features for preserving context when switching apps
const APP_LEVEL_FEATURES: Record<string, string> = {
  [routes.EventsList]: routes.EventsList,
  [routes.EventsDetail]: routes.EventsList,
  [routes.EventTypesList]: routes.EventTypesList,
  [routes.EventTypesNew]: routes.EventTypesList,
  [routes.SubscriptionsList]: routes.SubscriptionsList,
  [routes.SubscriptionsNew]: routes.SubscriptionsList,
  [routes.SubscriptionsDetail]: routes.SubscriptionsList,
  [routes.LogsList]: routes.LogsList,
  [routes.ApplicationSecretsList]: routes.ApplicationSecretsList,
  [routes.ApplicationSecretsDetail]: routes.ApplicationSecretsList,
  [routes.ApplicationSecretsNew]: routes.ApplicationSecretsList,
  [routes.ApplicationsDashboard]: routes.ApplicationsDashboard,
  [routes.ApplicationsDetail]: routes.ApplicationsDetail,
};

function switchToOrganization(orgId: string) {
  navigate({
    name: routes.OrganizationsDashboard,
    params: { organization_id: orgId },
  });
}

function switchToApplication(orgId: string, appId: string) {
  // Try to preserve the current feature when switching apps
  const currentRouteName = route.name as string;
  const preservedFeature = APP_LEVEL_FEATURES[currentRouteName];

  if (preservedFeature) {
    navigate({
      name: preservedFeature,
      params: { organization_id: orgId, application_id: appId },
    });
  } else {
    navigate({
      name: routes.ApplicationsDashboard,
      params: { organization_id: orgId, application_id: appId },
    });
  }
}

const commands = computed<CommandItem[]>(() => {
  const orgId = contextStore.organizationId;
  const appId = contextStore.applicationId;
  const items: CommandItem[] = [];

  // Recent workspaces (org/app combinations)
  const recentWorkspaces = uiStore.recentWorkspaces.slice(0, 3);
  if (recentWorkspaces.length > 0) {
    for (const workspace of recentWorkspaces) {
      // Skip current workspace
      if (workspace.organizationId === orgId && workspace.applicationId === appId) {
        continue;
      }

      const label = workspace.applicationName
        ? `${workspace.organizationName} > ${workspace.applicationName}`
        : workspace.organizationName;

      items.push({
        id: `recent-${workspace.organizationId}-${workspace.applicationId ?? 'org'}`,
        label,
        icon: History,
        category: t('commandPalette.recent'),
        action: () => {
          if (workspace.applicationId) {
            switchToApplication(workspace.organizationId, workspace.applicationId);
          } else {
            switchToOrganization(workspace.organizationId);
          }
        },
      });
    }
  }

  // Organization switching
  if (organizations.value && organizations.value.length > 0) {
    for (const org of organizations.value) {
      // Skip current org
      if (org.organization_id === orgId) continue;

      items.push({
        id: `switch-org-${org.organization_id}`,
        label: org.name,
        icon: Building2,
        category: t('commandPalette.switchOrganization'),
        action: () => switchToOrganization(org.organization_id),
      });
    }
  }

  // Application switching (within current org)
  if (orgId && applications.value && applications.value.length > 0) {
    for (const app of applications.value) {
      // Skip current app
      if (app.application_id === appId) continue;

      items.push({
        id: `switch-app-${app.application_id}`,
        label: app.name,
        icon: Box,
        category: t('commandPalette.switchApplication'),
        action: () => switchToApplication(orgId, app.application_id),
      });
    }
  }

  // Navigation
  items.push({
    id: 'nav-home',
    label: t('commandPalette.goToHome'),
    icon: Home,
    category: t('commandPalette.navigation'),
    action: () => navigate({ name: routes.Home }),
  });

  if (orgId) {
    const orgParams = { organization_id: orgId };
    items.push(
      {
        id: 'nav-apps',
        label: t('commandPalette.goToApplications'),
        icon: FolderTree,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.ApplicationsList, params: orgParams }),
      },
      {
        id: 'nav-service-tokens',
        label: t('commandPalette.goToServiceTokens'),
        icon: KeyRound,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.ServicesTokenList, params: orgParams }),
      },
      {
        id: 'nav-org-settings',
        label: t('commandPalette.goToOrgSettings'),
        icon: Settings,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.OrganizationsDetail, params: orgParams }),
      }
    );
  }

  if (orgId && appId) {
    const params = { organization_id: orgId, application_id: appId };
    items.push(
      {
        id: 'nav-event-types',
        label: t('commandPalette.goToEventTypes'),
        icon: FolderTree,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.EventTypesList, params }),
      },
      {
        id: 'nav-events',
        label: t('commandPalette.goToEvents'),
        icon: FileText,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.EventsList, params }),
      },
      {
        id: 'nav-subscriptions',
        label: t('commandPalette.goToSubscriptions'),
        icon: Link,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.SubscriptionsList, params }),
      },
      {
        id: 'nav-logs',
        label: t('commandPalette.goToLogs'),
        icon: FileCheck,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.LogsList, params }),
      },
      {
        id: 'nav-api-keys',
        label: t('commandPalette.goToApiKeys'),
        icon: Key,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.ApplicationSecretsList, params }),
      },
      {
        id: 'nav-app-settings',
        label: t('commandPalette.goToAppSettings'),
        icon: Settings,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.ApplicationsDashboard, params }),
      }
    );
  }

  // Actions
  items.push(
    {
      id: 'action-settings',
      label: t('commandPalette.userSettings'),
      icon: User,
      category: t('commandPalette.actions'),
      action: () => navigate({ name: routes.UserSettings }),
    },
    {
      id: 'action-toggle-theme',
      label:
        uiStore.effectiveColorMode === 'dark'
          ? t('commandPalette.switchToLight')
          : t('commandPalette.switchToDark'),
      icon: uiStore.effectiveColorMode === 'dark' ? Sun : Moon,
      category: t('commandPalette.actions'),
      action: () => {
        uiStore.toggleColorMode();
        close();
      },
    },
    {
      id: 'action-logout',
      label: t('commandPalette.logout'),
      icon: LogOut,
      category: t('commandPalette.actions'),
      action: () => {
        void authStore.logout();
        close();
      },
    }
  );

  return items;
});

const filteredCommands = computed(() => {
  const q = query.value.toLowerCase().trim();
  if (!q) return commands.value;
  return commands.value.filter(
    (cmd) => cmd.label.toLowerCase().includes(q) || cmd.category.toLowerCase().includes(q)
  );
});

const groupedCommands = computed(() => {
  const groups: Record<string, CommandItem[]> = {};
  for (const cmd of filteredCommands.value) {
    if (!groups[cmd.category]) groups[cmd.category] = [];
    groups[cmd.category].push(cmd);
  }
  return groups;
});

function navigate(to: { name: string; params?: Record<string, string> }) {
  router.push(to).catch(console.error);
  close();
}

function close() {
  uiStore.closeCommandPalette();
  query.value = '';
  selectedIndex.value = 0;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredCommands.value.length - 1);
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (e.key === 'Enter') {
    e.preventDefault();
    const cmd = filteredCommands.value[selectedIndex.value];
    if (cmd) cmd.action();
  }
}

watch(
  () => uiStore.commandPaletteOpen,
  (isOpen) => {
    if (isOpen) {
      void nextTick(() => {
        inputRef.value?.focus();
      });
    }
  }
);

watch(query, () => {
  selectedIndex.value = 0;
});
</script>

<template>
  <Teleport to="body">
    <Transition name="command-palette">
      <div
        v-if="uiStore.commandPaletteOpen"
        class="hook0-command-palette-overlay"
        @click.self="close"
      >
        <div
          class="hook0-command-palette"
          role="dialog"
          aria-label="Command palette"
          @keydown="onKeydown"
        >
          <div class="hook0-command-palette-input-wrapper">
            <Search :size="20" class="hook0-command-palette-search-icon" aria-hidden="true" />
            <input
              ref="inputRef"
              v-model="query"
              class="hook0-command-palette-input"
              :placeholder="t('commandPalette.placeholder')"
              type="text"
              role="combobox"
              aria-expanded="true"
              aria-controls="command-list"
              aria-autocomplete="list"
            />
          </div>

          <div id="command-list" class="hook0-command-palette-list" role="listbox">
            <div v-if="filteredCommands.length === 0" class="hook0-command-palette-empty">
              {{ t('commandPalette.noResults') }}
            </div>

            <div v-for="(items, category) in groupedCommands" :key="category">
              <div class="hook0-command-palette-group-label">{{ category }}</div>
              <button
                v-for="item in items"
                :key="item.id"
                class="hook0-command-palette-item"
                :class="{ selected: filteredCommands.indexOf(item) === selectedIndex }"
                role="option"
                :aria-selected="filteredCommands.indexOf(item) === selectedIndex"
                @click="item.action()"
                @mouseenter="selectedIndex = filteredCommands.indexOf(item)"
              >
                <component :is="item.icon" :size="18" aria-hidden="true" />
                <span>{{ item.label }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.hook0-command-palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 20vh;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.hook0-command-palette {
  width: 100%;
  max-width: 36rem;
  margin: 0 1rem;
  background-color: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.hook0-command-palette-input-wrapper {
  display: flex;
  align-items: center;
  padding: 0 1rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-command-palette-search-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.hook0-command-palette-input {
  flex: 1;
  padding: 0.875rem 0.75rem;
  font-size: 1rem;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  outline: none;
}

.hook0-command-palette-input::placeholder {
  color: var(--color-text-muted);
}

.hook0-command-palette-list {
  max-height: 20rem;
  overflow-y: auto;
  padding: 0.5rem;
}

.hook0-command-palette-empty {
  padding: 2rem 1rem;
  text-align: center;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.hook0-command-palette-group-label {
  padding: 0.5rem 0.75rem 0.25rem;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
}

.hook0-command-palette-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  padding: 0.625rem 0.75rem;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.1s ease;
  text-align: left;
}

.hook0-command-palette-item:hover,
.hook0-command-palette-item.selected {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}
</style>
