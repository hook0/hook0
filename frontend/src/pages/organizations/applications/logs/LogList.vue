<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { useMediaQuery } from '@vueuse/core';

import { ArrowLeft, Send } from 'lucide-vue-next';
import { DOCS_LOGS_URL, API_DOCS_LOGS_URL } from '@/constants/externalLinks';

import { useLogList } from './useLogQueries';
import { useLogColumns } from './useLogColumns';
import type { RequestAttemptExtended } from './LogService';
import { routes } from '@/routes';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';

import LogDetailContent from './LogDetailContent.vue';
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
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { organizationId, applicationId } = useRouteIds();
const { data: requestAttempts, isLoading, error, refetch } = useLogList(applicationId);
const { data: organization } = useOrganizationDetail(organizationId);

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
      selectedRow.value = found ?? null;
    } else if (isDesktop.value && !selectedRow.value) {
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

    <Hook0Card v-else-if="isLoading || !requestAttempts" data-test="logs-card">
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

      <div v-if="requestAttempts.length > 0" class="log-split">
        <!-- Mobile: detail full-width with back button -->
        <template v-if="!isDesktop && selectedRow">
          <div class="log-split__detail log-split__detail--mobile">
            <Hook0Button variant="ghost" size="sm" class="log-split__back" @click="goBackToList">
              <ArrowLeft :size="16" aria-hidden="true" />
              {{ t('logs.backToList') }}
            </Hook0Button>
            <div class="log-split__detail-content">
              <LogDetailContent :attempt="selectedRow" :application-id="applicationId" />
            </div>
          </div>
        </template>

        <!-- Mobile: list (no detail selected) -->
        <template v-else-if="!isDesktop">
          <div class="log-split__list">
            <Hook0Table
              data-test="logs-table"
              :columns="columns"
              :data="requestAttempts"
              row-id-field="request_attempt_id"
              clickable-rows
              @row-click="handleRowClick"
            />
          </div>
        </template>

        <!-- Desktop: split layout -->
        <template v-else>
          <div class="log-split__list">
            <Hook0Table
              data-test="logs-table"
              :columns="columns"
              :data="requestAttempts"
              row-id-field="request_attempt_id"
              clickable-rows
              :active-row-id="selectedRow?.request_attempt_id"
              @row-click="handleRowClick"
            />
          </div>

          <div class="log-split__detail">
            <Transition name="log-detail-fade" mode="out-in">
              <div v-if="selectedRow" :key="selectedRow.request_attempt_id">
                <LogDetailContent :attempt="selectedRow" :application-id="applicationId" />
              </div>
              <Hook0Skeleton v-else size="block" />
            </Transition>
          </div>
        </template>
      </div>

      <Hook0Card v-else>
        <Hook0CardContent>
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
  </Hook0PageLayout>
</template>

<style scoped>
/* Align doc buttons to top when subtitle wraps */
:deep(.hook0-card-header__container) {
  align-items: flex-start;
}

/* Split layout — table left, detail right on desktop */
.log-split {
  margin-top: 1rem;
}

@media (min-width: 768px) {
  .log-split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .log-split__list,
  .log-split__detail {
    height: calc(100vh - 23rem);
  }
}

.log-split__list {
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  min-width: 0;
}

.log-split__list :deep(table) {
  table-layout: fixed;
}

/* Status column: fixed width, never truncate */
.log-split__list :deep(.hook0-table-th:first-child),
.log-split__list :deep(.hook0-table-td:first-child) {
  width: 12rem;
  white-space: nowrap;
  overflow: visible;
}

@media (max-width: 767px) {
  .log-split__list :deep(.hook0-table-th:first-child),
  .log-split__list :deep(.hook0-table-td:first-child) {
    width: 7rem;
  }

  .log-split__list :deep(.log-status) {
    max-width: 6rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

/* Event + date columns truncate with ellipsis */
.log-split__list :deep(.hook0-table-td:not(:first-child)),
.log-split__list :deep(.hook0-table-th:not(:first-child)) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-split__detail {
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  padding: 1.25rem;
  isolation: isolate;
}

.log-split__detail--mobile {
  position: static;
  padding: 0;
}

.log-split__back {
  padding: 0.75rem 1.25rem;
  margin-top: 0.5rem;
  border-bottom: 1px solid var(--color-border);
}

.log-split__detail-content {
  padding: 1.25rem;
}

/* Detail panel fade out → fade in on delivery switch */
.log-detail-fade-enter-active {
  transition: opacity 200ms ease;
}

.log-detail-fade-leave-active {
  transition: opacity 100ms ease;
}

.log-detail-fade-enter-from,
.log-detail-fade-leave-to {
  opacity: 0;
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
