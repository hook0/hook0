<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Clock, Pencil, Plus, Trash2 } from 'lucide-vue-next';
import { toast } from 'vue-sonner';

import { useRouteIds } from '@/composables/useRouteIds';
import { usePermissions } from '@/composables/usePermissions';
import { handleMutationError } from '@/utils/handleMutationError';
import { routes } from '@/routes';

import { useDeleteRetrySchedule, useRetryScheduleList } from './useRetryScheduleQueries';
import { useRetryScheduleLimits } from './useRetryScheduleLimits';
import { computeDelays } from './retryScheduleFormatters';
import type { RetrySchedule } from './retrySchedule.types';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0IntervalChips from './Hook0IntervalChips.vue';

const { t } = useI18n();
const router = useRouter();
const { organizationId } = useRouteIds();
const perms = usePermissions();

const { data: schedules, isLoading, error, refetch } = useRetryScheduleList(organizationId);
const { limits } = useRetryScheduleLimits();

function strategyLabel(strategy: RetrySchedule['strategy']): string {
  switch (strategy) {
    case 'exponential_increasing':
      return t('retrySchedules.strategies.exponentialIncreasing');
    case 'linear':
      return t('retrySchedules.strategies.linear');
    case 'custom':
      return t('retrySchedules.strategies.custom');
  }
}

function intervalsFor(schedule: RetrySchedule): number[] {
  if (!limits.value) return [];
  return computeDelays(schedule, limits.value);
}

// Delete confirmation
const scheduleToDelete = ref<RetrySchedule | null>(null);
const deleteMutation = useDeleteRetrySchedule();

function confirmDelete(schedule: RetrySchedule) {
  scheduleToDelete.value = schedule;
}

function cancelDelete() {
  scheduleToDelete.value = null;
}

function executeDelete() {
  const schedule = scheduleToDelete.value;
  if (!schedule) return;
  deleteMutation.mutate(schedule.retry_schedule_id, {
    onSuccess: () => {
      toast.success(t('common.success'), {
        description: t('retrySchedules.deleted', { name: schedule.name }),
      });
      scheduleToDelete.value = null;
    },
    onError: (err) => handleMutationError(err),
  });
}

function goToEdit(scheduleId: string) {
  void router.push({
    name: routes.RetrySchedulesEdit,
    params: { organization_id: organizationId.value, retry_schedule_id: scheduleId },
  });
}

function goToNew() {
  void router.push({
    name: routes.RetrySchedulesNew,
    params: { organization_id: organizationId.value },
  });
}
</script>

<template>
  <Hook0PageLayout :title="t('retrySchedules.title')">
    <template #actions>
      <Hook0Button v-if="perms.canCreate('retry_schedule')" variant="primary" @click="goToNew">
        <Plus :size="14" aria-hidden="true" />
        {{ t('retrySchedules.new') }}
      </Hook0Button>
    </template>

    <!-- Loading/skeleton first (also covers disabled-query state per CLAUDE.md). -->
    <Hook0Card v-if="isLoading || !schedules">
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />

    <template v-else-if="schedules">
      <Hook0Card v-if="schedules.length === 0">
        <Hook0CardContent>
          <Hook0EmptyState
            :title="t('retrySchedules.emptyTitle')"
            :description="t('retrySchedules.emptyDescription')"
            :icon="Clock"
          >
            <template #action>
              <Hook0Button
                v-if="perms.canCreate('retry_schedule')"
                variant="primary"
                @click="goToNew"
              >
                {{ t('retrySchedules.createFirst') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>
      </Hook0Card>

      <Hook0Card v-else>
        <Hook0CardHeader>
          <template #header>{{ t('retrySchedules.listHeader') }}</template>
          <template #subtitle>{{ t('retrySchedules.listSubtitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <table class="schedules-table">
            <thead>
              <tr>
                <th>{{ t('retrySchedules.columns.name') }}</th>
                <th>{{ t('retrySchedules.columns.strategy') }}</th>
                <th>{{ t('retrySchedules.columns.maxRetries') }}</th>
                <th>{{ t('retrySchedules.columns.intervals') }}</th>
                <th class="schedules-table__actions-col">{{ t('common.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="schedule in schedules"
                :key="schedule.retry_schedule_id"
                class="schedules-table__row"
              >
                <td>{{ schedule.name }}</td>
                <td>{{ strategyLabel(schedule.strategy) }}</td>
                <td>{{ schedule.max_retries }}</td>
                <td>
                  <Hook0IntervalChips :values="intervalsFor(schedule)" :max="8" />
                </td>
                <td class="schedules-table__actions">
                  <Hook0Button
                    v-if="perms.canEdit('retry_schedule')"
                    variant="ghost"
                    type="button"
                    :aria-label="t('common.edit')"
                    @click="goToEdit(schedule.retry_schedule_id)"
                  >
                    <Pencil :size="14" aria-hidden="true" />
                  </Hook0Button>
                  <Hook0Button
                    v-if="perms.canDelete('retry_schedule')"
                    variant="ghost"
                    type="button"
                    :aria-label="t('common.delete')"
                    @click="confirmDelete(schedule)"
                  >
                    <Trash2 :size="14" aria-hidden="true" />
                  </Hook0Button>
                </td>
              </tr>
            </tbody>
          </table>
        </Hook0CardContent>
      </Hook0Card>
    </template>

    <Hook0Dialog
      :open="scheduleToDelete !== null"
      variant="danger"
      :title="t('retrySchedules.deleteTitle')"
      :confirm-text="t('common.delete')"
      @close="cancelDelete"
      @confirm="executeDelete"
    >
      <p>{{ t('retrySchedules.deleteConfirm', { name: scheduleToDelete?.name ?? '' }) }}</p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
.schedules-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8125rem;
}

.schedules-table thead th {
  text-align: left;
  padding: 0.5rem 0.75rem;
  font-size: 0.6875rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--color-text-tertiary);
  border-bottom: 1px solid var(--color-border);
}

.schedules-table tbody td {
  padding: 0.625rem 0.75rem;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text-primary);
  vertical-align: middle;
}

.schedules-table__row:hover td {
  background-color: var(--color-bg-secondary);
}

.schedules-table__actions-col {
  width: 1%;
  text-align: right;
}

.schedules-table__actions {
  display: flex;
  gap: 0.25rem;
  justify-content: flex-end;
}
</style>
