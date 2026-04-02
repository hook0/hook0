import type { Component } from 'vue';
import { CheckCircle2, XCircle, Clock, Loader, CircleDashed } from 'lucide-vue-next';
import { RequestAttemptStatusType } from './LogService';

export type StatusVariant = 'success' | 'error' | 'warning' | 'info' | 'muted';

export interface StatusConfig {
  labelKey: string;
  variant: StatusVariant;
  tooltipKey: string;
  tooltipDateField: 'succeeded_at' | 'failed_at' | 'created_at' | 'picked_at' | 'delay_until';
  icon: Component;
}

export const STATUS_CONFIG: Record<RequestAttemptStatusType, StatusConfig> = {
  [RequestAttemptStatusType.Successful]: {
    labelKey: 'logs.statusSent',
    variant: 'success',
    tooltipKey: 'logs.tooltipSuccessful',
    tooltipDateField: 'succeeded_at',
    icon: CheckCircle2,
  },
  [RequestAttemptStatusType.Failed]: {
    labelKey: 'logs.statusFailed',
    variant: 'error',
    tooltipKey: 'logs.tooltipFailed',
    tooltipDateField: 'failed_at',
    icon: XCircle,
  },
  [RequestAttemptStatusType.Pending]: {
    labelKey: 'logs.statusPending',
    variant: 'warning',
    tooltipKey: 'logs.tooltipPending',
    tooltipDateField: 'created_at',
    icon: CircleDashed,
  },
  [RequestAttemptStatusType.InProgress]: {
    labelKey: 'logs.statusRetrying',
    variant: 'warning',
    tooltipKey: 'logs.tooltipInProgress',
    tooltipDateField: 'picked_at',
    icon: Loader,
  },
  [RequestAttemptStatusType.Waiting]: {
    labelKey: 'logs.statusWaiting',
    variant: 'info',
    tooltipKey: 'logs.tooltipWaiting',
    tooltipDateField: 'delay_until',
    icon: Clock,
  },
};

// Fallback for status types the frontend doesn't recognize yet (API/frontend version skew)
export const DEFAULT_STATUS_CONFIG: StatusConfig = {
  labelKey: 'logs.statusSkipped',
  variant: 'muted',
  tooltipKey: 'logs.statusUnknown',
  tooltipDateField: 'created_at',
  icon: CircleDashed,
};

export function getStatusConfig(type: RequestAttemptStatusType): StatusConfig {
  return STATUS_CONFIG[type] ?? DEFAULT_STATUS_CONFIG;
}
