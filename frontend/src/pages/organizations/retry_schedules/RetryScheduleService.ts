import http from '@/http';
import type {
  RetrySchedule,
  RetryScheduleCreatePayload,
  RetrySchedulePayload,
} from './retrySchedule.types';

export function listSchedules(organizationId: string): Promise<RetrySchedule[]> {
  return http
    .get<RetrySchedule[]>('/retry_schedules', { params: { organization_id: organizationId } })
    .then((response) => response.data);
}

export function getSchedule(retryScheduleId: string): Promise<RetrySchedule> {
  return http
    .get<RetrySchedule>(`/retry_schedules/${retryScheduleId}`)
    .then((response) => response.data);
}

export function createSchedule(body: RetryScheduleCreatePayload): Promise<RetrySchedule> {
  return http
    .post<RetrySchedule>('/retry_schedules', {
      organization_id: body.organization_id,
      ...body.payload,
    })
    .then((response) => response.data);
}

export function updateSchedule(
  retryScheduleId: string,
  payload: RetrySchedulePayload
): Promise<RetrySchedule> {
  return http
    .put<RetrySchedule>(`/retry_schedules/${retryScheduleId}`, payload)
    .then((response) => response.data);
}

export function deleteSchedule(retryScheduleId: string): Promise<void> {
  return http.delete<void>(`/retry_schedules/${retryScheduleId}`).then(() => undefined);
}
