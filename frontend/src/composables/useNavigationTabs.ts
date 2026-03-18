import { computed, ref, onMounted, type Component } from 'vue';
import { useRoute } from 'vue-router';
import {
  FileText,
  Link,
  FolderTree,
  ScrollText,
  KeyRound,
  Settings,
  LayoutDashboard,
  Box,
  Key,
  Users,
} from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { routes } from '@/routes';
import { useContextStore } from '@/stores/context';
import { type InstanceConfig, getInstanceConfig } from '@/utils/biscuit_auth';

export type NavTab = {
  id: string;
  label: string;
  icon: Component;
  to: { name: string; params?: Record<string, string> };
  active: boolean;
  badge?: string;
};

export function useNavigationTabs() {
  const { t } = useI18n();
  const route = useRoute();
  const contextStore = useContextStore();
  const instanceConfig = ref<InstanceConfig | null>(null);

  onMounted(() => {
    getInstanceConfig()
      .then((config) => {
        instanceConfig.value = config;
      })
      .catch((err) => {
        // If instance config fetch fails (network, 500), default to null.
        // appSecretCompat will fall back to true (show API Keys tab by default).
        console.warn('[useNavigationTabs] Failed to load instance config:', err);
        instanceConfig.value = null;
      });
  });

  const navTabs = computed<NavTab[]>(() => {
    const orgId = contextStore.organizationId;
    const appId = contextStore.applicationId;
    const appSecretCompat = instanceConfig.value?.application_secret_compatibility ?? true;

    // App-level navigation
    if (orgId && appId) {
      const params = { organization_id: orgId, application_id: appId };
      const tabs: NavTab[] = [
        {
          id: 'dashboard',
          label: t('nav.dashboard'),
          icon: LayoutDashboard,
          to: { name: routes.ApplicationsDashboard, params },
          active: route.name === routes.ApplicationsDashboard,
        },
        {
          id: 'events',
          label: t('nav.events'),
          icon: FileText,
          to: { name: routes.EventsList, params },
          active: route.name === routes.EventsList || route.name === routes.EventsDetail,
        },
        {
          id: 'subscriptions',
          label: t('nav.subscriptions'),
          icon: Link,
          to: { name: routes.SubscriptionsList, params },
          active:
            route.name === routes.SubscriptionsList ||
            route.name === routes.SubscriptionsNew ||
            route.name === routes.SubscriptionsDetail,
        },
        {
          id: 'event-types',
          label: t('nav.eventTypes'),
          icon: FolderTree,
          to: { name: routes.EventTypesList, params },
          active: route.name === routes.EventTypesList || route.name === routes.EventTypesNew,
        },
        {
          id: 'logs',
          label: t('nav.logs'),
          icon: ScrollText,
          to: { name: routes.LogsList, params },
          active: route.name === routes.LogsList,
        },
      ];

      if (appSecretCompat) {
        tabs.push({
          id: 'api-keys',
          label: t('nav.apiKeys'),
          icon: KeyRound,
          to: { name: routes.ApplicationSecretsList, params },
          active: route.name === routes.ApplicationSecretsList,
        });
      }

      tabs.push({
        id: 'settings',
        label: t('nav.settings'),
        icon: Settings,
        to: { name: routes.ApplicationsDetail, params },
        active: route.name === routes.ApplicationsDetail,
      });

      return tabs;
    }

    // Org-level navigation
    if (orgId) {
      const params = { organization_id: orgId };
      return [
        {
          id: 'dashboard',
          label: t('nav.dashboard'),
          icon: LayoutDashboard,
          to: { name: routes.OrganizationsDashboard, params },
          active: route.name === routes.OrganizationsDashboard,
        },
        {
          id: 'applications',
          label: t('nav.applications'),
          icon: Box,
          to: { name: routes.ApplicationsList, params },
          active: route.name === routes.ApplicationsList,
        },
        {
          id: 'service-tokens',
          label: t('nav.serviceTokens'),
          icon: Key,
          to: { name: routes.ServicesTokenList, params },
          active: route.name === routes.ServicesTokenList || route.name === routes.ServiceTokenView,
        },
        {
          id: 'team',
          label: t('nav.team'),
          icon: Users,
          to: { name: routes.OrganizationsTeam, params },
          active: route.name === routes.OrganizationsTeam,
        },
        {
          id: 'org-settings',
          label: t('nav.settings'),
          icon: Settings,
          to: { name: routes.OrganizationsDetail, params },
          active: route.name === routes.OrganizationsDetail,
        },
      ];
    }

    return [];
  });

  return { navTabs };
}
