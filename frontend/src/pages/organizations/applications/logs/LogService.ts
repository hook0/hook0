import http, { UUID } from '@/http';
import { subDays } from 'date-fns';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type RequestAttempt = definitions['RequestAttempt'];

export type RequestAttemptStatusType = RequestAttempt['status']['type'];

// Transforms snake_case to PascalCase: 'in_progress' → 'InProgress'
type SnakeToPascal<S extends string> = S extends `${infer H}_${infer T}`
  ? `${Capitalize<H>}${SnakeToPascal<T>}`
  : Capitalize<S>;

// Mapped type requires one key per union variant, key name derived from value.
// Adding/removing/renaming a variant in the OpenAPI type fails compilation here.
export const RequestAttemptStatusType = {
  Waiting: 'waiting',
  Pending: 'pending',
  InProgress: 'in_progress',
  Successful: 'successful',
  Failed: 'failed',
} as const satisfies { [K in RequestAttemptStatusType as SnakeToPascal<K>]: K };

export type RequestAttemptStatus = RequestAttempt['status'];

export type RequestAttemptTypeFixed = RequestAttempt;

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
