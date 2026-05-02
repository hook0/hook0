import http, { UUID } from '@/http';
import { subDays } from 'date-fns';
import type { components } from '@/types';
import { followAllPages, unwrapCursorPage } from '@/utils/cursorPagination';
import type { CursorPage } from '@/composables/useCursorInfiniteQuery';

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

// TODO: These fields should be in the OpenAPI-generated RequestAttemptTypeFixed type. Remove this extension when the spec is updated.
export type RequestAttemptExtended = RequestAttemptTypeFixed & {
  retry_count?: number;
  succeeded_at?: string | null;
  failed_at?: string | null;
  picked_at?: string | null;
  delay_until?: string | null;
  completed_at?: string | null;
  created_at?: string | null;
  event_type_name?: string | null;
};

/**
 * Fetch one page of request attempts (delivery log entries).
 * - If `cursor` is undefined → fetches the first page using `application_id`
 *   and a 7-day `min_created_at` filter.
 * - If `cursor` is a fully-qualified URL (extracted from the `Link` header of
 *   a previous response) → axios follows it directly. The backend preserves
 *   the original filters in the cursor URL so we don't need to re-pass them.
 */
export function listPage(
  application_id: UUID,
  cursor?: string
): Promise<CursorPage<RequestAttemptTypeFixed>> {
  if (cursor) {
    return unwrapCursorPage(http.get<RequestAttemptTypeFixed[]>(cursor));
  }
  return unwrapCursorPage(
    http.get<RequestAttemptTypeFixed[]>('/request_attempts', {
      params: {
        application_id: application_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
    })
  );
}

/**
 * Backward-compatible shim that follows ALL cursors and returns a flat array.
 *
 * NOTE: Fixes the silent-drop bug where the previous implementation only ever
 * fetched the first page from the cursor-paginated `request_attempts` endpoint
 * (issue #45 — adjacent bug).
 */
export function list(application_id: UUID): Promise<Array<RequestAttemptTypeFixed>> {
  return followAllPages<RequestAttemptTypeFixed>(
    () => listPage(application_id),
    (url) => listPage(application_id, url)
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
