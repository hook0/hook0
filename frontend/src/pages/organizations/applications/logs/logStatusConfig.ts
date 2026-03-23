import { RequestAttemptStatusType } from './LogService';

export type StatusVariant = 'success' | 'error' | 'warning' | 'info' | 'muted';

export interface StatusConfig {
  labelKey: string;
  variant: StatusVariant;
  tooltipKey: string;
  tooltipDateField: 'succeeded_at' | 'failed_at' | 'created_at' | 'picked_at' | 'delay_until';
}

export const STATUS_CONFIG: Record<RequestAttemptStatusType, StatusConfig> = {
  [RequestAttemptStatusType.Successful]: {
    labelKey: 'logs.statusSent',
    variant: 'success',
    tooltipKey: 'logs.tooltipSuccessful',
    tooltipDateField: 'succeeded_at',
  },
  [RequestAttemptStatusType.Failed]: {
    labelKey: 'logs.statusFailed',
    variant: 'error',
    tooltipKey: 'logs.tooltipFailed',
    tooltipDateField: 'failed_at',
  },
  [RequestAttemptStatusType.Pending]: {
    labelKey: 'logs.statusPending',
    variant: 'warning',
    tooltipKey: 'logs.tooltipPending',
    tooltipDateField: 'created_at',
  },
  [RequestAttemptStatusType.InProgress]: {
    labelKey: 'logs.statusRetrying',
    variant: 'warning',
    tooltipKey: 'logs.tooltipInProgress',
    tooltipDateField: 'picked_at',
  },
  [RequestAttemptStatusType.Waiting]: {
    labelKey: 'logs.statusQueued',
    variant: 'info',
    tooltipKey: 'logs.tooltipWaiting',
    tooltipDateField: 'delay_until',
  },
};

export const DEFAULT_STATUS_CONFIG: StatusConfig = {
  labelKey: 'logs.statusSkipped',
  variant: 'muted',
  tooltipKey: 'logs.statusUnknown',
  tooltipDateField: 'created_at',
};

export function getStatusConfig(type: RequestAttemptStatusType): StatusConfig {
  return STATUS_CONFIG[type] ?? DEFAULT_STATUS_CONFIG;
}
