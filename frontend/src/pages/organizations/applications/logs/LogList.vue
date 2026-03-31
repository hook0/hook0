<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';

import { Send } from 'lucide-vue-next';
import { DOCS_LOGS_URL, API_DOCS_LOGS_URL } from '@/constants/externalLinks';

import { useLogList } from './useLogQueries';
import { useLogColumns } from './useLogColumns';
import type { RequestAttemptExtended } from './LogService';
import { routes } from '@/routes';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';

import LogSidePanel from './LogSidePanel.vue';
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
import Hook0HelpText from '@/components/Hook0HelpText.vue';

const { t } = useI18n();
const route = useRoute();
const { organizationId, applicationId } = useRouteIds();
const { data: requestAttempts, isLoading, error, refetch } = useLogList(applicationId);
const { data: organization } = useOrganizationDetail(organizationId);

const retentionDays = computed(
  () => organization.value?.quotas.days_of_events_retention_limit ?? 7
);

const columns = useLogColumns();

// Side panel state
const sidePanelOpen = ref(false);
const selectedRow = ref<RequestAttemptExtended | null>(null);

function handleRowClick(row: RequestAttemptExtended) {
  selectedRow.value = row;
  sidePanelOpen.value = true;
}

function closeSidePanel() {
  sidePanelOpen.value = false;
  selectedRow.value = null;
}
</script>

<template>
  <Hook0PageLayout :title="t('logs.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="void refetch()" />

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

        <Hook0CardContent v-if="requestAttempts.length > 0">
          <Hook0Table
            data-test="logs-table"
            :columns="columns"
            :data="requestAttempts"
            row-id-field="request_attempt_id"
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

    <!-- Log side panel -->
    <LogSidePanel
      v-if="selectedRow"
      :open="sidePanelOpen"
      :attempt="selectedRow"
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
  color: var(--color-text-tertiary);
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
</style>
