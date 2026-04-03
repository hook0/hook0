<script setup lang="ts">
import { h, markRaw } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Plus, Repeat, Trash2 } from 'lucide-vue-next';

import { useRetryScheduleList, useRemoveRetrySchedule } from './useRetryScheduleQueries';
import type { RetrySchedule } from './RetryScheduleService';
import { routes } from '@/routes';
import { formatDelaySummary, strategyLabel } from './retryScheduleFormatters';
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
import Hook0DateTime from '@/components/Hook0DateTime.vue';
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
    accessorKey: 'strategy',
    header: t('retrySchedules.strategyColumn'),
    cell: (info) =>
      h(Hook0Badge, { variant: 'info', size: 'sm' }, () =>
        strategyLabel(info.getValue<string>(), t)
      ),
  },
  {
    accessorKey: 'max_retries',
    header: t('retrySchedules.maxRetriesColumn'),
  },
  {
    id: 'delay',
    header: t('retrySchedules.delayColumn'),
    cell: (info) => formatDelaySummary(info.row.original, t),
  },
  {
    accessorKey: 'created_at',
    header: t('retrySchedules.createdAtColumn'),
    enableSorting: true,
    cell: (info) => h(Hook0DateTime, { value: info.getValue<string>() }),
  },
  // Conditionally append the delete column — the spread-of-ternary keeps TanStack Table's column identity stable
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
    : []), // No actions column when user lacks delete permission — column simply doesn't exist
];
</script>

<template>
  <Hook0PageLayout :title="t('retrySchedules.title')">
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <Hook0Card v-else-if="isLoading || !schedules" data-test="retry-schedules-card">
      <Hook0CardHeader>
        <template #header>{{ t('retrySchedules.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="5" />
      </Hook0CardContent>
    </Hook0Card>

    <Hook0Card v-else-if="schedules" data-test="retry-schedules-card">
      <Hook0CardHeader>
        <template #header>{{ t('retrySchedules.title') }}</template>
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
