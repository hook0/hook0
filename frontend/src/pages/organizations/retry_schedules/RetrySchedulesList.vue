<script setup lang="ts">
import { h, markRaw } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Plus, Repeat, Trash2 } from 'lucide-vue-next';

import { useRetryScheduleList, useRemoveRetrySchedule } from './useRetryScheduleQueries';
import type { RetrySchedule } from './RetryScheduleService';
import { routes } from '@/routes';
import { strategyLabel } from './retryScheduleFormatters';
import { formatDuration } from '@/utils/formatDuration';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityDelete } from '@/composables/useEntityDelete';
import { useRouteIds } from '@/composables/useRouteIds';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';

// Retry schedule list page — shows all schedules for an organization with create/delete actions.
//
// How it works:
// 1. Fetches schedules via TanStack Query, renders table with name/strategy/delay/date columns
// 2. Delete uses useEntityDelete composable for confirm-then-mutate-then-toast pattern
// 3. Permissions gate create/delete visibility

const { t } = useI18n();
const { trackEvent } = useTracking();
const { canCreate, canDelete } = usePermissions();

const { organizationId } = useRouteIds();
const { data: schedules, isLoading, error, refetch } = useRetryScheduleList(organizationId);

const removeMutation = useRemoveRetrySchedule();

// useEntityDelete encapsulates the confirm-then-mutate-then-toast pattern — handles dialog state, async deletion, and feedback
const {
  showDeleteDialog,
  entityToDelete: scheduleToDelete,
  requestDelete,
  confirmDelete,
} = useEntityDelete<RetrySchedule>({
  deleteFn: (row) =>
    removeMutation.mutateAsync({
      retryScheduleId: row.retry_schedule_id,
      organizationId: organizationId.value,
    }),
  successTitle: t('common.success'),
  successMessage: t('retrySchedules.deleted'),
  onSuccess: () => trackEvent('retry-schedule', 'delete', 'success'),
});

function computeDelays(schedule: RetrySchedule): number[] {
  const max = schedule.max_retries;
  switch (schedule.strategy) {
    case 'increasing': {
      const bd = schedule.increasing_base_delay ?? 3;
      const wf = schedule.increasing_wait_factor ?? 3;
      return Array.from({ length: max }, (_, i) => Math.round(bd * Math.pow(wf, i)));
    }
    case 'linear':
      return Array.from({ length: max }, () => schedule.linear_delay ?? 60);
    case 'custom':
      return schedule.custom_intervals ?? [];
    default:
      return [];
  }
}

const columns: ColumnDef<RetrySchedule, unknown>[] = [
  {
    accessorKey: 'name',
    header: t('retrySchedules.nameColumn'),
    enableSorting: true,
    cell: (info) => {
      const row = info.row.original;
      return h(Hook0TableCellLink, {
        value: row.name,
        to: {
          name: routes.RetrySchedulesEdit,
          params: {
            organization_id: organizationId.value,
            retry_schedule_id: row.retry_schedule_id,
          },
        },
      });
    },
  },
  {
    id: 'preview',
    header: t('retrySchedules.previewColumn'),
    cell: (info) => {
      const row = info.row.original;
      const delays = computeDelays(row);
      return h('div', { class: 'preview-chips' }, [
        h(Hook0Badge, { variant: 'info', size: 'sm' }, () => strategyLabel(row.strategy, t)),
        ...delays.map((s) => h('span', { class: 'preview-chips__chip' }, formatDuration(s))),
      ]);
    },
  },
  ...(canDelete('retry_schedule')
    ? [
        {
          id: 'actions',
          header: t('common.actions'),
          cell: (info: { row: { original: RetrySchedule } }) =>
            h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              variant: 'danger',
              onClick: () => requestDelete(info.row.original),
            }),
        } satisfies ColumnDef<RetrySchedule, unknown>,
      ]
    : []),
];
</script>

<template>
  <Hook0PageLayout :title="t('retrySchedules.title')">
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <Hook0Card v-else-if="isLoading || !schedules" data-test="retry-schedules-card">
      <Hook0CardHeader>
        <template #header>{{ t('retrySchedules.title') }}</template>
        <template #subtitle>{{ t('retrySchedules.subtitle') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="5" />
      </Hook0CardContent>
    </Hook0Card>

    <Hook0Card v-else data-test="retry-schedules-card">
      <Hook0CardHeader>
        <template #header>{{ t('retrySchedules.title') }}</template>
        <template #subtitle>{{ t('retrySchedules.subtitle') }}</template>
      </Hook0CardHeader>

      <Hook0CardContent v-if="schedules.length > 0">
        <Hook0Table
          data-test="retry-schedules-table"
          :columns="columns"
          :data="schedules"
          row-id-field="retry_schedule_id"
        />
      </Hook0CardContent>

      <Hook0CardContent v-else>
        <Hook0EmptyState
          :title="t('retrySchedules.empty.title')"
          :description="t('retrySchedules.empty.description')"
          :icon="Repeat"
        >
          <template v-if="canCreate('retry_schedule')" #action>
            <Hook0Button
              variant="primary"
              :to="{
                name: routes.RetrySchedulesNew,
                params: { organization_id: organizationId },
              }"
            >
              <template #left>
                <Plus :size="16" aria-hidden="true" />
              </template>
              {{ t('retrySchedules.empty.cta') }}
            </Hook0Button>
          </template>
        </Hook0EmptyState>
      </Hook0CardContent>

      <Hook0CardFooter v-if="schedules.length > 0 && canCreate('retry_schedule')">
        <Hook0Button
          variant="primary"
          :to="{
            name: routes.RetrySchedulesNew,
            params: { organization_id: organizationId },
          }"
        >
          <template #left>
            <Plus :size="16" aria-hidden="true" />
          </template>
          {{ t('retrySchedules.create') }}
        </Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('retrySchedules.delete')"
      @close="
        showDeleteDialog = false;
        scheduleToDelete = null;
      "
      @confirm="confirmDelete()"
    >
      <p>{{ t('retrySchedules.deleteConfirm') }}</p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
/* :deep() required because these elements are rendered via h() inside Hook0Table's cell slots */
:deep(.preview-chips) {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  align-items: center;
}

:deep(.preview-chips__chip) {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  font-variant-numeric: tabular-nums;
}
</style>
