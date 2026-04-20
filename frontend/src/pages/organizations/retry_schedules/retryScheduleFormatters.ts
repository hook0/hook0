import type { RetrySchedule, RetryScheduleLimits } from './retrySchedule.types';

/**
 * Compute the exact delay sequence (in seconds) the schedule will emit.
 * Each term is clamped to the max_single_delay policy to mirror the worker.
 */
export function computeDelays(schedule: RetrySchedule, limits: RetryScheduleLimits): number[] {
  const cap = limits.max_single_delay_secs;
  switch (schedule.strategy) {
    case 'exponential_increasing': {
      const base = schedule.increasing_base_delay ?? 0;
      const factor = schedule.increasing_wait_factor ?? 0;
      const delays: number[] = [];
      for (let i = 0; i < schedule.max_retries; i += 1) {
        const term = Math.floor(base * Math.pow(factor, i));
        delays.push(Math.min(term, cap));
      }
      return delays;
    }
    case 'linear': {
      const delay = Math.min(schedule.linear_delay ?? 0, cap);
      return Array.from({ length: schedule.max_retries }, () => delay);
    }
    case 'custom':
      return (schedule.custom_intervals ?? []).map((v) => Math.min(v, cap));
  }
}

/** Short human-readable form like "5s", "3min", "2h", "1d". */
export function formatDelay(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}min`;
  if (seconds < 86400) return `${Math.round(seconds / 3600)}h`;
  return `${Math.round(seconds / 86400)}d`;
}
