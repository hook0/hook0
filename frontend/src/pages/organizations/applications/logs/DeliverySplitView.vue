<script setup lang="ts">
/**
 * Reusable delivery list with split layout — table on the left, detail on the right.
 *
 * Used by both the main Deliveries page (LogList) and the Subscription Detail page.
 * Handles: column setup, row selection, desktop auto-select, mobile back button,
 * and all the CSS for log status pills and split layout table overrides.
 */
import { ref, watch, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useMediaQuery } from '@vueuse/core';
import type { ColumnDef } from '@tanstack/vue-table';
import { ArrowLeft } from 'lucide-vue-next';

import { useLogColumns } from './useLogColumns';
import LogDetailContent from './LogDetailContent.vue';
import type { RequestAttemptExtended } from './LogService';

import Hook0SplitLayout from '@/components/Hook0SplitLayout.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

const props = defineProps<{
  /** The delivery rows to display */
  deliveries: RequestAttemptExtended[];
  /** Application ID — passed to LogDetailContent for fetching response details */
  applicationId: string;
  /** Extra columns appended after the default log columns (e.g. retry button) */
  extraColumns?: ColumnDef<RequestAttemptExtended, unknown>[];
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const isDesktop = useMediaQuery('(min-width: 768px)');

const baseColumns = useLogColumns();
const columns = computed(() => [...baseColumns, ...(props.extraColumns ?? [])]);

const selectedRow = ref<RequestAttemptExtended | null>(null);

// Sync selection with ?delivery= query param — survives page refresh.
// Auto-select first row on desktop when no valid selection exists.
watch(
  [() => props.deliveries, () => route.query.delivery, isDesktop],
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
  <Hook0SplitLayout :show-detail="!!selectedRow" :detail-key="selectedRow?.request_attempt_id">
    <template #back>
      <Hook0Button variant="ghost" size="sm" @click="goBackToList">
        <ArrowLeft :size="16" aria-hidden="true" />
        {{ t('logs.backToList') }}
      </Hook0Button>
    </template>
    <template #list>
      <Hook0Table
        :columns="columns"
        :data="deliveries"
        row-id-field="request_attempt_id"
        clickable-rows
        :active-row-id="selectedRow?.request_attempt_id"
        @row-click="handleRowClick"
      />
    </template>
    <template #detail>
      <LogDetailContent v-if="selectedRow" :attempt="selectedRow" :application-id="applicationId" />
      <Hook0Skeleton v-else size="block" />
    </template>
  </Hook0SplitLayout>
</template>

<style scoped>
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

/* Right-align the last visible date column — header + cells */
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

/* Log cell styles — rendered via h() in useLogColumns.ts, needs :deep() to pierce scoped boundary */

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

:deep(.log-status-cell) {
  display: flex;
  align-items: center;
  gap: 0.375rem;
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
