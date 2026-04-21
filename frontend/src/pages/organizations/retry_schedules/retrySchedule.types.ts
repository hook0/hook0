import type { components } from '@/types';

type definitions = components['schemas'];

export type RetryStrategy = 'exponential_increasing' | 'linear' | 'custom';
export type RetrySchedule = definitions['RetrySchedule'];

/** PUT body. Shared fields without `organization_id`. */
export type RetrySchedulePayload = definitions['RetryScheduleFields'];

/** POST body. `organization_id` + the shared body fields (flattened for the wire). */
export type RetryScheduleCreatePayload = definitions['RetrySchedulePost'];

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
  default_schedule_delays_secs: number[];
}
