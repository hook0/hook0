import http, { UUID } from '@/http';
import { subDays } from 'date-fns';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type RequestAttempt = definitions['RequestAttempt'];

export const enum RequestAttemptStatusType {
  Waiting = 'waiting',
  Pending = 'pending',
  InProgress = 'inprogress',
  Successful = 'successful',
  Failed = 'failed',
}

export type RequestAttemptStatus = {
  type: RequestAttemptStatusType;
};

type Modify<T, R> = Omit<T, keyof R> & R;

export type RequestAttemptExtended = Modify<RequestAttempt, { status: RequestAttemptStatus }>;

export function list(application_id: UUID): Promise<Array<RequestAttemptExtended>> {
  return unwrapResponse(
    http.get<Array<RequestAttemptExtended>>('/request_attempts', {
      params: {
        application_id: application_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
    })
  );
}

export function retry(
  requestAttemptId: UUID,
  applicationId: UUID
): Promise<{ request_attempt_id: string }> {
  return unwrapResponse(
    http.post<{ request_attempt_id: string }>(`/request_attempts/${requestAttemptId}/retry`, null, {
      params: { application_id: applicationId },
    })
  );
}

export function getById(
  requestAttemptId: UUID,
  applicationId: UUID
): Promise<RequestAttemptExtended> {
  return http
    .get<RequestAttemptExtended>(`/request_attempts/${requestAttemptId}`, {
      params: { application_id: applicationId },
    })
    .then((res) => res.data);
}
