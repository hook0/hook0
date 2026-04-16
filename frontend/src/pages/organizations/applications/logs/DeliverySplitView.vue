<script setup lang="ts">
/**
 * Reusable delivery list with split layout — table on the left, detail on the right.
 *
 * Used by both the main Deliveries page (LogList) and the Subscription Detail page.
 * Handles: column setup, row selection, desktop auto-select, mobile back button,
 * and all the CSS for log status pills and split layout table overrides.
 */
import { h, ref, watch, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useMediaQuery } from '@vueuse/core';
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
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const isDesktop = useMediaQuery('(min-width: 768px)');

const columns = computed(() => useLogColumns());

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

  const nextQuery = { ...route.query };
  delete nextQuery.delivery;

  void router.replace({ query: nextQuery });
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
@import './log-cells.css';

:deep(.hook0-split-layout__list table) {
  table-layout: auto;
  width: 100%;
}

:deep(.log-col-event) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:deep(.log-col-event .log-cell-link) {
  display: inline;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
