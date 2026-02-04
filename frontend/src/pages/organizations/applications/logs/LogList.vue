<script setup lang="ts">
import { computed, h } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import { useLogList } from './useLogQueries';
import type { RequestAttemptTypeFixed } from './LogService';
import { RequestAttemptStatusType } from './LogService';
import type { RequestAttempt } from './LogService';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0TableCellIcon from '@/components/Hook0TableCellIcon.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';

const { t } = useI18n();
const route = useRoute();

const applicationId = computed(() => route.params.application_id as string);
const { data: requestAttempts, isLoading, error, refetch } = useLogList(applicationId);

function statusTitle(row: RequestAttemptTypeFixed): string {
  switch (row.status.type) {
    case RequestAttemptStatusType.Waiting:
      return t('logs.statusWaiting');
    case RequestAttemptStatusType.Pending:
      return t('logs.statusPending');
    case RequestAttemptStatusType.InProgress:
      return t('logs.statusInProgress');
    case RequestAttemptStatusType.Failed:
      return t('logs.statusFailed');
    case RequestAttemptStatusType.Successful:
      return t('logs.statusSuccessful');
    default:
      return t('logs.statusUnknown');
  }
}

function statusIcon(row: RequestAttemptTypeFixed): string {
  switch (row.status.type) {
    case RequestAttemptStatusType.Waiting:
      return 'calendar';
    case RequestAttemptStatusType.Pending:
      return 'pause';
    case RequestAttemptStatusType.InProgress:
      return 'spinner';
    case RequestAttemptStatusType.Failed:
      return 'times';
    case RequestAttemptStatusType.Successful:
      return 'check';
    default:
      return 'question-circle';
  }
}

const columns: ColumnDef<RequestAttemptTypeFixed, unknown>[] = [
  {
    accessorKey: 'status',
    header: t('common.status'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0TableCellIcon, {
        title: statusTitle(info.row.original),
        icon: statusIcon(info.row.original),
      }),
  },
  {
    accessorKey: 'event_id',
    header: t('logs.eventId'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: String(info.getValue()),
        to: {
          name: routes.EventsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            event_id: (info.row.original as unknown as RequestAttempt).event_id,
          },
        },
      }),
  },
  {
    id: 'subscription',
    header: t('logs.subscription'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: String(info.row.original.subscription.description ?? ''),
        to: {
          name: routes.SubscriptionsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            subscription_id: info.row.original.subscription.subscription_id,
          },
        },
      }),
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue() as string | null }),
  },
  {
    accessorKey: 'picked_at',
    header: t('logs.pickedAt'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0TableCellDate, {
        value: info.getValue() as string | null,
        defaultText: t('logs.pendingPlaceholder'),
      }),
  },
];
</script>

<template>
  <Hook0PageLayout :title="t('logs.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !requestAttempts" data-test="logs-card">
      <Hook0CardHeader>
        <template #header>{{ t('logs.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (requestAttempts is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="logs-card">
        <Hook0CardHeader>
          <template #header>{{ t('logs.title') }}</template>
          <template #subtitle>
            {{ t('logs.subtitle') }}
            <Hook0HelpText tone="emphasis">{{ t('logs.subtitleRetention') }}</Hook0HelpText>
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="requestAttempts.length > 0">
          <Hook0Table
            data-test="logs-table"
            :columns="columns"
            :data="requestAttempts"
            row-id-field="event_id"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('logs.empty.title')"
            :description="t('logs.empty.description')"
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

        <Hook0CardFooter />
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped></style>
