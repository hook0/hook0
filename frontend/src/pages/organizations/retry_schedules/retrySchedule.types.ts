/**
 * Local types for retry_schedule feature. Mirrors the backend OpenAPI shape exactly.
 * TODO: swap for `components['schemas']['RetrySchedule*']` once `npm run generate:types`
 * is re-run against the branch's running API.
 */

export type RetryStrategy = 'exponential_increasing' | 'linear' | 'custom';

export interface RetrySchedule {
  retry_schedule_id: string;
  organization_id: string;
  name: string;
  strategy: RetryStrategy;
  max_retries: number;
  custom_intervals: number[] | null;
  linear_delay: number | null;
  increasing_base_delay: number | null;
  increasing_wait_factor: number | null;
  created_at: string;
  updated_at: string;
}

/** PUT body. Fields irrelevant to the strategy must be `null`; backend has `deny_unknown_fields`. */
export interface RetrySchedulePayload {
  name: string;
  strategy: RetryStrategy;
  max_retries: number;
  custom_intervals: number[] | null;
  linear_delay: number | null;
  increasing_base_delay: number | null;
  increasing_wait_factor: number | null;
}

/** POST body. Same as PUT plus the owning organization. */
export type RetryScheduleCreatePayload = RetrySchedulePayload & {
  organization_id: string;
};

/** Snapshot of the /instance `retry_schedule` limits block. */
export interface RetryScheduleLimits {
  min_single_delay_secs: number;
  max_single_delay_secs: number;
  max_retries: number;
  max_custom_intervals_length: number;
  max_total_duration_secs: number;
  exponential_base_delay_min_secs: number;
  exponential_base_delay_max_secs: number;
  exponential_wait_factor_min: number;
  exponential_wait_factor_max: number;
  max_per_organization: number;
}
