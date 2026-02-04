<script setup lang="ts">
import { RouteLocationRaw, RouteParams, useRoute, useRouter } from 'vue-router';
import { onMounted, onUnmounted, onUpdated, ref, watch } from 'vue';
import {
  Box,
  Plus,
  ChevronRight,
  Package,
  Layers,
  ChevronDown,
  FolderKanban,
} from 'lucide-vue-next';

import * as OrganizationService from './organizations/OrganizationService';
import * as ApplicationService from './organizations/applications/ApplicationService';
import { UUID } from '@/http';
import { Organization } from './organizations/OrganizationService';
import { Application } from './organizations/applications/ApplicationService';
import { routes } from '@/routes';
import Hook0DropdownOptions from '@/components/Hook0DropdownOptions';
import Hook0DropdownMenuItemLink from '@/components/Hook0DropdownMenuItemLink.vue';
import Hook0Dropdown from '@/components/Hook0Dropdown.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Avatar from '@/components/Hook0Avatar.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import { useAuthStore } from '@/stores/auth';
import { isPricingEnabled } from '@/instance';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type ApplicationsPerOrganization = {
  organization: Organization;
  applications: Array<Application>;
};

const router = useRouter();
const route = useRoute();

const pricingEnabled = ref<boolean>(false);

const applicationsPerOrganization = ref<null | ApplicationsPerOrganization[]>(null);
const organization_name = ref('');
const application_name = ref('');
const removeRouterGuard = ref<null | (() => void)>(null);

const props = defineProps<{
  displayAsCards: boolean;
}>();

watch(
  () => useAuthStore().accessToken,
  (newToken, oldToken) => {
    if (newToken !== oldToken) {
      if (newToken) {
        void getApplicationsPerOrganization().then((result) => {
          applicationsPerOrganization.value = result;
        });
      } else {
        applicationsPerOrganization.value = null;
      }
    }
  },
  { deep: true }
);

function getApplicationsPerOrganization(): Promise<ApplicationsPerOrganization[]> {
  return OrganizationService.list().then((organizations) =>
    Promise.all(
      organizations.map((organization) => ApplicationService.list(organization.organization_id))
    ).then((applications) => {
      return applications.reduce(
        (m, applications) => {
          return applications.reduce((m, application) => {
            const organization = organizations.find(
              (org) => org.organization_id === application.organization_id
            );

            if (!organization) {
              console.error(
                'should never happen, application is linkedin to unknown organization. Silent fail'
              );
              return m;
            }

            let organization_in_group = m.find(
              (item) => item.organization.organization_id === application.organization_id
            );

            if (!organization_in_group) {
              console.error(
                'should never happen, application is linkedin to unknown organization. Silent fail'
              );
              return m;
            }

            organization_in_group.applications.push(application);

            return m;
          }, m);
        },
        organizations.map((organization) => {
          return {
            organization: organization,
            applications: [],
          };
        }) as ApplicationsPerOrganization[]
      );
    })
  );
}

function goto(parent: Hook0DropdownOptions, route: RouteLocationRaw) {
  parent.close();
  return router.push(route);
}

function _updateDropdown(params: RouteParams) {
  if (applicationsPerOrganization.value !== null) {
    const groups = applicationsPerOrganization.value;
    const orgId = params.organization_id as UUID | undefined;

    if (orgId) {
      const matchedGroup = groups.find((group) => group.organization.organization_id === orgId);
      organization_name.value = matchedGroup ? matchedGroup.organization.name : '';
    } else {
      organization_name.value = '';
    }

    const matchedApp = groups
      .flatMap((group) => group.applications)
      .find((application) => application.application_id === params.application_id);

    application_name.value = matchedApp ? matchedApp.name : 'Select an application';
  }
}

// Map of application-level features for preserving context when switching apps
const APP_LEVEL_FEATURES: Record<string, string> = {
  [routes.EventsList]: routes.EventsList,
  [routes.EventsDetail]: routes.EventsList, // Falls back to list
  [routes.EventTypesList]: routes.EventTypesList,
  [routes.EventTypesNew]: routes.EventTypesList, // Falls back to list
  [routes.SubscriptionsList]: routes.SubscriptionsList,
  [routes.SubscriptionsNew]: routes.SubscriptionsList, // Falls back to list
  [routes.SubscriptionsDetail]: routes.SubscriptionsList, // Falls back to list
  [routes.LogsList]: routes.LogsList,
  [routes.ApplicationSecretsList]: routes.ApplicationSecretsList,
  [routes.ApplicationSecretsDetail]: routes.ApplicationSecretsList, // Falls back to list
  [routes.ApplicationSecretsNew]: routes.ApplicationSecretsList, // Falls back to list
  [routes.ApplicationsDashboard]: routes.ApplicationsDashboard,
  [routes.ApplicationsDetail]: routes.ApplicationsDetail,
};

function navigateToOrg(orgId: UUID) {
  void router.push({
    name: routes.OrganizationsDashboard,
    params: { organization_id: orgId },
  });
}

function navigateToApp(orgId: UUID, appId: UUID) {
  // Try to preserve the current feature when switching apps
  const currentRouteName = route.name as string;
  const preservedFeature = APP_LEVEL_FEATURES[currentRouteName];

  if (preservedFeature) {
    // Navigate to the same feature in the new app
    void router.push({
      name: preservedFeature,
      params: {
        organization_id: orgId,
        application_id: appId,
      },
    });
  } else {
    // Fallback to app dashboard
    void router.push({
      name: routes.ApplicationsDashboard,
      params: {
        organization_id: orgId,
        application_id: appId,
      },
    });
  }
}

function navigateToNewApp(orgId: UUID) {
  void router.push({
    name: routes.ApplicationsNew,
    params: { organization_id: orgId },
  });
}

onMounted(() => {
  if (useAuthStore().accessToken) {
    void getApplicationsPerOrganization().then((result) => {
      applicationsPerOrganization.value = result;
    });
  }

  removeRouterGuard.value = router.afterEach(() => {
    return _updateDropdown(route.params);
  });

  void isPricingEnabled().then((enabled) => {
    pricingEnabled.value = enabled;
  });

  return _updateDropdown(route.params);
});

onUpdated(() => {
  return _updateDropdown(route.params);
});

onUnmounted(() => {
  if (removeRouterGuard.value !== null) {
    removeRouterGuard.value();
  }
});

const avatarGradients = [
  'linear-gradient(135deg, #4f46e5, #7c3aed)',
  'linear-gradient(135deg, #059669, #10b981)',
  'linear-gradient(135deg, #d97706, #f59e0b)',
  'linear-gradient(135deg, #dc2626, #ef4444)',
  'linear-gradient(135deg, #2563eb, #3b82f6)',
  'linear-gradient(135deg, #7c3aed, #a855f7)',
  'linear-gradient(135deg, #0891b2, #06b6d4)',
  'linear-gradient(135deg, #c2410c, #ea580c)',
];

function getGradient(index: number): string {
  return avatarGradients[index % avatarGradients.length];
}
</script>

<template>
  <!-- ==================== CARD VIEW ==================== -->
  <template v-if="applicationsPerOrganization !== null && props.displayAsCards">
    <Hook0Stack layout="grid" gap="lg" grid-size="wide">
      <!-- Organization Cards -->
      <Hook0Card
        v-for="(organizationGroup, index) in applicationsPerOrganization"
        :key="organizationGroup.organization.organization_id"
        variant="interactive"
      >
        <!-- Card Header -->
        <Hook0CardContent>
          <Hook0Stack
            direction="row"
            align="center"
            gap="md"
            role="button"
            tabindex="0"
            class="org-card__header"
            @click="navigateToOrg(organizationGroup.organization.organization_id)"
            @keydown.enter="navigateToOrg(organizationGroup.organization.organization_id)"
          >
            <Hook0Avatar
              :name="organizationGroup.organization.name"
              size="lg"
              variant="square"
              :gradient="getGradient(index)"
            />
            <Hook0Stack direction="column" gap="none" style="flex: 1; min-width: 0">
              <Hook0Stack direction="row" align="center" gap="sm" wrap>
                <Hook0Text variant="primary" size="md" weight="semibold">
                  {{ organizationGroup.organization.name }}
                </Hook0Text>
                <Hook0Badge
                  v-if="pricingEnabled && organizationGroup.organization.plan"
                  variant="primary"
                  size="sm"
                >
                  {{ organizationGroup.organization.plan?.label || '' }}
                </Hook0Badge>
                <Hook0Badge v-else-if="pricingEnabled" variant="default" size="sm">
                  {{ t('orgAppSelector.developer') }}
                </Hook0Badge>
              </Hook0Stack>
              <Hook0Stack direction="row" align="center" gap="xs" inline>
                <Layers :size="13" aria-hidden="true" />
                <Hook0Text variant="muted" size="sm">
                  {{ t('orgAppSelector.appsCount', organizationGroup.applications.length) }}
                </Hook0Text>
              </Hook0Stack>
            </Hook0Stack>
            <ChevronRight :size="16" aria-hidden="true" />
          </Hook0Stack>
        </Hook0CardContent>

        <!-- Applications List -->
        <Hook0CardContent>
          <Hook0Stack direction="column" gap="xs">
            <template v-if="organizationGroup.applications.length > 0">
              <Hook0ListItem
                v-for="application in organizationGroup.applications"
                :key="application.application_id"
                show-chevron
                @click="
                  navigateToApp(
                    organizationGroup.organization.organization_id,
                    application.application_id
                  )
                "
              >
                <template #icon>
                  <Box :size="16" aria-hidden="true" />
                </template>
                <template #left>
                  <Hook0Text variant="primary" size="sm" weight="medium">
                    {{ application.name }}
                  </Hook0Text>
                </template>
              </Hook0ListItem>
            </template>

            <!-- Empty State -->
            <Hook0EmptyState
              v-else
              :title="t('orgAppSelector.noApplicationFound')"
              :description="t('orgAppSelector.noApplicationDescription')"
            >
              <template #icon>
                <Package :size="24" aria-hidden="true" />
              </template>
            </Hook0EmptyState>

            <!-- Create App Button -->
            <Hook0ListItem
              variant="action"
              separated
              @click="navigateToNewApp(organizationGroup.organization.organization_id)"
            >
              <template #icon>
                <Plus :size="14" aria-hidden="true" />
              </template>
              <template #left>
                {{ t('orgAppSelector.createNewApplication') }}
              </template>
            </Hook0ListItem>
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>

      <!-- New Organization Card -->
      <Hook0Card
        variant="dashed"
        as="button"
        @click="void router.push({ name: routes.OrganizationsNew })"
      >
        <Hook0CardContent>
          <Hook0Stack direction="column" align="center" justify="center" gap="sm">
            <Hook0Avatar name="+" size="xl" variant="rounded" />
            <Hook0Text variant="primary" size="md" weight="semibold">
              {{ t('orgAppSelector.newOrganization') }}
            </Hook0Text>
            <Hook0Text variant="muted" size="sm">
              {{ t('orgAppSelector.newOrganizationDescription') }}
            </Hook0Text>
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>
    </Hook0Stack>
  </template>

  <!-- ==================== DROPDOWN VIEW ==================== -->
  <Hook0Dropdown
    v-else-if="applicationsPerOrganization !== null && !props.displayAsCards"
    justify="left"
  >
    <template #menu="parent">
      <Hook0Button @click="parent.toggle">
        <template #default>
          <Hook0Stack direction="column" gap="none" align="start">
            <Hook0Text variant="muted" size="sm">{{ organization_name }}</Hook0Text>
            <Hook0Text variant="primary" size="md" weight="medium">{{
              application_name
            }}</Hook0Text>
          </Hook0Stack>
        </template>
        <template #right>
          <ChevronDown :size="16" aria-hidden="true" />
        </template>
      </Hook0Button>
    </template>

    <template #dropdown="parent">
      <Hook0Stack direction="column" gap="none">
        <Hook0Stack
          v-for="(organizationGroup, index) in applicationsPerOrganization"
          :key="index"
          direction="column"
          gap="none"
        >
          <Hook0DropdownMenuItemLink
            @click="
              goto(parent, {
                name: routes.OrganizationsDashboard,
                params: { organization_id: organizationGroup.organization.organization_id },
              })
            "
          >
            <Hook0Stack direction="row" align="center" gap="sm">
              <FolderKanban :size="16" aria-hidden="true" />
              <Hook0Text variant="primary" size="md">
                {{ organizationGroup.organization.name }}
              </Hook0Text>
            </Hook0Stack>
          </Hook0DropdownMenuItemLink>

          <Hook0Stack direction="column" gap="none" style="padding-left: 0.5rem">
            <Hook0DropdownMenuItemLink
              v-for="(application, appIndex) in organizationGroup.applications"
              :key="appIndex"
              @click="
                goto(parent, {
                  name: routes.ApplicationsDashboard,
                  params: {
                    application_id: application.application_id,
                    organization_id: organizationGroup.organization.organization_id,
                  },
                })
              "
            >
              <Hook0Stack direction="row" align="center" gap="xs">
                <Hook0Text variant="primary" size="md">{{ application.name }}</Hook0Text>
                <Hook0Text variant="muted" size="sm">application</Hook0Text>
              </Hook0Stack>
            </Hook0DropdownMenuItemLink>
          </Hook0Stack>
        </Hook0Stack>
      </Hook0Stack>
      <Hook0DropdownMenuItemLink :to="{ name: routes.OrganizationsNew }">
        <Hook0Stack direction="row" align="center" gap="sm">
          <Plus :size="16" aria-hidden="true" />
          <Hook0Text variant="primary" size="md">{{
            t('orgAppSelector.newOrganization')
          }}</Hook0Text>
        </Hook0Stack>
      </Hook0DropdownMenuItemLink>
    </template>
  </Hook0Dropdown>
</template>

<style scoped>
/* Organization Card Header - interactive clickable area */
.org-card__header {
  cursor: pointer;
  margin: calc(-1 * var(--spacing-md));
  padding: var(--spacing-md);
  border-radius: var(--radius-md);
  transition: all 0.15s ease;
}

.org-card__header:hover {
  background-color: var(--color-bg-secondary);
}

.org-card__header:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  background-color: var(--color-bg-secondary);
}

.org-card__header:active {
  transform: scale(0.99);
  background-color: var(--color-bg-tertiary);
}
</style>
