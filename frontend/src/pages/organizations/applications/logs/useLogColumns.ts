import { h } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import type { RequestAttempt } from './LogService';
import { RequestAttemptStatusType } from './LogService';
import { getStatusConfig } from './logStatusConfig';
import { routes } from '@/routes';

import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import Hook0DateFormatted from '@/components/Hook0DateFormatted.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { formatDate, formatRelativeTime } from '@/utils/formatDate';

function statusLabel(row: RequestAttempt, t: ReturnType<typeof useI18n>['t']): string {
  const httpCode = row.http_response_status;
  if (httpCode != null) return `${httpCode}`;
  // Failed with no response = timeout/DNS failure, distinct from a failed attempt with an error HTTP code
  if (row.status.type === RequestAttemptStatusType.Failed && !row.response_id) {
    return t('logs.statusTimeout');
  }
  if (row.status.type === RequestAttemptStatusType.Waiting && row.delay_until) {
    return t('logs.statusQueued', { time: formatRelativeTime(row.delay_until) });
  }
  const config = getStatusConfig(row.status.type);
  return t(config.labelKey);
}

function statusTooltip(row: RequestAttempt, t: ReturnType<typeof useI18n>['t']): string {
  const config = getStatusConfig(row.status.type);
  const retry = Number(row.retry_count ?? 0);
  const retryStr = retry > 0 ? t('logs.tooltipRetry', { count: retry }) : '';
  const date = formatDate(row[config.tooltipDateField]);
  return t(config.tooltipKey, { date, retry: retryStr });
}

function renderStatusPill(row: RequestAttempt, t: ReturnType<typeof useI18n>['t']) {
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

function getEventTypeName(row: RequestAttempt): string {
  return row.event.event_type_name;
}

export function useLogColumns(): ColumnDef<RequestAttempt, unknown>[] {
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
        // Wrapper stops propagation so clicking the link navigates to event detail
        // instead of triggering the row-click handler (which selects the delivery)
        return h(
          'div',
          { onClick: (e: MouseEvent) => e.stopPropagation() },
          h(
            Hook0Button,
            {
              variant: 'link',
              to: {
                name: routes.EventsDetail,
                params: {
                  organization_id: route.params.organization_id,
                  application_id: route.params.application_id,
                  event_id: row.event_id,
                },
              },
              class: 'log-cell-link',
              'data-test': 'log-event-link',
            },
            () => getEventTypeName(row)
          )
        );
      },
    },
    {
      accessorKey: 'created_at',
      header: t('common.createdAt'),
      enableSorting: true,
      cell: (info) => h(Hook0DateFormatted, { value: info.getValue() as string | null }),
    },
  ];
}
