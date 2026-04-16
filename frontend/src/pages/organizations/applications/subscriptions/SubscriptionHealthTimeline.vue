<script setup lang="ts">
import { h, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import type { CursorPage, PaginationDirection } from '@/utils/pagination';
import type { HealthEvent } from './SubscriptionHealthService';
import type { Problem } from '@/http';
import { useHealthThresholds } from '@/composables/useHealthThresholds';

import Hook0Table from '@/components/Hook0Table.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0DateFormatted from '@/components/Hook0DateFormatted.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';

const props = defineProps<{
  page: CursorPage<HealthEvent> | undefined;
  isLoading: boolean;
  error: Problem | Error | string | null;
  refetch: () => void;
}>();

const emit = defineEmits<{
  navigate: [cursor: string | null, direction: PaginationDirection];
}>();

const { t } = useI18n();
const { warning: warningThreshold, critical: criticalThreshold } = useHealthThresholds();

// --- Formatting helpers ---
// Extracted so column definitions stay readable.

function getStatusVariant(status: string) {
  switch (status) {
    case 'resolved':
      return 'success';
    case 'warning':
      return 'warning';
    case 'disabled':
      return 'danger';
    default:
      return 'default';
  }
}

function getReasonText(status: string): string {
  const threshold = status === 'disabled' ? criticalThreshold.value : warningThreshold.value;
  return t(`subscriptionDetail.healthReason.${status}`, { threshold });
}

// --- Table columns ---

const columns = computed<ColumnDef<HealthEvent, unknown>[]>(() => [
  {
    accessorKey: 'status',
    header: t('common.status'),
    cell: (info) => {
      const event = info.row.original;
      const label = t(`subscriptionDetail.healthStatus.${event.status}`);

      return h(Hook0Badge, { variant: getStatusVariant(event.status), size: 'sm' }, () => label);
    },
  },
  {
    accessorKey: 'cause',
    header: t('subscriptionDetail.healthEventCause'),
    cell: (info) => {
      const event = info.row.original;
      const label = t(`subscriptionDetail.healthCause.${event.cause}`);

      return h(Hook0Badge, { variant: 'default', size: 'sm' }, () => label);
    },
  },
  {
    id: 'reason',
    header: t('subscriptionDetail.healthEventReason'),
    cell: (info) => {
      const event = info.row.original;

      if (event.cause === 'manual') {
        return h('span', { class: 'health-timeline__dash' }, '—');
      }

      return h('span', { class: 'health-timeline__reason' }, getReasonText(event.status));
    },
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    cell: (info) => {
      return h(Hook0DateFormatted, { value: info.getValue() as string | null });
    },
  },
]);

function goNext() {
  if (props.page?.nextCursor) {
    emit('navigate', props.page.nextCursor, 'forward');
  }
}

function goPrev() {
  if (props.page?.prevCursor) {
    emit('navigate', props.page.prevCursor, 'backward');
  }
}
</script>

<template>
  <div class="health-timeline">
    <Hook0SkeletonGroup v-if="isLoading || !page" :count="3" />
    <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />
    <template v-else-if="page">
      <Hook0EmptyState
        v-if="page.data.length === 0"
        :title="t('subscriptionDetail.healthTimelineEmpty')"
        :description="t('subscriptionDetail.healthTimelineEmptyDescription')"
      />
      <template v-else>
        <Hook0Table :columns="columns" :data="page.data" row-id-field="health_event_id" />
        <div class="health-timeline__pagination">
          <Hook0Button variant="secondary" size="sm" :disabled="!page.prevCursor" @click="goPrev">
            {{ t('common.previousPage') }}
          </Hook0Button>
          <Hook0Button variant="secondary" size="sm" :disabled="!page.nextCursor" @click="goNext">
            {{ t('common.nextPage') }}
          </Hook0Button>
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.health-timeline__pagination {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding-top: 0.75rem;
}

.health-timeline__reason {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.health-timeline__dash {
  color: var(--color-text-tertiary);
}
</style>
