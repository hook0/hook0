import http, { type UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];
export type RetrySchedule = definitions['RetrySchedule'];
export type RetrySchedulePost = definitions['RetrySchedulePost'];
export type RetrySchedulePut = definitions['RetrySchedulePut'];

export function list(organization_id: UUID): Promise<RetrySchedule[]> {
  return unwrapResponse(
    http.get<RetrySchedule[]>('/retry_schedules', { params: { organization_id } })
  );
}

export function get(retry_schedule_id: UUID, organization_id: UUID): Promise<RetrySchedule> {
  return unwrapResponse(
    http.get<RetrySchedule>(`/retry_schedules/${retry_schedule_id}`, {
      params: { organization_id },
    })
  );
}

export function create(schedule: RetrySchedulePost): Promise<RetrySchedule> {
  return unwrapResponse(http.post<RetrySchedule>('/retry_schedules', schedule));
}

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

export function remove(retry_schedule_id: UUID, organization_id: UUID): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/retry_schedules/${retry_schedule_id}`, {
      params: { organization_id },
    })
  );
}
