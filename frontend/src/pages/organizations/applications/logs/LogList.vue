<script setup lang="ts">
import { computed, h, resolveComponent } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import { useLogList } from './useLogQueries';
import type { RequestAttemptTypeFixed } from './LogService';
import { RequestAttemptStatusType } from './LogService';
import type { RequestAttempt } from './LogService';
import { routes } from '@/routes';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';

const { t } = useI18n();
const route = useRoute();

const organizationId = computed(() => route.params.organization_id as string);
const applicationId = computed(() => route.params.application_id as string);
const { data: requestAttempts, isLoading, error, refetch } = useLogList(applicationId);
const { data: organization } = useOrganizationDetail(organizationId);

const retentionDays = computed(
  () => organization.value?.quotas.days_of_events_retention_limit ?? 7
);

function statusVariant(
  row: RequestAttemptTypeFixed
): 'success' | 'error' | 'warning' | 'info' | 'muted' {
  switch (row.status.type) {
    case RequestAttemptStatusType.Successful:
      return 'success';
    case RequestAttemptStatusType.Failed:
      return 'error';
    case RequestAttemptStatusType.Pending:
    case RequestAttemptStatusType.InProgress:
      return 'warning';
    case RequestAttemptStatusType.Waiting:
      return 'info';
    default:
      return 'muted';
  }
}

function statusLabel(row: RequestAttemptTypeFixed): string {
  const httpCode = (row as unknown as Record<string, unknown>).http_response_status as
    | number
    | null
    | undefined;
  const shortTitle = (() => {
    switch (row.status.type) {
      case RequestAttemptStatusType.Successful:
        return 'Sent';
      case RequestAttemptStatusType.Failed:
        return 'Failed';
      case RequestAttemptStatusType.Pending:
        return 'Pending';
      case RequestAttemptStatusType.InProgress:
        return 'Sending';
      case RequestAttemptStatusType.Waiting:
        return 'Scheduled';
      default:
        return 'Unknown';
    }
  })();
  return httpCode ? `${httpCode} ${shortTitle}` : shortTitle;
}

const STATUS_COLORS: Record<string, { bg: string; fg: string }> = {
  success: { bg: 'var(--color-success-light)', fg: 'var(--color-success)' },
  error: { bg: 'var(--color-error-light)', fg: 'var(--color-error)' },
  warning: { bg: 'var(--color-warning-light)', fg: 'var(--color-warning)' },
  info: { bg: 'var(--color-info-light)', fg: 'var(--color-info)' },
  muted: { bg: 'var(--color-bg-tertiary)', fg: 'var(--color-text-muted)' },
};

function fmtDate(val: unknown): string {
  if (!val || typeof val !== 'string') return '—';
  try {
    return new Intl.DateTimeFormat('en', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    }).format(new Date(val));
  } catch {
    return String(val);
  }
}

function statusTooltip(row: RequestAttemptTypeFixed): string {
  const r = row as unknown as Record<string, unknown>;
  const retry = Number(r.retry_count ?? 0);
  const retryStr = retry > 0 ? t('logs.tooltipRetry', { count: retry }) : '';
  switch (row.status.type) {
    case RequestAttemptStatusType.Successful:
      return t('logs.tooltipSuccessful', { date: fmtDate(r.succeeded_at), retry: retryStr });
    case RequestAttemptStatusType.Failed:
      return t('logs.tooltipFailed', { date: fmtDate(r.failed_at), retry: retryStr });
    case RequestAttemptStatusType.Pending:
      return t('logs.tooltipPending', { date: fmtDate(r.created_at), retry: retryStr });
    case RequestAttemptStatusType.InProgress:
      return t('logs.tooltipInProgress', { date: fmtDate(r.picked_at), retry: retryStr });
    case RequestAttemptStatusType.Waiting:
      return t('logs.tooltipWaiting', { date: fmtDate(r.delay_until), retry: retryStr });
    default:
      return t('logs.statusUnknown');
  }
}

function renderStatusPill(row: RequestAttemptTypeFixed) {
  const variant = statusVariant(row);
  const label = statusLabel(row);
  const tooltip = statusTooltip(row);
  const colors = STATUS_COLORS[variant] ?? STATUS_COLORS.muted;
  const pillStyle = `display:inline-flex;align-items:center;gap:0.375rem;padding:0.1875rem 0.625rem;border-radius:9999px;font-size:0.8125rem;font-weight:600;white-space:nowrap;background:${colors.bg};color:${colors.fg};cursor:default`;
  const dotStyle = `width:6px;height:6px;border-radius:50%;background:${colors.fg};flex-shrink:0`;
  return h(Hook0Tooltip, { content: tooltip }, () =>
    h('span', { style: pillStyle }, [h('span', { style: dotStyle }), label])
  );
}

function computeDuration(row: RequestAttemptTypeFixed): string {
  const r = row as unknown as Record<string, unknown>;
  const created = r.created_at as string | null;
  const completed = (r.succeeded_at ?? r.failed_at ?? r.completed_at) as string | null;
  if (!created || !completed) return '—';
  const ms = new Date(completed).getTime() - new Date(created).getTime();
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function computeDurationTooltip(row: RequestAttemptTypeFixed): string {
  const r = row as unknown as Record<string, unknown>;
  const created = fmtDate(r.created_at);
  const picked = fmtDate(r.picked_at);
  const completed = fmtDate(r.succeeded_at ?? r.failed_at ?? r.completed_at);
  return t('logs.tooltipDuration', { created, picked, completed });
}

const columns: ColumnDef<RequestAttemptTypeFixed, unknown>[] = [
  {
    accessorKey: 'status',
    header: t('common.status'),
    enableSorting: true,
    cell: (info) => renderStatusPill(info.row.original),
  },
  {
    accessorKey: 'event_id',
    header: t('logs.eventId'),
    cell: (info) => {
      const row = info.row.original;
      const eventType = (row as unknown as Record<string, unknown>).event_type_name as
        | string
        | undefined;
      const RouterLink = resolveComponent('router-link');
      const link = h(
        RouterLink,
        {
          to: {
            name: routes.EventsDetail,
            params: {
              application_id: route.params.application_id,
              organization_id: route.params.organization_id,
              event_id: (row as unknown as RequestAttempt).event_id,
            },
          },
          style:
            'font-family:var(--font-mono);font-size:0.8125rem;font-weight:500;color:var(--color-text-primary);text-decoration:none',
          onMouseenter: (e: MouseEvent) => {
            (e.currentTarget as HTMLElement).style.textDecoration = 'underline';
            (e.currentTarget as HTMLElement).style.textUnderlineOffset = '2px';
          },
          onMouseleave: (e: MouseEvent) => {
            (e.currentTarget as HTMLElement).style.textDecoration = 'none';
          },
        },
        () => String(info.getValue())
      );
      if (eventType) {
        return h('div', { style: 'display:flex;flex-direction:column;gap:0.125rem' }, [
          link,
          h('span', { style: 'font-size:0.75rem;color:var(--color-text-secondary)' }, eventType),
        ]);
      }
      return link;
    },
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
    id: 'duration',
    header: t('logs.duration'),
    cell: (info) =>
      h(Hook0Tooltip, { content: computeDurationTooltip(info.row.original) }, () =>
        h(
          'span',
          {
            style:
              'font-family:var(--font-mono);font-size:0.8125rem;color:var(--color-text-secondary);cursor:default',
          },
          computeDuration(info.row.original)
        )
      ),
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
            <Hook0HelpText tone="emphasis">{{
              t('logs.subtitleRetention', { days: retentionDays })
            }}</Hook0HelpText>
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

<style scoped>
.log-status {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  white-space: nowrap;
}

.log-status--success {
  background-color: var(--color-success-light);
  color: var(--color-success);
}

.log-status--error {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.log-status--warning {
  background-color: var(--color-warning-light);
  color: var(--color-warning);
}

.log-status--info {
  background-color: var(--color-info-light);
  color: var(--color-info);
}

.log-status--muted {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-muted);
}

.log-event-cell {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.log-event-type {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
}

.log-duration {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}
</style>
