import { h } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import type { RequestAttemptExtended } from './LogService';
import { RequestAttemptStatusType } from './LogService';
import { getStatusConfig } from './logStatusConfig';
import { routes } from '@/routes';

import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import Hook0DateFormatted from '@/components/Hook0DateFormatted.vue';
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

function formatRelativeTime(dateStr: string): string {
  const diff = new Date(dateStr).getTime() - Date.now();
  if (diff <= 0) return '<1m';
  const mins = Math.ceil(diff / 60000);
  if (mins < 60) return `${mins}m`;
  return `${Math.floor(mins / 60)}h${mins % 60}m`;
}

function statusLabel(row: RequestAttemptExtended, t: ReturnType<typeof useI18n>['t']): string {
  const httpCode = row.http_response_status;
  if (httpCode != null) return `${httpCode}`;
  if (row.status.type === RequestAttemptStatusType.Failed && !row.response_id) {
    return t('logs.statusTimeout');
  }
  if (row.status.type === RequestAttemptStatusType.Waiting && row.delay_until) {
    return t('logs.statusQueued', { time: formatRelativeTime(row.delay_until) });
  }
  const config = getStatusConfig(row.status.type);
  return t(config.labelKey);
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
      [h(config.icon, { size: 14, 'aria-hidden': 'true', class: 'log-status__icon' }), label]
    )
  );
}

function getEventTypeName(row: RequestAttemptExtended): string {
  return row.event_type_name ?? row.event?.event_type_name ?? row.event_id;
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
      header: t('logs.event'),
      cell: (info) => {
        const row = info.row.original;
        return h(
          Hook0Button,
          {
            variant: 'link',
            to: {
              name: routes.EventsDetail,
              params: { ...route.params, event_id: row.event_id },
            },
            onClick: (e: MouseEvent) => e.stopPropagation(),
            style: 'font-size: 0.875rem',
          },
          () => getEventTypeName(row)
        );
      },
    },
    {
      id: 'subscription',
      header: t('logs.subscription'),
      enableSorting: true,
      cell: (info) => {
        const row = info.row.original;
        return h(
          Hook0Button,
          {
            variant: 'link',
            to: {
              name: routes.SubscriptionsDetail,
              params: {
                organization_id: route.params.organization_id,
                application_id: route.params.application_id,
                subscription_id: row.subscription.subscription_id,
              },
            },
            onClick: (e: MouseEvent) => e.stopPropagation(),
            style: 'font-size: 0.875rem',
          },
          () => String(row.subscription.description ?? row.subscription.subscription_id)
        );
      },
    },
    {
      accessorKey: 'created_at',
      header: () =>
        h(
          'span',
          { style: 'display:flex;justify-content:flex-end;width:100%' },
          t('common.createdAt')
        ),
      enableSorting: true,
      meta: { align: 'right' },
      cell: (info) =>
        h('div', { style: 'text-align:right' }, [
          h(Hook0DateFormatted, { value: info.getValue() as string | null }),
        ]),
    },
  ];
}
