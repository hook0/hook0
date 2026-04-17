import http, { UUID } from '@/http';
import { subDays } from 'date-fns';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type RequestAttempt = definitions['RequestAttempt'];

export const enum RequestAttemptStatusType {
  Waiting = 'waiting',
  Pending = 'pending',
  InProgress = 'in_progress',
  Successful = 'successful',
  Failed = 'failed',
}

export type RequestAttemptStatus = {
  type: RequestAttemptStatusType;
};

type Modify<T, R> = Omit<T, keyof R> & R;

export type RequestAttemptTypeFixed = Modify<RequestAttempt, { status: RequestAttemptStatus }>;

// TODO: These fields should be in the OpenAPI-generated RequestAttemptTypeFixed type. Remove this extension when the spec is updated.
export type RequestAttemptExtended = RequestAttemptTypeFixed & {
  succeeded_at?: string | null;
  completed_at?: string | null;
  event_type_name?: string | null;
};

export function list(application_id: UUID): Promise<Array<RequestAttemptTypeFixed>> {
  return unwrapResponse(
    http.get<Array<RequestAttemptTypeFixed>>('/request_attempts', {
      params: {
        application_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
    })
  );
}

export function retry(requestAttemptId: UUID): Promise<{ request_attempt_id: string }> {
  return unwrapResponse(
    http.post<{ request_attempt_id: string }>(`/request_attempts/${requestAttemptId}/retry`)
  );
}

/** Fetch deliveries scoped to a single subscription — used by the subscription detail page */
export function listBySubscription(
  application_id: UUID,
  subscription_id: UUID
): Promise<Array<RequestAttemptTypeFixed>> {
  return unwrapResponse(
    http.get<Array<RequestAttemptTypeFixed>>('/request_attempts', {
      params: {
        application_id,
        subscription_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
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
