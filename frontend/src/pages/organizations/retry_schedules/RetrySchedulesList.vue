<script setup lang="ts">
import { computed, h, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Clock, Pencil, Plus, Trash2 } from 'lucide-vue-next';
import { toast } from 'vue-sonner';

import { useRouteIds } from '@/composables/useRouteIds';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { handleMutationError } from '@/utils/handleMutationError';
import { routes } from '@/routes';

import { useDeleteRetrySchedule, useRetryScheduleList } from './useRetryScheduleQueries';
import { computeDelays, formatDelay } from './retryScheduleFormatters';
import type { RetrySchedule, RetryScheduleLimits } from './retrySchedule.types';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';

const { t } = useI18n();
const router = useRouter();
const { organizationId } = useRouteIds();

const { data: schedules, isLoading, error, refetch } = useRetryScheduleList(organizationId);

const instanceQuery = useInstanceConfig();
const limits = computed<RetryScheduleLimits | null>(() => {
  const cfg = instanceQuery.data.value as unknown as
    | { retry_schedule?: RetryScheduleLimits }
    | undefined;
  return cfg?.retry_schedule ?? null;
});

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

function renderIntervals(schedule: RetrySchedule) {
  if (!limits.value) return h('span', { class: 'muted' }, '—');
  const delays = computeDelays(schedule, limits.value);
  if (delays.length === 0) return h('span', { class: 'muted' }, '—');
  return h(
    'div',
    { class: 'interval-chips' },
    delays.slice(0, 8).map((s) => h('span', { class: 'interval-chips__chip' }, formatDelay(s)))
  );
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

function goToEdit(scheduleId: string, organization_id: string) {
  void router.push({
    name: routes.RetrySchedulesEdit,
    params: { organization_id, retry_schedule_id: scheduleId },
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
      <Hook0Button variant="primary" @click="goToNew">
        <Plus :size="14" aria-hidden="true" />
        {{ t('retrySchedules.new') }}
      </Hook0Button>
    </template>

    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <Hook0Card v-else-if="isLoading || !schedules">
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <Hook0Card v-else-if="schedules.length === 0">
      <Hook0CardContent>
        <Hook0EmptyState
          :title="t('retrySchedules.emptyTitle')"
          :description="t('retrySchedules.emptyDescription')"
          :icon="Clock"
        >
          <template #action>
            <Hook0Button variant="primary" @click="goToNew">
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
              <td><component :is="renderIntervals(schedule)" /></td>
              <td class="schedules-table__actions">
                <Hook0Button
                  variant="ghost"
                  type="button"
                  :aria-label="t('common.edit')"
                  @click="goToEdit(schedule.retry_schedule_id, organizationId)"
                >
                  <Pencil :size="14" aria-hidden="true" />
                </Hook0Button>
                <Hook0Button
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

    <Hook0Dialog
      :open="scheduleToDelete !== null"
      variant="danger"
      :title="t('retrySchedules.deleteTitle')"
      :confirm-text="t('common.delete')"
      @close="cancelDelete"
      @confirm="executeDelete"
    >
      <i18n-t keypath="retrySchedules.deleteConfirm" tag="p">
        <template #name>
          &ldquo;<strong>{{ scheduleToDelete?.name }}</strong
          >&rdquo;
        </template>
      </i18n-t>
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

.muted {
  color: var(--color-text-tertiary);
}

:deep(.interval-chips) {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

:deep(.interval-chips__chip) {
  display: inline-flex;
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-sm);
  background-color: var(--color-bg-secondary);
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  color: var(--color-text-secondary);
}
</style>
