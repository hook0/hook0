<script setup lang="ts">
/**
 * Hook0MobileDrawer - Mobile navigation drawer
 *
 * A slide-up drawer for mobile navigation that shows:
 * - Workspace selector (org/app)
 * - Navigation links based on context
 * - User menu options
 */
import { computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import {
  X,
  Building2,
  Box,
  ChevronRight,
  Calendar,
  Webhook,
  ListChecks,
  FileText,
  Key,
  Settings,
  BookOpen,
  Code2,
  ExternalLink,
  LogOut,
  Sun,
  Moon,
  Plus,
} from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';

const { t } = useI18n();
const route = useRoute();
const authStore = useAuthStore();
const contextStore = useContextStore();
const uiStore = useUiStore();

// Data
const { data: organizations } = useOrganizationList();
const { data: applications } = useApplicationList(
  computed(() => contextStore.organizationId ?? '')
);

// Close drawer on route change
watch(
  () => route.fullPath,
  () => {
    uiStore.closeMobileDrawer();
  }
);

// Navigation items based on context
interface NavItem {
  id: string;
  label: string;
  icon: typeof Calendar;
  to: { name: string; params?: Record<string, string> };
  active: boolean;
}

const navItems = computed<NavItem[]>(() => {
  const orgId = contextStore.organizationId;
  const appId = contextStore.applicationId;

  // App-level navigation
  if (orgId && appId) {
    const params = { organization_id: orgId, application_id: appId };
    return [
      {
        id: 'events',
        label: t('nav.events'),
        icon: Calendar,
        to: { name: routes.EventsList, params },
        active: route.name === routes.EventsList || route.name === routes.EventsDetail,
      },
      {
        id: 'subscriptions',
        label: t('nav.subscriptions'),
        icon: Webhook,
        to: { name: routes.SubscriptionsList, params },
        active:
          route.name === routes.SubscriptionsList ||
          route.name === routes.SubscriptionsNew ||
          route.name === routes.SubscriptionsDetail,
      },
      {
        id: 'event-types',
        label: t('nav.eventTypes'),
        icon: ListChecks,
        to: { name: routes.EventTypesList, params },
        active: route.name === routes.EventTypesList || route.name === routes.EventTypesNew,
      },
      {
        id: 'logs',
        label: t('nav.logs'),
        icon: FileText,
        to: { name: routes.LogsList, params },
        active: route.name === routes.LogsList,
      },
      {
        id: 'api-keys',
        label: t('nav.apiKeys'),
        icon: Key,
        to: { name: routes.ApplicationSecretsList, params },
        active: route.name === routes.ApplicationSecretsList,
      },
      {
        id: 'settings',
        label: t('nav.settings'),
        icon: Settings,
        to: { name: routes.ApplicationsDashboard, params },
        active:
          route.name === routes.ApplicationsDashboard || route.name === routes.ApplicationsDetail,
      },
    ];
  }

  // Org-level navigation
  if (orgId) {
    const params = { organization_id: orgId };
    return [
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
        id: 'org-settings',
        label: t('nav.settings'),
        icon: Settings,
        to: { name: routes.OrganizationsDashboard, params },
        active:
          route.name === routes.OrganizationsDashboard || route.name === routes.OrganizationsDetail,
      },
    ];
  }

  return [];
});

function handleClose() {
  uiStore.closeMobileDrawer();
}

function handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    handleClose();
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="backdrop">
      <div
        v-if="uiStore.mobileDrawerOpen"
        class="hook0-mobile-drawer-backdrop"
        @click="handleBackdropClick"
      >
        <Transition name="drawer">
          <div
            v-if="uiStore.mobileDrawerOpen"
            class="hook0-mobile-drawer"
            role="dialog"
            aria-modal="true"
          >
            <!-- Handle -->
            <div class="hook0-mobile-drawer__handle" />

            <!-- Header -->
            <div class="hook0-mobile-drawer__header">
              <h2 class="hook0-mobile-drawer__title">{{ t('nav.openMenu') }}</h2>
              <Hook0Button
                variant="ghost"
                class="hook0-mobile-drawer__close"
                :aria-label="t('common.close')"
                @click="handleClose"
              >
                <X :size="20" aria-hidden="true" />
              </Hook0Button>
            </div>

            <!-- Content -->
            <div class="hook0-mobile-drawer__content">
              <!-- Workspace Section -->
              <div class="hook0-mobile-drawer__section">
                <div class="hook0-mobile-drawer__section-header">
                  <Building2 :size="14" aria-hidden="true" />
                  {{ t('nav.organizations') }}
                </div>
                <div class="hook0-mobile-drawer__list">
                  <Hook0Button
                    v-for="org in organizations"
                    :key="org.organization_id"
                    variant="link"
                    :to="{
                      name: routes.ApplicationsList,
                      params: { organization_id: org.organization_id },
                    }"
                    class="hook0-mobile-drawer__item"
                    :class="{ active: org.organization_id === contextStore.organizationId }"
                  >
                    <span class="hook0-mobile-drawer__item-name">{{ org.name }}</span>
                    <ChevronRight
                      :size="16"
                      class="hook0-mobile-drawer__item-chevron"
                      aria-hidden="true"
                    />
                  </Hook0Button>
                  <Hook0Button
                    variant="link"
                    :to="{ name: routes.OrganizationsNew }"
                    class="hook0-mobile-drawer__item hook0-mobile-drawer__item--create"
                  >
                    <Plus :size="16" aria-hidden="true" />
                    {{ t('nav.newOrganization') }}
                  </Hook0Button>
                </div>
              </div>

              <!-- Applications (if org selected) -->
              <div
                v-if="contextStore.organizationId && applications && applications.length > 0"
                class="hook0-mobile-drawer__section"
              >
                <div class="hook0-mobile-drawer__section-header">
                  <Box :size="14" aria-hidden="true" />
                  {{ t('nav.applications') }}
                </div>
                <div class="hook0-mobile-drawer__list">
                  <Hook0Button
                    v-for="app in applications"
                    :key="app.application_id"
                    variant="link"
                    :to="{
                      name: routes.EventsList,
                      params: { organization_id: contextStore.organizationId!, application_id: app.application_id },
                    }"
                    class="hook0-mobile-drawer__item"
                    :class="{ active: app.application_id === contextStore.applicationId }"
                  >
                    <span class="hook0-mobile-drawer__item-name">{{ app.name }}</span>
                    <ChevronRight
                      :size="16"
                      class="hook0-mobile-drawer__item-chevron"
                      aria-hidden="true"
                    />
                  </Hook0Button>
                  <Hook0Button
                    variant="link"
                    :to="{
                      name: routes.ApplicationsNew,
                      params: { organization_id: contextStore.organizationId! },
                    }"
                    class="hook0-mobile-drawer__item hook0-mobile-drawer__item--create"
                  >
                    <Plus :size="16" aria-hidden="true" />
                    {{ t('nav.newApplication') }}
                  </Hook0Button>
                </div>
              </div>

              <!-- Navigation -->
              <div v-if="navItems.length > 0" class="hook0-mobile-drawer__section">
                <div class="hook0-mobile-drawer__section-header">{{ t('common.navigation') }}</div>
                <div class="hook0-mobile-drawer__list">
                  <Hook0Button
                    v-for="item in navItems"
                    :key="item.id"
                    variant="link"
                    :to="item.to"
                    class="hook0-mobile-drawer__item"
                    :class="{ active: item.active }"
                  >
                    <component :is="item.icon" :size="18" aria-hidden="true" />
                    {{ item.label }}
                  </Hook0Button>
                </div>
              </div>

              <!-- Quick Actions -->
              <div class="hook0-mobile-drawer__section">
                <div class="hook0-mobile-drawer__section-header">
                  <BookOpen :size="14" aria-hidden="true" />
                  {{ t('nav.documentation') }}
                </div>
                <div class="hook0-mobile-drawer__list">
                  <a
                    href="https://documentation.hook0.com/"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="hook0-mobile-drawer__item"
                  >
                    <BookOpen :size="18" aria-hidden="true" />
                    {{ t('nav.documentation') }}
                    <ExternalLink
                      :size="14"
                      class="hook0-mobile-drawer__item-chevron"
                      aria-hidden="true"
                    />
                  </a>
                  <a
                    href="https://documentation.hook0.com/api"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="hook0-mobile-drawer__item"
                  >
                    <Code2 :size="18" aria-hidden="true" />
                    {{ t('nav.apiReference') }}
                    <ExternalLink
                      :size="14"
                      class="hook0-mobile-drawer__item-chevron"
                      aria-hidden="true"
                    />
                  </a>
                </div>
              </div>

              <!-- Preferences -->
              <div class="hook0-mobile-drawer__section">
                <div class="hook0-mobile-drawer__list">
                  <Hook0Button
                    variant="ghost"
                    class="hook0-mobile-drawer__item"
                    @click="uiStore.toggleColorMode()"
                  >
                    <Sun
                      v-if="uiStore.effectiveColorMode === 'dark'"
                      :size="18"
                      aria-hidden="true"
                    />
                    <Moon v-else :size="18" aria-hidden="true" />
                    {{
                      uiStore.effectiveColorMode === 'dark' ? t('nav.lightMode') : t('nav.darkMode')
                    }}
                  </Hook0Button>
                  <Hook0Button
                    variant="link"
                    :to="{ name: routes.UserSettings }"
                    class="hook0-mobile-drawer__item"
                  >
                    <Settings :size="18" aria-hidden="true" />
                    {{ t('nav.settings') }}
                  </Hook0Button>
                </div>
              </div>
            </div>

            <!-- Footer -->
            <div class="hook0-mobile-drawer__footer">
              <Hook0Button
                variant="danger"
                class="hook0-mobile-drawer__logout"
                @click="authStore.logout()"
              >
                <LogOut :size="18" aria-hidden="true" />
                {{ t('nav.logout') }}
              </Hook0Button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.hook0-mobile-drawer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
}

.hook0-mobile-drawer {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 50;
  display: flex;
  flex-direction: column;
  max-height: 85vh;
  background-color: var(--color-bg-primary);
  border-top-left-radius: var(--radius-xl);
  border-top-right-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
}

.hook0-mobile-drawer__handle {
  width: 2.5rem;
  height: 0.25rem;
  margin: 0.75rem auto 0;
  background-color: var(--color-border-strong);
  border-radius: var(--radius-full);
}

.hook0-mobile-drawer__header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-mobile-drawer__title {
  flex: 1;
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-mobile-drawer__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  padding: 0;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.hook0-mobile-drawer__close:hover {
  background-color: var(--color-bg-secondary);
  color: var(--color-text-primary);
}

.hook0-mobile-drawer__content {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  -webkit-overflow-scrolling: touch;
}

.hook0-mobile-drawer__section {
  margin-bottom: 0.5rem;
}

.hook0-mobile-drawer__section-header {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
}

.hook0-mobile-drawer__section-header :deep(svg) {
  flex-shrink: 0;
}

.hook0-mobile-drawer__list {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.hook0-mobile-drawer__item {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  justify-content: flex-start;
  gap: 0.75rem;
  padding: 0.75rem;
  font-size: 0.9375rem;
  color: var(--color-text-secondary);
  text-decoration: none;
  border: none;
  background: none;
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: background-color 0.15s ease, color 0.15s ease;
  width: 100%;
  text-align: left;
  white-space: nowrap;
}

.hook0-mobile-drawer__item :deep(svg) {
  flex-shrink: 0;
}

.hook0-mobile-drawer__item:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-mobile-drawer__item.active {
  background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.hook0-mobile-drawer__item--create {
  color: var(--color-primary);
}

.hook0-mobile-drawer__item-name {
  flex: 1;
}

.hook0-mobile-drawer__item-chevron {
  color: var(--color-text-muted);
}

.hook0-mobile-drawer__footer {
  padding: 0.75rem;
  padding-bottom: calc(0.75rem + env(safe-area-inset-bottom, 0));
  border-top: 1px solid var(--color-border);
}

.hook0-mobile-drawer__logout {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.75rem;
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--color-error);
  border: 1px solid var(--color-error);
  background: transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.hook0-mobile-drawer__logout:hover {
  background-color: var(--color-error-light);
}

/* Transitions */
.backdrop-enter-active,
.backdrop-leave-active {
  transition: opacity 0.2s ease;
}

.backdrop-enter-from,
.backdrop-leave-to {
  opacity: 0;
}

.drawer-enter-active,
.drawer-leave-active {
  transition: transform 0.3s ease;
}

.drawer-enter-from,
.drawer-leave-to {
  transform: translateY(100%);
}
</style>
