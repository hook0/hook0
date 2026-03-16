<script setup lang="ts">
/**
 * Hook0Breadcrumbs - Detailed navigation breadcrumbs for all /organizations/* pages
 *
 * Design principles:
 * - Shows full navigation path with icons and links
 * - Organization and application names from context store
 * - Adapts based on current route depth
 * - Organization and application segments have dropdowns for quick switching
 * - All segments except the last one are clickable
 */
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ChevronRight, ChevronDown, Building2, Box, Home, Check, Plus } from 'lucide-vue-next';
import { useContextStore } from '@/stores/context';
import { useI18n } from 'vue-i18n';
import { routes } from '@/routes';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import type { UUID } from '@/http';

const route = useRoute();
const router = useRouter();
const contextStore = useContextStore();
const { t } = useI18n();

// Dropdown state
const orgDropdownOpen = ref(false);
const appDropdownOpen = ref(false);

interface Crumb {
  label: string;
  to: string | null;
  icon?: 'home' | 'organization' | 'application';
  dropdown?: 'organization' | 'application';
}

// Check if we're in an organization-related route
const isOrganizationRoute = computed(() => {
  return route.path.startsWith('/organizations');
});

// Check if we're in an application-related route
const isApplicationRoute = computed(() => {
  return isOrganizationRoute.value && route.params.application_id !== undefined;
});

// Get route parameters
const orgId = computed(() => route.params.organization_id as string | undefined);
const appId = computed(() => route.params.application_id as string | undefined);

// Fetch organizations and applications for dropdowns
const { data: organizations } = useOrganizationList();

const currentOrgId = computed(() => orgId.value ?? '');
const { data: applications } = useApplicationList(currentOrgId);

// Build the breadcrumb trail
const crumbs = computed<Crumb[]>(() => {
  const routeName = String(route.name ?? '');
  const result: Crumb[] = [];

  // Not an organization route - no breadcrumbs
  if (!isOrganizationRoute.value) {
    return result;
  }

  // Root: Organizations (links to home/organizations list)
  result.push({
    label: t('nav.organizations'),
    to: router.resolve({ name: routes.Home }).fullPath,
    icon: 'home',
  });

  // Special case: New Organization page
  if (routeName === 'OrganizationsNew') {
    result.push({
      label: t('breadcrumbs.newOrganization'),
      to: null,
    });
    return result;
  }

  // If we have an organization ID, add organization crumb
  if (orgId.value) {
    const orgName = contextStore.organizationName ?? t('common.loading');

    // Organization-level pages (no app context)
    if (!isApplicationRoute.value) {
      // Determine if this is a detail/settings page or list page
      if (routeName === 'OrganizationsDashboard') {
        // Dashboard - org name is the current page (dropdown enabled)
        result.push({
          label: orgName,
          to: null,
          icon: 'organization',
          dropdown: 'organization',
        });
      } else if (routeName === 'OrganizationsDetail') {
        // Org settings - show org then settings
        result.push({
          label: orgName,
          to: router.resolve({
            name: routes.OrganizationsDashboard,
            params: { organization_id: orgId.value },
          }).fullPath,
          icon: 'organization',
          dropdown: 'organization',
        });
        result.push({
          label: t('common.settings'),
          to: null,
        });
      } else if (routeName === 'ApplicationsList') {
        // Applications list
        result.push({
          label: orgName,
          to: router.resolve({
            name: routes.OrganizationsDashboard,
            params: { organization_id: orgId.value },
          }).fullPath,
          icon: 'organization',
          dropdown: 'organization',
        });
        result.push({
          label: t('nav.applications'),
          to: null,
        });
      } else if (routeName === 'ApplicationsNew') {
        // New application
        result.push({
          label: orgName,
          to: router.resolve({
            name: routes.OrganizationsDashboard,
            params: { organization_id: orgId.value },
          }).fullPath,
          icon: 'organization',
          dropdown: 'organization',
        });
        result.push({
          label: t('nav.applications'),
          to: router.resolve({
            name: routes.ApplicationsList,
            params: { organization_id: orgId.value },
          }).fullPath,
        });
        result.push({
          label: t('breadcrumbs.newApplication'),
          to: null,
        });
      } else if (routeName === 'ServicesTokenList') {
        // Service tokens list
        result.push({
          label: orgName,
          to: router.resolve({
            name: routes.OrganizationsDashboard,
            params: { organization_id: orgId.value },
          }).fullPath,
          icon: 'organization',
          dropdown: 'organization',
        });
        result.push({
          label: t('nav.serviceTokens'),
          to: null,
        });
      } else if (routeName === 'ServiceTokenView') {
        // Service token detail
        result.push({
          label: orgName,
          to: router.resolve({
            name: routes.OrganizationsDashboard,
            params: { organization_id: orgId.value },
          }).fullPath,
          icon: 'organization',
          dropdown: 'organization',
        });
        result.push({
          label: t('nav.serviceTokens'),
          to: router.resolve({
            name: routes.ServicesTokenList,
            params: { organization_id: orgId.value },
          }).fullPath,
        });
        result.push({
          label: t('breadcrumbs.serviceTokenDetail'),
          to: null,
        });
      } else {
        // Fallback for other org-level pages
        result.push({
          label: orgName,
          to: null,
          icon: 'organization',
          dropdown: 'organization',
        });
      }
    } else {
      // Application-level pages (have app context)
      const appName = contextStore.applicationName ?? t('common.loading');

      // Always show org first (clickable with dropdown)
      result.push({
        label: orgName,
        to: router.resolve({
          name: routes.OrganizationsDashboard,
          params: { organization_id: orgId.value },
        }).fullPath,
        icon: 'organization',
        dropdown: 'organization',
      });

      // Application Dashboard
      if (routeName === 'ApplicationsDashboard') {
        result.push({
          label: appName,
          to: null,
          icon: 'application',
          dropdown: 'application',
        });
      } else if (routeName === 'ApplicationsDetail') {
        // Application settings
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('common.settings'),
          to: null,
        });
      } else if (routeName === 'ApplicationSecretsList') {
        // API Keys list
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.apiKeys'),
          to: null,
        });
      } else if (routeName === 'EventTypesList') {
        // Event types list
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.eventTypes'),
          to: null,
        });
      } else if (routeName === 'EventTypesNew') {
        // New event type
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.eventTypes'),
          to: router.resolve({
            name: routes.EventTypesList,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
        });
        result.push({
          label: t('breadcrumbs.newEventType'),
          to: null,
        });
      } else if (routeName === 'EventsList') {
        // Events list
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.events'),
          to: null,
        });
      } else if (routeName === 'EventsDetail') {
        // Event detail
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.events'),
          to: router.resolve({
            name: routes.EventsList,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
        });
        const eventId = route.params.event_id as string;
        result.push({
          label: eventId ? `${eventId.substring(0, 8)}...` : t('breadcrumbs.eventDetail'),
          to: null,
        });
      } else if (routeName === 'SubscriptionsList') {
        // Subscriptions list
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.subscriptions'),
          to: null,
        });
      } else if (routeName === 'SubscriptionsNew') {
        // New subscription
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.subscriptions'),
          to: router.resolve({
            name: routes.SubscriptionsList,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
        });
        result.push({
          label: t('breadcrumbs.newSubscription'),
          to: null,
        });
      } else if (routeName === 'SubscriptionsDetail') {
        // Subscription detail
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.subscriptions'),
          to: router.resolve({
            name: routes.SubscriptionsList,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
        });
        result.push({
          label: t('breadcrumbs.subscriptionDetail'),
          to: null,
        });
      } else if (routeName === 'LogsList') {
        // Logs list
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('nav.logs'),
          to: null,
        });
      } else if (routeName === 'APIDocumentationForApplication') {
        // API Documentation for application
        result.push({
          label: appName,
          to: router.resolve({
            name: routes.ApplicationsDashboard,
            params: { organization_id: orgId.value, application_id: appId.value },
          }).fullPath,
          icon: 'application',
          dropdown: 'application',
        });
        result.push({
          label: t('common.documentation'),
          to: null,
        });
      } else {
        // Fallback for other app-level pages
        result.push({
          label: appName,
          to: null,
          icon: 'application',
          dropdown: 'application',
        });
      }
    }
  }

  return result;
});

// Show breadcrumbs for all organization routes
const shouldShow = computed(() => {
  return isOrganizationRoute.value && crumbs.value.length > 0;
});

// Dropdown toggle functions
function toggleOrgDropdown(event: Event) {
  event.preventDefault();
  event.stopPropagation();
  orgDropdownOpen.value = !orgDropdownOpen.value;
  appDropdownOpen.value = false;
}

function toggleAppDropdown(event: Event) {
  event.preventDefault();
  event.stopPropagation();
  appDropdownOpen.value = !appDropdownOpen.value;
  orgDropdownOpen.value = false;
}

function closeDropdowns() {
  orgDropdownOpen.value = false;
  appDropdownOpen.value = false;
}

// Map of organization-level features for preserving context when switching orgs
const ORG_LEVEL_FEATURES: Record<string, string> = {
  [routes.OrganizationsDashboard]: routes.OrganizationsDashboard,
  [routes.OrganizationsDetail]: routes.OrganizationsDetail,
  [routes.ServicesTokenList]: routes.ServicesTokenList,
  [routes.ServiceTokenView]: routes.ServicesTokenList,
  [routes.ApplicationsList]: routes.ApplicationsList,
  [routes.ApplicationsNew]: routes.ApplicationsList,
};

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
  [routes.APIDocumentationForApplication]: routes.APIDocumentationForApplication,
};

function navigateToOrg(newOrgId: UUID) {
  closeDropdowns();

  // Try to preserve the current feature when switching orgs
  const currentRouteName = String(route.name ?? '');

  // If we're on an org-level feature, try to preserve it
  const preservedOrgFeature = ORG_LEVEL_FEATURES[currentRouteName];
  if (preservedOrgFeature) {
    void router.push({
      name: preservedOrgFeature,
      params: { organization_id: newOrgId },
    });
    return;
  }

  // Default: go to org dashboard
  void router.push({
    name: routes.OrganizationsDashboard,
    params: { organization_id: newOrgId },
  });
}

function navigateToApp(newOrgId: UUID, newAppId: UUID) {
  closeDropdowns();

  // Try to preserve the current feature when switching apps
  const currentRouteName = String(route.name ?? '');
  const preservedFeature = APP_LEVEL_FEATURES[currentRouteName];

  if (preservedFeature) {
    void router.push({
      name: preservedFeature,
      params: {
        organization_id: newOrgId,
        application_id: newAppId,
      },
    });
    return;
  }

  // Default: go to app dashboard
  void router.push({
    name: routes.ApplicationsDashboard,
    params: {
      organization_id: newOrgId,
      application_id: newAppId,
    },
  });
}

function navigateToNewOrg() {
  closeDropdowns();
  void router.push({ name: routes.OrganizationsNew });
}

function navigateToNewApp() {
  closeDropdowns();
  if (orgId.value) {
    void router.push({
      name: routes.ApplicationsNew,
      params: { organization_id: orgId.value },
    });
  }
}

// Close dropdowns when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement;
  if (!target.closest('.hook0-breadcrumbs__dropdown-trigger')) {
    closeDropdowns();
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <nav v-if="shouldShow" :aria-label="t('common.breadcrumb')" class="hook0-breadcrumbs">
    <ol class="hook0-breadcrumbs__list">
      <li v-for="(crumb, index) in crumbs" :key="index" class="hook0-breadcrumbs__item">
        <ChevronRight
          v-if="index > 0"
          :size="12"
          class="hook0-breadcrumbs__separator"
          aria-hidden="true"
        />

        <!-- Organization dropdown trigger -->
        <template v-if="crumb.dropdown === 'organization'">
          <div class="hook0-breadcrumbs__dropdown-wrapper">
            <button
              type="button"
              class="hook0-breadcrumbs__dropdown-trigger"
              :class="{
                'hook0-breadcrumbs__dropdown-trigger--link': crumb.to,
                'hook0-breadcrumbs__dropdown-trigger--current': !crumb.to,
              }"
              :aria-expanded="orgDropdownOpen"
              aria-haspopup="listbox"
              @click="toggleOrgDropdown"
            >
              <Building2 :size="14" class="hook0-breadcrumbs__icon" aria-hidden="true" />
              {{ crumb.label }}
              <ChevronDown :size="12" class="hook0-breadcrumbs__chevron" aria-hidden="true" />
            </button>

            <!-- Organization dropdown menu -->
            <div
              v-if="orgDropdownOpen"
              class="hook0-breadcrumbs__dropdown"
              role="listbox"
              :aria-label="t('breadcrumbs.selectOrganization')"
            >
              <div class="hook0-breadcrumbs__dropdown-content">
                <button
                  v-for="org in organizations"
                  :key="org.organization_id"
                  type="button"
                  role="option"
                  class="hook0-breadcrumbs__dropdown-item"
                  :class="{
                    'hook0-breadcrumbs__dropdown-item--selected': org.organization_id === orgId,
                  }"
                  :aria-selected="org.organization_id === orgId"
                  @click="navigateToOrg(org.organization_id)"
                >
                  <Building2 :size="14" aria-hidden="true" />
                  <span class="hook0-breadcrumbs__dropdown-item-label">{{ org.name }}</span>
                  <Check
                    v-if="org.organization_id === orgId"
                    :size="14"
                    class="hook0-breadcrumbs__dropdown-check"
                    aria-hidden="true"
                  />
                </button>
                <div class="hook0-breadcrumbs__dropdown-divider"></div>
                <button
                  type="button"
                  class="hook0-breadcrumbs__dropdown-item hook0-breadcrumbs__dropdown-item--action"
                  @click="navigateToNewOrg"
                >
                  <Plus :size="14" aria-hidden="true" />
                  <span class="hook0-breadcrumbs__dropdown-item-label">{{
                    t('breadcrumbs.newOrganization')
                  }}</span>
                </button>
              </div>
            </div>
          </div>
        </template>

        <!-- Application dropdown trigger -->
        <template v-else-if="crumb.dropdown === 'application'">
          <div class="hook0-breadcrumbs__dropdown-wrapper">
            <button
              type="button"
              class="hook0-breadcrumbs__dropdown-trigger"
              :class="{
                'hook0-breadcrumbs__dropdown-trigger--link': crumb.to,
                'hook0-breadcrumbs__dropdown-trigger--current': !crumb.to,
              }"
              :aria-expanded="appDropdownOpen"
              aria-haspopup="listbox"
              @click="toggleAppDropdown"
            >
              <Box :size="14" class="hook0-breadcrumbs__icon" aria-hidden="true" />
              {{ crumb.label }}
              <ChevronDown :size="12" class="hook0-breadcrumbs__chevron" aria-hidden="true" />
            </button>

            <!-- Application dropdown menu -->
            <div
              v-if="appDropdownOpen"
              class="hook0-breadcrumbs__dropdown"
              role="listbox"
              :aria-label="t('breadcrumbs.selectApplication')"
            >
              <div class="hook0-breadcrumbs__dropdown-content">
                <template v-if="applications && applications.length > 0">
                  <button
                    v-for="app in applications"
                    :key="app.application_id"
                    type="button"
                    role="option"
                    class="hook0-breadcrumbs__dropdown-item"
                    :class="{
                      'hook0-breadcrumbs__dropdown-item--selected': app.application_id === appId,
                    }"
                    :aria-selected="app.application_id === appId"
                    @click="navigateToApp(app.organization_id, app.application_id)"
                  >
                    <Box :size="14" aria-hidden="true" />
                    <span class="hook0-breadcrumbs__dropdown-item-label">{{ app.name }}</span>
                    <Check
                      v-if="app.application_id === appId"
                      :size="14"
                      class="hook0-breadcrumbs__dropdown-check"
                      aria-hidden="true"
                    />
                  </button>
                </template>
                <div v-else class="hook0-breadcrumbs__dropdown-empty">
                  {{ t('applications.empty.title') }}
                </div>
                <div class="hook0-breadcrumbs__dropdown-divider"></div>
                <button
                  type="button"
                  class="hook0-breadcrumbs__dropdown-item hook0-breadcrumbs__dropdown-item--action"
                  @click="navigateToNewApp"
                >
                  <Plus :size="14" aria-hidden="true" />
                  <span class="hook0-breadcrumbs__dropdown-item-label">{{
                    t('breadcrumbs.newApplication')
                  }}</span>
                </button>
              </div>
            </div>
          </div>
        </template>

        <!-- Regular link (no dropdown) -->
        <router-link v-else-if="crumb.to" :to="crumb.to" class="hook0-breadcrumbs__link">
          <Home
            v-if="crumb.icon === 'home'"
            :size="14"
            class="hook0-breadcrumbs__icon"
            aria-hidden="true"
          />
          <Building2
            v-else-if="crumb.icon === 'organization'"
            :size="14"
            class="hook0-breadcrumbs__icon"
            aria-hidden="true"
          />
          <Box
            v-else-if="crumb.icon === 'application'"
            :size="14"
            class="hook0-breadcrumbs__icon"
            aria-hidden="true"
          />
          {{ crumb.label }}
        </router-link>

        <!-- Current page (no dropdown, no link) -->
        <span v-else class="hook0-breadcrumbs__current" aria-current="page">
          <Home
            v-if="crumb.icon === 'home'"
            :size="14"
            class="hook0-breadcrumbs__icon"
            aria-hidden="true"
          />
          <Building2
            v-else-if="crumb.icon === 'organization'"
            :size="14"
            class="hook0-breadcrumbs__icon"
            aria-hidden="true"
          />
          <Box
            v-else-if="crumb.icon === 'application'"
            :size="14"
            class="hook0-breadcrumbs__icon"
            aria-hidden="true"
          />
          {{ crumb.label }}
        </span>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.hook0-breadcrumbs {
  margin-bottom: 0.75rem;
}

.hook0-breadcrumbs__list {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  list-style: none;
  padding: 0;
  margin: 0;
  flex-wrap: wrap;
}

.hook0-breadcrumbs__item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.hook0-breadcrumbs__link {
  display: inline-flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.25rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  text-decoration: none;
  transition: color 0.15s ease;
}

.hook0-breadcrumbs__link:hover {
  color: var(--color-text-secondary);
}

.hook0-breadcrumbs__current {
  display: inline-flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.25rem;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.hook0-breadcrumbs__separator {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.hook0-breadcrumbs__icon {
  flex-shrink: 0;
}

/* Dropdown wrapper */
.hook0-breadcrumbs__dropdown-wrapper {
  position: relative;
}

/* Dropdown trigger button */
.hook0-breadcrumbs__dropdown-trigger {
  display: inline-flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.25rem;
  font-size: 0.8125rem;
  font-family: inherit;
  background: none;
  border: none;
  padding: 0.125rem 0.25rem;
  margin: -0.125rem -0.25rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
}

.hook0-breadcrumbs__dropdown-trigger--link {
  color: var(--color-text-muted);
}

.hook0-breadcrumbs__dropdown-trigger--link:hover {
  color: var(--color-text-secondary);
  background-color: var(--color-bg-secondary);
}

.hook0-breadcrumbs__dropdown-trigger--current {
  color: var(--color-text-secondary);
}

.hook0-breadcrumbs__dropdown-trigger--current:hover {
  background-color: var(--color-bg-secondary);
}

.hook0-breadcrumbs__dropdown-trigger:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-breadcrumbs__chevron {
  flex-shrink: 0;
  opacity: 0.6;
  transition: transform 0.15s ease;
}

.hook0-breadcrumbs__dropdown-trigger[aria-expanded='true'] .hook0-breadcrumbs__chevron {
  transform: rotate(180deg);
}

/* Dropdown menu */
.hook0-breadcrumbs__dropdown {
  position: absolute;
  top: calc(100% + 0.25rem);
  left: 0;
  z-index: 50;
  min-width: 12rem;
  max-width: 20rem;
  background-color: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  animation: dropdown-fade-in 0.15s ease;
}

@keyframes dropdown-fade-in {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.hook0-breadcrumbs__dropdown-content {
  padding: 0.25rem;
  max-height: 20rem;
  overflow-y: auto;
}

/* Dropdown item */
.hook0-breadcrumbs__dropdown-item {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  font-family: inherit;
  color: var(--color-text-primary);
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-align: left;
  transition: background-color 0.15s ease;
}

.hook0-breadcrumbs__dropdown-item :deep(svg) {
  flex-shrink: 0;
}

.hook0-breadcrumbs__dropdown-item:hover {
  background-color: var(--color-bg-secondary);
}

.hook0-breadcrumbs__dropdown-item:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.hook0-breadcrumbs__dropdown-item--selected {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.hook0-breadcrumbs__dropdown-item--selected:hover {
  background-color: var(--color-primary-light);
}

.hook0-breadcrumbs__dropdown-item--action {
  color: var(--color-primary);
}

.hook0-breadcrumbs__dropdown-item-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hook0-breadcrumbs__dropdown-check {
  flex-shrink: 0;
  color: var(--color-primary);
}

/* Dropdown divider */
.hook0-breadcrumbs__dropdown-divider {
  height: 1px;
  margin: 0.25rem 0;
  background-color: var(--color-border);
}

/* Empty state */
.hook0-breadcrumbs__dropdown-empty {
  padding: 0.75rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  text-align: center;
}
</style>
