import { h } from 'vue';
import { useRoute } from 'vue-router';
import Hook0Uuid from '@/components/Hook0Uuid.vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import type { RequestAttemptExtended } from './LogService';
import { getStatusConfig } from './logStatusConfig';
import { routes } from '@/routes';

import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import Hook0Button from '@/components/Hook0Button.vue';

const dateFmt = new Intl.DateTimeFormat(undefined, {
  month: 'short',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
});

function fmtDate(val: unknown): string {
  if (!val || typeof val !== 'string') return '\u2014';
  try {
    return dateFmt.format(new Date(val));
  } catch {
    return String(val);
  }
}

function statusLabel(row: RequestAttemptExtended, t: ReturnType<typeof useI18n>['t']): string {
  const config = getStatusConfig(row.status.type);
  const shortTitle = t(config.labelKey);
  const httpCode = row.http_response_status;
  return httpCode ? `${httpCode} ${shortTitle}` : shortTitle;
}

function statusTooltip(row: RequestAttemptExtended, t: ReturnType<typeof useI18n>['t']): string {
  const config = getStatusConfig(row.status.type);
  const retry = Number(row.retry_count ?? 0);
  const retryStr = retry > 0 ? t('logs.tooltipRetry', { count: retry }) : '';
  const date = fmtDate(row[config.tooltipDateField]);
  return t(config.tooltipKey, { date, retry: retryStr });
}

function renderStatusPill(row: RequestAttemptExtended, t: ReturnType<typeof useI18n>['t']) {
  const config = getStatusConfig(row.status.type);
  const label = statusLabel(row, t);
  const tooltip = statusTooltip(row, t);
  return h(Hook0Tooltip, { content: tooltip }, () =>
    h(
      'span',
      {
        class: ['log-status', `log-status--${config.variant}`],
        role: 'status',
        'aria-label': label,
      },
      [h('span', { class: 'log-status__dot', 'aria-hidden': 'true' }), label]
    )
  );
}

function computeDuration(row: RequestAttemptExtended): string {
  const created = row.created_at;
  const completed = row.succeeded_at ?? row.failed_at ?? row.completed_at;
  if (!created || !completed) return '\u2014';
  const ms = new Date(completed).getTime() - new Date(created).getTime();
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function computeDurationTooltip(
  row: RequestAttemptExtended,
  t: ReturnType<typeof useI18n>['t']
): string {
  const created = fmtDate(row.created_at);
  const picked = fmtDate(row.picked_at);
  const completed = fmtDate(row.succeeded_at ?? row.failed_at ?? row.completed_at);
  return t('logs.tooltipDuration', { created, picked, completed });
}

export function useLogColumns(): ColumnDef<RequestAttemptExtended, unknown>[] {
  const { t } = useI18n();
  const route = useRoute();

  return [
    {
      accessorKey: 'status',
      header: t('common.status'),
      enableSorting: true,
      cell: (info) => renderStatusPill(info.row.original, t),
    },
    {
      accessorKey: 'event_id',
      header: t('logs.eventId'),
      cell: (info) => {
        const row = info.row.original;
        const eventType = row.event_type_name;
        const link = h(
          Hook0Button,
          {
            variant: 'link',
            to: {
              name: routes.EventsDetail,
              params: { ...route.params, event_id: row.event_id },
            },
            onClick: (e: MouseEvent) => e.stopPropagation(),
            'data-test': 'log-event-link',
            style: 'color: var(--color-link)',
          },
          () =>
            h(Hook0Uuid, {
              value: String(info.getValue()),
              truncated: true,
              style: 'color: inherit',
            })
        );
        if (eventType) {
          return h('div', { class: 'log-event-cell' }, [
            link,
            h('span', { class: 'log-event-type' }, eventType),
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
        h(Hook0Tooltip, { content: computeDurationTooltip(info.row.original, t) }, () =>
          h('span', { class: 'log-duration' }, computeDuration(info.row.original))
        ),
    },
  ];
}
