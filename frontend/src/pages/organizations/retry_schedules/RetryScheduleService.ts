// HTTP service layer for retry schedule CRUD.
//
// How it works:
// 1. Thin wrappers around axios calls to /retry_schedules endpoints
// 2. unwrapResponse extracts .data from AxiosResponse, centralizing error handling
// 3. Types are generated from the OpenAPI spec (components['schemas'])
// 4. Consumed exclusively by useRetryScheduleQueries — never called directly from components

import http, { type UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];
export type RetrySchedule = definitions['RetrySchedule'];
export type RetrySchedulePost = definitions['RetrySchedulePost'];
export type RetrySchedulePut = definitions['RetrySchedulePut'];

/**
 * Fetches all retry schedules for an organization.
 *
 * @example
 * list('org-123')
 * // => Promise<RetrySchedule[]>
 */
export function list(organization_id: UUID): Promise<RetrySchedule[]> {
  return unwrapResponse(
    http.get<RetrySchedule[]>('/retry_schedules', { params: { organization_id } })
  );
}

/**
 * Fetches a single retry schedule by ID.
 *
 * @example
 * get('sched-456', 'org-123')
 * // => Promise<RetrySchedule>
 */
export function get(retry_schedule_id: UUID, organization_id: UUID): Promise<RetrySchedule> {
  return unwrapResponse(
    http.get<RetrySchedule>(`/retry_schedules/${retry_schedule_id}`, {
      params: { organization_id },
    })
  );
}

/**
 * Creates a new retry schedule.
 *
 * @example
 * create({ organization_id: 'org-123', name: 'Fast', strategy: 'linear', ... })
 * // => Promise<RetrySchedule>
 */
export function create(schedule: RetrySchedulePost): Promise<RetrySchedule> {
  return unwrapResponse(http.post<RetrySchedule>('/retry_schedules', schedule));
}

/**
 * Updates an existing retry schedule (full replace).
 *
 * @example
 * update('sched-456', 'org-123', { name: 'Slow', strategy: 'increasing', ... })
 * // => Promise<RetrySchedule>
 */
export function update(
  retry_schedule_id: UUID,
  organization_id: UUID,
  schedule: RetrySchedulePut
): Promise<RetrySchedule> {
  return unwrapResponse(
    http.put<RetrySchedule>(`/retry_schedules/${retry_schedule_id}`, schedule, {
      params: { organization_id },
    })
  );
}

/**
 * Deletes a retry schedule. Subscriptions referencing it will lose their schedule FK.
 *
 * @example
 * remove('sched-456', 'org-123')
 * // => Promise<void>
 */
export function remove(retry_schedule_id: UUID, organization_id: UUID): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/retry_schedules/${retry_schedule_id}`, {
      params: { organization_id },
    })
  );
}
