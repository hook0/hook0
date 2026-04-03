// Display formatters for retry schedule data in table cells and badges.
//
// How it works:
// 1. formatDelaySummary renders a one-line delay description per strategy
// 2. strategyLabel maps strategy enum values to i18n display names

import { formatDuration } from '@/utils/formatDuration';
import type { RetrySchedule } from './RetryScheduleService';

type TranslateFn = (key: string, params?: Record<string, unknown>) => string;

/**
 * Renders a one-line delay description for the schedule list table.
 *
 * @example
 * formatDelaySummary({ strategy: 'linear', linear_delay: 60, ... }, t)
 * // => "1min fixed"
 */
export function formatDelaySummary(schedule: RetrySchedule, t: TranslateFn): string {
  switch (schedule.strategy) {
    case 'increasing':
      return t('retrySchedules.delayIncreasing', {
        baseDelay: formatDuration(schedule.increasing_base_delay ?? 0),
        factor: schedule.increasing_wait_factor ?? 0,
      });
    case 'linear':
      return t('retrySchedules.delayLinear', {
        delay: formatDuration(schedule.linear_delay ?? 0),
      });
    case 'custom':
      return t('retrySchedules.delayCustom', {
        count: (schedule.custom_intervals ?? []).length,
      });
    default:
      // Forward-compatible: unknown strategies render as empty rather than crashing
      return '';
  }
}

/**
 * Maps a strategy enum value to its localized display name.
 * Falls back to raw string for unknown strategies (forward-compatible).
 *
 * @example
 * strategyLabel('increasing', t)
 * // => "Increasing"
 */
export function strategyLabel(strategy: string, t: TranslateFn): string {
  switch (strategy) {
    case 'increasing':
      return t('retrySchedules.strategyIncreasing');
    case 'linear':
      return t('retrySchedules.strategyLinear');
    case 'custom':
      return t('retrySchedules.strategyCustom');
    default:
      // Forward-compatible: new strategies from the API display their raw value until i18n catches up
      return strategy;
  }
}
