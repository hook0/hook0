// Display formatters and delay computation for retry schedules.
//
// How it works:
// 1. strategyLabel maps strategy enum values to i18n display names
// 2. computeDelaysRaw projects a schedule into its configured delay sequence (no clamp)
// 3. computeDelays mirrors the worker by clamping each term to max_single_delay_secs

import type { useI18n } from 'vue-i18n';

import i18n from '@/plugins/i18n';
import { formatDuration } from '@/utils/duration';

import type { RetrySchedule } from './RetryScheduleService';
import type { RetryScheduleLimits } from './useRetryScheduleLimits';

type TranslateFn = ReturnType<typeof useI18n>['t'];

/** Canonical retry strategy type — single source across edit form, preview and list. */
export type RetryStrategy = RetrySchedule['strategy'];

/** Shape accepted by computeDelays/computeDelaysRaw — RetrySchedule plus the edit-form draft. */
export type ScheduleDelayInput = {
  strategy: RetryStrategy;
  max_retries: number;
  linear_delay_secs?: number | null;
  custom_intervals_secs?: number[] | null;
  increasing_base_delay_secs?: number | null;
  increasing_wait_factor?: number | null;
};

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

/**
 * Compute the configured delay sequence (in seconds) without clamping.
 * Uses Math.floor so the projected integer matches the worker's `as u64` truncation,
 * but keeps the raw value so callers can flag terms that exceed the cap instead of
 * silently clipping them.
 */
export function computeDelaysRaw(schedule: ScheduleDelayInput): number[] {
  switch (schedule.strategy) {
    case 'exponential_increasing': {
      const base = schedule.increasing_base_delay_secs ?? 0;
      const factor = schedule.increasing_wait_factor ?? 0;
      return Array.from({ length: schedule.max_retries }, (_, index) =>
        Math.floor(base * Math.pow(factor, index))
      );
    }
    case 'linear':
      return Array.from({ length: schedule.max_retries }, () => schedule.linear_delay_secs ?? 0);
    case 'custom':
      return schedule.custom_intervals_secs ?? [];
    default:
      // Forward-compatible: unknown future strategies project to an empty preview
      return [];
  }
}

/**
 * Compute the effective delay sequence (in seconds) the worker will emit.
 * Each term is clamped to the max_single_delay_secs policy to mirror the worker.
 */
export function computeDelays(schedule: ScheduleDelayInput, limits: RetryScheduleLimits): number[] {
  const cap = limits.max_single_delay_secs;
  return computeDelaysRaw(schedule).map((value) => Math.min(value, cap));
}

/**
 * Format a list of delays (in seconds) as a locale-aware, human-readable string —
 * e.g. "15 seconds, 1 minute, and 5 minutes" in English. Uses Intl.ListFormat so the
 * conjunction/separator rules follow the active i18n locale instead of a hardcoded
 * comma-space join.
 */
export function formatDelayList(delaysSecs: readonly number[]): string {
  const formatter = new Intl.ListFormat(i18n.global.locale.value, {
    style: 'long',
    type: 'conjunction',
  });
  return formatter.format(delaysSecs.map((secs) => formatDuration(secs)));
}
