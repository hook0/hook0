// Display formatters and delay computation for retry schedules.
//
// How it works:
// 1. formatDelaySummary renders a one-line delay description per strategy (list page)
// 2. strategyLabel maps strategy enum values to i18n display names
// 3. formatDelay produces a short human-readable form like "5s", "3min", "2h", "1d"
// 4. computeDelays projects a schedule into its actual delay sequence, clamped by the backend max_single_delay_secs cap

import type { useI18n } from 'vue-i18n';

import { formatDuration } from '@/utils/formatDuration';
import type { RetrySchedule } from './RetryScheduleService';
import type { RetryScheduleLimits } from './useRetryScheduleLimits';

type TranslateFn = ReturnType<typeof useI18n>['t'];

/**
 * Renders a one-line delay description for the schedule list table.
 *
 * @example
 * formatDelaySummary({ strategy: 'linear', linear_delay_secs: 60, ... }, t)
 * // => "1min fixed"
 */
export function formatDelaySummary(schedule: RetrySchedule, t: TranslateFn): string {
  switch (schedule.strategy) {
    case 'exponential_increasing':
      return t('retrySchedules.delayIncreasing', {
        baseDelay: formatDuration(schedule.increasing_base_delay_secs ?? 0),
        factor: schedule.increasing_wait_factor ?? 0,
      });
    case 'linear':
      return t('retrySchedules.delayLinear', {
        delay: formatDuration(schedule.linear_delay_secs ?? 0),
      });
    case 'custom':
      return t('retrySchedules.delayCustom', {
        count: (schedule.custom_intervals_secs ?? []).length,
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
 * strategyLabel('exponential_increasing', t)
 * // => "Increasing"
 */
export function strategyLabel(strategy: RetrySchedule['strategy'], t: TranslateFn): string {
  switch (strategy) {
    case 'exponential_increasing':
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

/** Short human-readable form like "5s", "3min", "2h", "1d". Floor-based so thresholds don't round up. */
export function formatDelay(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }
  if (seconds < 3600) {
    return `${Math.floor(seconds / 60)}min`;
  }
  if (seconds < 86400) {
    return `${Math.floor(seconds / 3600)}h`;
  }
  return `${Math.floor(seconds / 86400)}d`;
}

/**
 * Compute the exact delay sequence (in seconds) the schedule will emit.
 * Each term is clamped to the max_single_delay_secs policy to mirror the worker.
 */
export function computeDelays(schedule: RetrySchedule, limits: RetryScheduleLimits): number[] {
  const cap = limits.max_single_delay_secs;
  switch (schedule.strategy) {
    case 'exponential_increasing':
      return exponentialDelays(schedule, cap);
    case 'linear':
      return linearDelays(schedule, cap);
    case 'custom':
      return customDelays(schedule, cap);
    default:
      return assertNever(schedule.strategy);
  }
}

function exponentialDelays(schedule: RetrySchedule, cap: number): number[] {
  const base = schedule.increasing_base_delay_secs ?? 0;
  const factor = schedule.increasing_wait_factor ?? 0;
  const delays: number[] = [];
  for (let index = 0; index < schedule.max_retries; index += 1) {
    const term = Math.floor(base * Math.pow(factor, index));
    delays.push(Math.min(term, cap));
  }
  return delays;
}

function linearDelays(schedule: RetrySchedule, cap: number): number[] {
  const delay = Math.min(schedule.linear_delay_secs ?? 0, cap);
  return Array.from({ length: schedule.max_retries }, () => delay);
}

function customDelays(schedule: RetrySchedule, cap: number): number[] {
  return (schedule.custom_intervals_secs ?? []).map((value) => Math.min(value, cap));
}

function assertNever(value: never): never {
  throw new Error(`Unexpected retry strategy: ${String(value)}`);
}
