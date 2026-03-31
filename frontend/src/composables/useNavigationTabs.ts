import { computed, type Component } from 'vue';
import { useRoute } from 'vue-router';
import {
  ArrowDownToLine,
  Send,
  Link,
  FolderTree,
  KeyRound,
  Settings,
  LayoutDashboard,
  Box,
  Key,
  Users,
  Repeat,
} from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { routes } from '@/routes';
import { useContextStore } from '@/stores/context';
import { useInstanceConfig } from '@/composables/useInstanceConfig';

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
  const { data: instanceConfig } = useInstanceConfig();

  const navTabs = computed<NavTab[]>(() => {
    const orgId = contextStore.organizationId;
    const appId = contextStore.applicationId;
    const appSecretCompat = instanceConfig.value?.application_secret_compatibility ?? true;

    // App-level navigation
    if (orgId && appId) {
      const params = { organization_id: orgId, application_id: appId };
      return [
        {
          id: 'dashboard',
          label: t('nav.dashboard'),
          icon: LayoutDashboard,
          to: { name: routes.ApplicationsDashboard, params },
          active: route.name === routes.ApplicationsDashboard,
        },
        {
          id: 'event-types',
          label: t('nav.eventTypes'),
          icon: FolderTree,
          to: { name: routes.EventTypesList, params },
          active: route.name === routes.EventTypesList || route.name === routes.EventTypesNew,
        },
        {
          id: 'subscriptions',
          label: t('nav.webhooks'),
          icon: Link,
          to: { name: routes.SubscriptionsList, params },
          active:
            route.name === routes.SubscriptionsList ||
            route.name === routes.SubscriptionsNew ||
            route.name === routes.SubscriptionsDetail ||
            route.name === routes.SubscriptionsEdit,
        },
        {
          id: 'events',
          label: t('nav.inboundEvents'),
          icon: ArrowDownToLine,
          to: { name: routes.EventsList, params },
          active: route.name === routes.EventsList || route.name === routes.EventsDetail,
        },
        {
          id: 'logs',
          label: t('nav.deliveries'),
          icon: Send,
          to: { name: routes.LogsList, params },
          active: route.name === routes.LogsList,
        },
        ...(appSecretCompat
          ? [
              {
                id: 'application-secrets',
                label: t('nav.applicationSecrets'),
                icon: KeyRound,
                to: { name: routes.ApplicationSecretsList, params },
                active: route.name === routes.ApplicationSecretsList,
              },
            ]
          : []),
        {
          id: 'settings',
          label: t('nav.settings'),
          icon: Settings,
          to: { name: routes.ApplicationsDetail, params },
          active: route.name === routes.ApplicationsDetail,
        },
      ];
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
          id: 'retry-schedules',
          label: t('nav.retrySchedules'),
          icon: Repeat,
          to: { name: routes.RetrySchedulesList, params },
          active:
            route.name === routes.RetrySchedulesList ||
            route.name === routes.RetrySchedulesNew ||
            route.name === routes.RetrySchedulesEdit,
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
