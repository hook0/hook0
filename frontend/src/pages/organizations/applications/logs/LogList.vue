<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { X, Filter } from 'lucide-vue-next';
import { RouterLink } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';

import { Send } from 'lucide-vue-next';
import { DOCS_LOGS_URL, API_DOCS_LOGS_URL } from '@/constants/externalLinks';

import { useLogList } from './useLogQueries';
import { useLogColumns } from './useLogColumns';
import type { RequestAttemptExtended } from './LogService';
import { routes } from '@/routes';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';

import EventSidePanel from '@/pages/organizations/applications/events/EventSidePanel.vue';
import Hook0DocButtons from '@/components/Hook0DocButtons.vue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { organizationId, applicationId } = useRouteIds();
const subscriptionIdFilter = computed(() => (route.query.subscription_id as string) || undefined);
const subscriptionNameFilter = computed(() => route.query.subscription_name as string);
const {
  data: requestAttempts,
  isLoading,
  error,
  refetch,
} = useLogList(applicationId, subscriptionIdFilter);
const { data: organization } = useOrganizationDetail(organizationId);

const retentionDays = computed(
  () => organization.value?.quotas.days_of_events_retention_limit ?? 7
);

const columns = useLogColumns();

// Side panel state
const sidePanelOpen = ref(false);
const selectedEventId = ref('');

function handleRowClick(row: RequestAttemptExtended) {
  selectedEventId.value = row.event_id;
  sidePanelOpen.value = true;
}

function closeSidePanel() {
  sidePanelOpen.value = false;
}

function clearSubscriptionFilter() {
  const { subscription_id: _, ...rest } = route.query;
  void router.replace({ query: rest });
}
</script>

<template>
  <Hook0PageLayout :title="t('logs.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !requestAttempts" data-test="logs-card">
      <Hook0CardHeader>
        <template #header>{{ t('logs.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (requestAttempts is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="logs-card">
        <Hook0CardHeader>
          <template #header>{{ t('logs.title') }}</template>
          <template #subtitle>
            {{ t('logs.subtitle') }}
            <Hook0HelpText tone="emphasis">{{
              t('logs.subtitleRetention', { days: retentionDays })
            }}</Hook0HelpText>
          </template>
          <template #actions>
            <Hook0DocButtons :doc-url="DOCS_LOGS_URL" :api-url="API_DOCS_LOGS_URL" />
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="subscriptionIdFilter" class="logs-filter-bar">
          <Filter :size="14" class="logs-filter-bar__icon" aria-hidden="true" />
          <Hook0Badge variant="info" size="sm">
            <RouterLink
              :to="{
                name: routes.SubscriptionsEdit,
                params: {
                  organization_id: route.params.organization_id,
                  application_id: route.params.application_id,
                  subscription_id: subscriptionIdFilter,
                },
              }"
              class="logs-filter-badge__link"
            >
              {{ subscriptionNameFilter }}
            </RouterLink>
            <button
              class="logs-filter-badge__remove"
              :aria-label="t('common.remove')"
              @click="clearSubscriptionFilter"
            >
              <X :size="14" aria-hidden="true" />
            </button>
          </Hook0Badge>
        </Hook0CardContent>

        <Hook0CardContent v-if="requestAttempts.length > 0">
          <Hook0Table
            data-test="logs-table"
            :columns="columns"
            :data="requestAttempts"
            row-id-field="event_id"
            clickable-rows
            @row-click="handleRowClick"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('logs.empty.title')"
            :description="t('logs.empty.description')"
            :icon="Send"
          >
            <template #action>
              <Hook0Button
                variant="primary"
                :to="{
                  name: routes.SubscriptionsNew,
                  params: {
                    organization_id: route.params.organization_id,
                    application_id: route.params.application_id,
                  },
                }"
              >
                {{ t('subscriptions.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>
      </Hook0Card>
    </template>

    <!-- Event side panel -->
    <EventSidePanel
      :open="sidePanelOpen"
      :event-id="selectedEventId"
      :application-id="applicationId"
      @close="closeSidePanel"
    />
  </Hook0PageLayout>
</template>

<style scoped>
/* Column cell styles rendered via h() in useLogColumns.ts — :deep() required
   because VNodes are created outside this SFC's scope. */

:deep(.log-status) {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.1875rem 0.625rem;
  border-radius: var(--radius-full);
  font-size: 0.8125rem;
  font-weight: 600;
  white-space: nowrap;
  cursor: default;
}

:deep(.log-status__dot) {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: currentColor;
  flex-shrink: 0;
}

:deep(.log-status--success) {
  background-color: var(--color-success-light);
  color: var(--color-success);
}

:deep(.log-status--error) {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

:deep(.log-status--warning) {
  background-color: var(--color-warning-light);
  color: var(--color-warning);
}

:deep(.log-status--info) {
  background-color: var(--color-info-light);
  color: var(--color-info);
}

:deep(.log-status--muted) {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-muted);
}

:deep(.log-event-cell) {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

:deep(.log-event-link) {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  text-decoration: none;
}

:deep(.log-event-link:hover) {
  text-decoration: underline;
  text-underline-offset: 2px;
}

:deep(.log-event-type) {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

:deep(.log-duration) {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  cursor: default;
}

.logs-filter-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-info-light);
}

.logs-filter-bar__icon {
  color: var(--color-info);
  flex-shrink: 0;
}

.logs-filter-badge__link {
  color: inherit;
  text-decoration: none;
  font-weight: 600;
}

.logs-filter-badge__link:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}

.logs-filter-badge__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 0.25rem;
  padding: 0;
  border: none;
  background: none;
  color: inherit;
  cursor: pointer;
  border-radius: 50%;
  opacity: 0.7;
  transition: opacity 0.15s ease;
}

.logs-filter-badge__remove:hover {
  opacity: 1;
}

.logs-filter-badge__remove:focus-visible {
  outline: 2px solid currentColor;
  outline-offset: 2px;
}
</style>
