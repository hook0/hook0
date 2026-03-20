import { ref, computed, watch, nextTick, type Ref, type Component } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import {
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
  Plus,
  Send,
  Copy,
  Keyboard,
  BookOpen,
  Code2,
} from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { useI18n } from 'vue-i18n';

export type CommandItem = {
  id: string;
  label: string;
  icon: Component;
  category: string;
  action: () => void;
};

// Map of application-level features for preserving context when switching apps"
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

export function useCommandPalette(inputRef: Ref<HTMLInputElement | null>) {
  const { t } = useI18n();
  const router = useRouter();
  const route = useRoute();
  const authStore = useAuthStore();
  const contextStore = useContextStore();
  const uiStore = useUiStore();

  const { data: organizations } = useOrganizationList();
  const { data: applications } = useApplicationList(
    computed(() => contextStore.organizationId ?? '')
  );

  const query = ref('');
  const selectedIndex = ref(0);

  function navigate(to: { name: string; params?: Record<string, string> }) {
    void router.push(to);
    close();
  }

  function close() {
    uiStore.closeCommandPalette();
    query.value = '';
    selectedIndex.value = 0;
  }

  function switchToOrganization(orgId: string) {
    navigate({
      name: routes.OrganizationsDashboard,
      params: { organization_id: orgId },
    });
  }

  function switchToApplication(orgId: string, appId: string) {
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

  function buildRecentCommands(): CommandItem[] {
    const orgId = contextStore.organizationId;
    const appId = contextStore.applicationId;

    return uiStore.recentWorkspaces
      .slice(0, 3)
      .filter((ws) => ws.organizationId !== orgId || ws.applicationId !== appId)
      .map((workspace) => ({
        id: `recent-${workspace.organizationId}-${workspace.applicationId ?? 'org'}`,
        label: workspace.applicationName
          ? `${workspace.organizationName} > ${workspace.applicationName}`
          : workspace.organizationName,
        icon: History,
        category: t('commandPalette.recent'),
        action: () => {
          if (workspace.applicationId) {
            switchToApplication(workspace.organizationId, workspace.applicationId);
          } else {
            switchToOrganization(workspace.organizationId);
          }
        },
      }));
  }

  function buildOrgSwitchCommands(): CommandItem[] {
    const orgId = contextStore.organizationId;

    return (organizations.value ?? [])
      .filter((org) => org.organization_id !== orgId)
      .map((org) => ({
        id: `switch-org-${org.organization_id}`,
        label: org.name,
        icon: Building2,
        category: t('commandPalette.switchOrganization'),
        action: () => switchToOrganization(org.organization_id),
      }));
  }

  function buildAppSwitchCommands(): CommandItem[] {
    const orgId = contextStore.organizationId;
    const appId = contextStore.applicationId;

    if (!orgId) return [];

    return (applications.value ?? [])
      .filter((app) => app.application_id !== appId)
      .map((app) => ({
        id: `switch-app-${app.application_id}`,
        label: app.name,
        icon: Box,
        category: t('commandPalette.switchApplication'),
        action: () => switchToApplication(orgId, app.application_id),
      }));
  }

  function buildNavigationCommands(): CommandItem[] {
    const orgId = contextStore.organizationId;
    const appId = contextStore.applicationId;
    const items: CommandItem[] = [];

    items.push(
      {
        id: 'nav-home',
        label: t('commandPalette.goToHome'),
        icon: Home,
        category: t('commandPalette.navigation'),
        action: () => navigate({ name: routes.Home }),
      },
      {
        id: 'nav-documentation',
        label: t('commandPalette.goToDocumentation'),
        icon: BookOpen,
        category: t('commandPalette.navigation'),
        action: () => {
          window.open('https://documentation.hook0.com/', '_blank', 'noopener,noreferrer');
          close();
        },
      },
      {
        id: 'nav-api-reference',
        label: t('commandPalette.goToApiReference'),
        icon: Code2,
        category: t('commandPalette.navigation'),
        action: () => {
          window.open('https://documentation.hook0.com/api', '_blank', 'noopener,noreferrer');
          close();
        },
      }
    );

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

    return items;
  }

  function buildQuickActionCommands(): CommandItem[] {
    const orgId = contextStore.organizationId;
    const appId = contextStore.applicationId;

    if (!orgId || !appId) return [];

    const params = { organization_id: orgId, application_id: appId };
    return [
      {
        id: 'quick-create-subscription',
        label: t('commandPalette.createSubscription'),
        icon: Plus,
        category: t('commandPalette.quickActions'),
        action: () => navigate({ name: routes.SubscriptionsNew, params }),
      },
      {
        id: 'quick-create-event-type',
        label: t('commandPalette.createEventType'),
        icon: Plus,
        category: t('commandPalette.quickActions'),
        action: () => navigate({ name: routes.EventTypesNew, params }),
      },
      {
        id: 'quick-send-test-event',
        label: t('commandPalette.sendTestEvent'),
        icon: Send,
        category: t('commandPalette.quickActions'),
        action: () => navigate({ name: routes.EventsList, params }),
      },
      {
        id: 'quick-copy-api-key',
        label: t('commandPalette.copyApiKey'),
        icon: Copy,
        category: t('commandPalette.quickActions'),
        action: () => navigate({ name: routes.ApplicationSecretsList, params }),
      },
    ];
  }

  function buildGlobalCommands(): CommandItem[] {
    return [
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
        id: 'action-shortcuts',
        label: t('commandPalette.keyboardShortcuts'),
        icon: Keyboard,
        category: t('commandPalette.actions'),
        action: () => {
          close();
          uiStore.openShortcutsCheatSheet();
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
      },
    ];
  }

  const commands = computed<CommandItem[]>(() => [
    ...buildRecentCommands(),
    ...buildOrgSwitchCommands(),
    ...buildAppSwitchCommands(),
    ...buildNavigationCommands(),
    ...buildQuickActionCommands(),
    ...buildGlobalCommands(),
  ]);

  const filteredCommands = computed(() => {
    const q = query.value.toLowerCase().trim();
    if (!q) return commands.value;
    return commands.value.filter(
      (cmd) => cmd.label.toLowerCase().includes(q) || cmd.category.toLowerCase().includes(q)
    );
  });

  // Group filtered commands by category for sectioned rendering in the palette UI
  const groupedCommands = computed(() => {
    const groups: Record<string, CommandItem[]> = {};
    for (const cmd of filteredCommands.value) {
      (groups[cmd.category] ??= []).push(cmd);
    }
    return groups;
  });

  function scrollSelectedIntoView() {
    void nextTick(() => {
      const selected = document.querySelector('.hook0-command-palette-item.selected');
      selected?.scrollIntoView({ block: 'nearest' });
    });
  }

  function onKeydown(e: KeyboardEvent) {
    const handlers: Record<string, (e: KeyboardEvent) => void> = {
      ArrowDown: (e) => {
        e.preventDefault();
        selectedIndex.value = Math.min(selectedIndex.value + 1, filteredCommands.value.length - 1);
        scrollSelectedIntoView();
      },
      ArrowUp: (e) => {
        e.preventDefault();
        selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
        scrollSelectedIntoView();
      },
      Enter: (e) => {
        e.preventDefault();
        filteredCommands.value[selectedIndex.value]?.action();
      },
    };
    handlers[e.key]?.(e);
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

  return {
    query,
    selectedIndex,
    filteredCommands,
    groupedCommands,
    close,
    onKeydown,
  };
}
