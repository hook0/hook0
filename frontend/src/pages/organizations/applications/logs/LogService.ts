import http, { UUID } from '@/http';
import { subDays } from 'date-fns';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type RequestAttempt = definitions['RequestAttempt'];

export type RequestAttemptStatusType = RequestAttempt['status']['type'];

type Capitalize_<S extends string> = S extends `${infer H}${infer T}`
  ? `${Uppercase<H>}${T}`
  : S;

export const RequestAttemptStatusType = {
  Waiting: 'waiting',
  Pending: 'pending',
  Inprogress: 'inprogress',
  Successful: 'successful',
  Failed: 'failed',
} as const satisfies { [K in RequestAttemptStatusType as Capitalize_<K>]: K };

export function list(application_id: UUID): Promise<Array<RequestAttempt>> {
  return unwrapResponse(
    http.get<Array<RequestAttempt>>('/request_attempts', {
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
): Promise<Array<RequestAttempt>> {
  return unwrapResponse(
    http.get<Array<RequestAttempt>>('/request_attempts', {
      params: {
        application_id,
        subscription_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
    })
  );
}

export function getById(requestAttemptId: UUID, applicationId: UUID): Promise<RequestAttempt> {
  return http
    .get<RequestAttempt>(`/request_attempts/${requestAttemptId}`, {
      params: { application_id: applicationId },
    })
    .then((res) => res.data);
}
