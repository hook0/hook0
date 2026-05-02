<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { useMediaQuery } from '@vueuse/core';

import { ArrowLeft, Send } from 'lucide-vue-next';
import { DOCS_LOGS_URL, API_DOCS_LOGS_URL } from '@/constants/externalLinks';

import { useLogListInfinite } from './useLogQueries';
import { useLogColumns } from './useLogColumns';
import type { RequestAttemptExtended } from './LogService';
import { routes } from '@/routes';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';
import { usePermissions } from '@/composables/usePermissions';

import LogDetailContent from './LogDetailContent.vue';
import Hook0DocButtons from '@/components/Hook0DocButtons.vue';

import Hook0SplitLayout from '@/components/Hook0SplitLayout.vue';
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
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0PaginatedList from '@/components/Hook0PaginatedList.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { organizationId, applicationId } = useRouteIds();
const logsQuery = useLogListInfinite(applicationId);
const {
  isLoading,
  error,
  refetch,
  totalPagesSeen,
  currentPageItems: requestAttempts,
  currentPageIdx,
} = logsQuery;
const hasAnyAttempts = computed(
  () => totalPagesSeen.value > 0 && (requestAttempts.value.length > 0 || currentPageIdx.value > 0)
);
const { data: organization } = useOrganizationDetail(organizationId);
const { canCreate } = usePermissions();

const isDesktop = useMediaQuery('(min-width: 768px)');

const retentionDays = computed(() => {
  const days = organization.value?.quotas.days_of_events_retention_limit;
  // API uses INT32_MAX (2147483647) as sentinel for "unlimited retention"
  if (!days || days >= 2_147_483_647) return null;
  return days;
});

const columns = useLogColumns();

const selectedRow = ref<RequestAttemptExtended | null>(null);

// Sync selection from route query; desktop auto-selects first item, mobile does not
watch(
  [requestAttempts, () => route.query.delivery, isDesktop],
  ([attempts, deliveryId]) => {
    if (!attempts?.length) {
      selectedRow.value = null;
      return;
    }
    if (deliveryId) {
      const found = attempts.find((a) => a.request_attempt_id === deliveryId);
      if (found) {
        selectedRow.value = found;
        return;
      }
    }
    // Auto-select first row on desktop when no valid selection exists
    if (isDesktop.value) {
      selectedRow.value = attempts[0];
      void router.replace({
        query: { ...route.query, delivery: attempts[0].request_attempt_id },
      });
    } else {
      selectedRow.value = null;
    }
  },
  { immediate: true }
);

function handleRowClick(row: RequestAttemptExtended) {
  selectedRow.value = row;
  void router.replace({
    query: { ...route.query, delivery: row.request_attempt_id },
  });
}

function goBackToList() {
  selectedRow.value = null;
  const { delivery: _, ...rest } = route.query;
  void router.replace({ query: rest });
}
</script>

<template>
  <Hook0PageLayout :title="t('logs.title')">
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="void refetch()" />

    <Hook0Card v-else-if="isLoading && totalPagesSeen === 0" data-test="logs-card">
      <Hook0CardHeader>
        <template #header>{{ t('logs.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <template v-else>
      <Hook0Card data-test="logs-card">
        <Hook0CardHeader>
          <template #header>{{ t('logs.title') }}</template>
          <template #subtitle>
            {{ t('logs.subtitle') }}
            <Hook0HelpText v-if="retentionDays" tone="emphasis">{{
              t('logs.subtitleRetention', { days: retentionDays })
            }}</Hook0HelpText>
          </template>
          <template #actions>
            <Hook0DocButtons :doc-url="DOCS_LOGS_URL" :api-url="API_DOCS_LOGS_URL" />
          </template>
        </Hook0CardHeader>
      </Hook0Card>

      <Hook0SplitLayout
        v-if="hasAnyAttempts"
        :show-detail="!!selectedRow"
        :detail-key="selectedRow?.request_attempt_id"
      >
        <template #back>
          <Hook0Button variant="ghost" size="sm" @click="goBackToList">
            <ArrowLeft :size="16" aria-hidden="true" />
            {{ t('logs.backToList') }}
          </Hook0Button>
        </template>
        <template #list>
          <Hook0PaginatedList :query="logsQuery">
            <template #default="{ items }">
              <Hook0Table
                data-test="logs-table"
                :columns="columns"
                :data="items"
                row-id-field="request_attempt_id"
                clickable-rows
                :active-row-id="selectedRow?.request_attempt_id"
                @row-click="handleRowClick"
              />
            </template>
          </Hook0PaginatedList>
        </template>
        <template #detail>
          <LogDetailContent
            v-if="selectedRow"
            :attempt="selectedRow"
            :application-id="applicationId"
          />
          <Hook0Skeleton v-else size="block" />
        </template>
      </Hook0SplitLayout>

      <Hook0Card v-else>
        <Hook0CardContent>
          <Hook0EmptyState
            :title="t('logs.empty.title')"
            :description="t('logs.empty.description')"
            :icon="Send"
          >
            <template v-if="canCreate('subscription')" #action>
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
  </Hook0PageLayout>
</template>

<style scoped>
/* Align doc buttons to top when subtitle wraps */
:deep(.hook0-card-header__container) {
  align-items: flex-start;
}

/* Table layout overrides for Hook0SplitLayout's list panel */
:deep(.hook0-split-layout__list table) {
  table-layout: fixed;
}

/* Status column: fixed width, never truncate */
:deep(.hook0-split-layout__list .hook0-table-th:first-child),
:deep(.hook0-split-layout__list .hook0-table-td:first-child) {
  width: 12rem;
  white-space: nowrap;
  overflow: visible;
}

@media (max-width: 767px) {
  :deep(.hook0-split-layout__list .hook0-table-th:first-child),
  :deep(.hook0-split-layout__list .hook0-table-td:first-child) {
    width: 7rem;
  }

  :deep(.hook0-split-layout__list .log-status) {
    max-width: 6rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

/* Event + date columns truncate with ellipsis */
:deep(.hook0-split-layout__list .hook0-table-td:not(:first-child)),
:deep(.hook0-split-layout__list .hook0-table-th:not(:first-child)) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Right-align the last column (created_at) — header + cells */
:deep(.hook0-table-th:last-child) {
  text-align: right;
}

:deep(.hook0-table-th:last-child .hook0-table-sort-button) {
  justify-content: flex-end;
  width: 100%;
}

:deep(.hook0-table-td:last-child) {
  text-align: right;
}

/* Column cell styles rendered via h() in useLogColumns.ts */

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

:deep(.log-status__icon) {
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

:deep(.log-cell-link.hook0-button.link) {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

:deep(.log-event-name) {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}
</style>
