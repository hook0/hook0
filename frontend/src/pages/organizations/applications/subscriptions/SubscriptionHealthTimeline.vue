<script setup lang="ts">
import { h, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import type { HealthEventPage } from './SubscriptionHealthService';
import { useHealthThresholds } from '@/composables/useHealthThresholds';

import Hook0Table from '@/components/Hook0Table.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0DateFormatted from '@/components/Hook0DateFormatted.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';

const props = defineProps<{
  page: HealthEventPage | undefined;
  isLoading: boolean;
  error: Error | string | null;
  refetch: () => void;
}>();

const emit = defineEmits<{
  navigate: [cursor: string | null, direction: 'forward' | 'backward'];
}>();

const { t } = useI18n();
const { warning: warningThreshold, critical: criticalThreshold } = useHealthThresholds();

type BadgeVariant = 'default' | 'success' | 'warning' | 'danger';

const statusVariantMap: Record<string, BadgeVariant> = {
  resolved: 'success',
  warning: 'warning',
  disabled: 'danger',
};

function reasonText(status: string): string {
  const threshold = status === 'disabled' ? criticalThreshold.value : warningThreshold.value;
  return t(`subscriptionDetail.healthReason.${status}`, { threshold });
}

const columns = computed<ColumnDef<HealthEventPage['data'][number], unknown>[]>(() => [
  {
    accessorKey: 'status',
    header: t('common.status'),
    cell: (info) => {
      const event = info.row.original;
      return h(
        Hook0Badge,
        { variant: statusVariantMap[event.status] ?? 'default', size: 'sm' },
        () => t(`subscriptionDetail.healthStatus.${event.status}`)
      );
    },
  },
  {
    accessorKey: 'cause',
    header: t('subscriptionDetail.healthEventCause'),
    cell: (info) =>
      h(Hook0Badge, { variant: 'default', size: 'sm' }, () =>
        t(`subscriptionDetail.healthCause.${info.row.original.cause}`)
      ),
  },
  {
    id: 'reason',
    header: t('subscriptionDetail.healthEventReason'),
    cell: (info) => {
      const event = info.row.original;
      if (event.cause === 'manual') return h('span', { class: 'health-timeline__dash' }, '—');
      return h('span', { class: 'health-timeline__reason' }, reasonText(event.status));
    },
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    cell: (info) => h(Hook0DateFormatted, { value: info.getValue() as string | null }),
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
