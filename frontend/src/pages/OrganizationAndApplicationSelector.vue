<script setup lang="ts">
import { RouteLocationRaw, RouteParams, useRoute, useRouter } from 'vue-router';
import { onMounted, onUnmounted, onUpdated, ref, watch } from 'vue';
import { Box, Plus, ChevronRight, Package, ChevronDown, FolderKanban } from 'lucide-vue-next';

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
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import { useAuthStore } from '@/stores/auth';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type ApplicationsPerOrganization = {
  organization: Organization;
  applications: Array<Application>;
};

const router = useRouter();
const route = useRoute();

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
                'should never happen, application is linked to unknown organization. Silent fail'
              );
              return m;
            }

            let organization_in_group = m.find(
              (item) => item.organization.organization_id === application.organization_id
            );

            if (!organization_in_group) {
              console.error(
                'should never happen, application is linked to unknown organization. Silent fail'
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

    application_name.value = matchedApp ? matchedApp.name : t('orgAppSelector.selectAnApplication');
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

onMounted(() => {
  if (useAuthStore().accessToken) {
    void getApplicationsPerOrganization().then((result) => {
      applicationsPerOrganization.value = result;
    });
  }

  removeRouterGuard.value = router.afterEach(() => {
    return _updateDropdown(route.params);
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

// Intentionally hardcoded: design decision for org avatar variety, not design-token eligible.
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
        class="stagger-item"
      >
        <!-- Card Header -->
        <Hook0CardContent compact>
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
              size="md"
              variant="square"
              :gradient="getGradient(index)"
            />
            <Hook0Stack direction="column" gap="none" class="org-card__info">
              <span class="org-selector__org-name">
                {{ organizationGroup.organization.name }}
              </span>
              <span class="org-card__meta">
                <span class="org-card__status-dot" aria-hidden="true" />
                <span class="org-selector__apps-count">
                  {{ t('orgAppSelector.appsCount', organizationGroup.applications.length) }}
                </span>
              </span>
            </Hook0Stack>
            <ChevronRight :size="15" aria-hidden="true" style="margin-top: 0.125rem" />
          </Hook0Stack>
        </Hook0CardContent>

        <!-- Applications List -->
        <Hook0CardContent compact>
          <template v-if="organizationGroup.applications.length > 0">
            <ul class="app-list">
              <li
                v-for="application in organizationGroup.applications"
                :key="application.application_id"
                class="app-list__item"
                role="button"
                tabindex="0"
                @click="
                  navigateToApp(
                    organizationGroup.organization.organization_id,
                    application.application_id
                  )
                "
                @keydown.enter="
                  navigateToApp(
                    organizationGroup.organization.organization_id,
                    application.application_id
                  )
                "
              >
                <span class="app-list__icon">
                  <Box :size="16" aria-hidden="true" />
                </span>
                <span class="app-list__name">
                  {{ application.name }}
                </span>
                <ChevronRight :size="14" aria-hidden="true" class="app-list__chevron" />
              </li>
            </ul>
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
        </Hook0CardContent>

        <!-- Create App Button -->
        <div class="org-card__action-row">
          <router-link
            :to="{
              name: routes.ApplicationsNew,
              params: { organization_id: organizationGroup.organization.organization_id },
            }"
            class="app-list__action"
          >
            <span class="app-list__action-icon">
              <Plus :size="16" aria-hidden="true" />
            </span>
            <span class="app-list__action-label">
              {{ t('orgAppSelector.createNewApplication') }}
            </span>
          </router-link>
        </div>
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
            <span class="org-selector__org-name">
              {{ t('orgAppSelector.newOrganization') }}
            </span>
            <span class="org-selector__apps-count">
              {{ t('orgAppSelector.newOrganizationDescription') }}
            </span>
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
            <span class="org-selector__dropdown-org">{{ organization_name }}</span>
            <span class="org-selector__dropdown-app">{{ application_name }}</span>
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
              <span class="org-selector__dropdown-item">
                {{ organizationGroup.organization.name }}
              </span>
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
                <span class="org-selector__dropdown-item">{{ application.name }}</span>
                <span class="org-selector__dropdown-label">{{
                  t('orgAppSelector.applicationLabel')
                }}</span>
              </Hook0Stack>
            </Hook0DropdownMenuItemLink>
          </Hook0Stack>
        </Hook0Stack>
      </Hook0Stack>
      <Hook0DropdownMenuItemLink :to="{ name: routes.OrganizationsNew }">
        <Hook0Stack direction="row" align="center" gap="sm">
          <Plus :size="16" aria-hidden="true" />
          <span class="org-selector__dropdown-item">{{ t('orgAppSelector.newOrganization') }}</span>
        </Hook0Stack>
      </Hook0DropdownMenuItemLink>
    </template>
  </Hook0Dropdown>
</template>

<style scoped>
.org-card__header {
  cursor: pointer;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-lg);
  transition:
    background-color 0.15s ease,
    transform 0.15s ease;
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

/* Application List */
.app-list {
  list-style: none;
  padding: 0;
  margin: 0.5rem 0 0;
}

.app-list__item {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 0.875rem;
  transition: background-color 0.15s ease;
}

.app-list__item:hover {
  background-color: var(--color-bg-secondary);
}

.app-list__item:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.app-list__icon {
  flex-shrink: 0;
  width: 1.75rem;
  height: 1.75rem;
  margin-right: 0.625rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.app-list__item:hover .app-list__icon {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.app-list__name {
  flex: 1;
  min-width: 0;
  color: var(--color-text-primary);
  font-weight: 500;
  font-size: 0.875rem;
}

.app-list__chevron {
  flex-shrink: 0;
  margin-left: 0.5rem;
  color: var(--color-text-tertiary);
  opacity: 0;
  transition:
    opacity 0.15s ease,
    color 0.15s ease;
}

.app-list__item:hover .app-list__chevron {
  opacity: 1;
  color: var(--color-text-secondary);
}

/* Create App Action Row */
.org-card__action-row {
  padding: 0 0.375rem 0.375rem;
}

/* Create App Action Button */
.app-list__action {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  cursor: pointer;
  text-align: left;
  font-size: 0.8125rem;
  font-family: inherit;
  transition: background-color 0.15s ease;
}

.app-list__action:hover {
  background-color: var(--color-primary-light);
}

.app-list__action:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.app-list__action-icon {
  flex-shrink: 0;
  width: 1.75rem;
  height: 1.75rem;
  margin-right: 0.625rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1.5px dashed var(--color-border-strong);
  color: var(--color-text-tertiary);
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    color 0.15s ease;
}

.app-list__action:hover .app-list__action-icon {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.app-list__action-label {
  color: var(--color-text-secondary);
  font-weight: 500;
  transition: color 0.15s ease;
}

.app-list__action:hover .app-list__action-label {
  color: var(--color-primary);
}

/* Organization Selector Text */
.org-card__info {
  flex: 1;
  min-width: 0;
  justify-content: center;
}

.org-selector__org-name {
  font-size: 0.875rem;
  font-weight: 600;
  line-height: 1.3;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  /* Allow wrapping on small screens */
  white-space: normal;
  word-break: break-word;
}

.org-card__meta {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.org-card__status-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background-color: var(--color-primary);
  flex-shrink: 0;
}

.org-selector__apps-count {
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
}

.org-selector__dropdown-org {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.org-selector__dropdown-app {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.org-selector__dropdown-item {
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.org-selector__dropdown-label {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}
</style>
