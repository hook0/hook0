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

export type RequestAttemptTypeFixed = Modify<RequestAttempt, { status: RequestAttemptStatus }>;

export function list(application_id: UUID): Promise<Array<RequestAttemptTypeFixed>> {
  return unwrapResponse(
    http.get<Array<RequestAttemptTypeFixed>>('/request_attempts', {
      params: {
        application_id: application_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
    })
  );
}
