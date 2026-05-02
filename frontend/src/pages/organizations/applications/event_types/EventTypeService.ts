import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';
import { followAllPages, unwrapCursorPage } from '@/utils/cursorPagination';
import type { CursorPage } from '@/composables/useCursorInfiniteQuery';

type definitions = components['schemas'];

export type EventType = definitions['EventType'];
export type EventTypePost = definitions['EventTypePost'];

export function create(event_type: EventTypePost): Promise<EventType> {
  return unwrapResponse(http.post<EventType>('/event_types', event_type));
}

export function deactivate(application_id: string, event_type_name: string): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/event_types/${event_type_name}`, {
      params: {
        application_id,
      },
    })
  );
}

/**
 * Fetch one page of event types.
 * - If `cursor` is undefined → fetches the first page using `application_id`.
 * - If `cursor` is a fully-qualified URL (extracted from the `Link` header of
 *   a previous response) → axios follows it directly.
 */
export function listPage(application_id: UUID, cursor?: string): Promise<CursorPage<EventType>> {
  if (cursor) {
    return unwrapCursorPage(http.get<EventType[]>(cursor));
  }
  return unwrapCursorPage(
    http.get<EventType[]>('/event_types', {
      params: {
        application_id: application_id,
      },
    })
  );
}

/**
 * Backward-compatible shim that follows all cursors and returns a flat array.
 * Used by dropdowns (e.g. SubscriptionsEdit, EventsSend) that need the full
 * list of event types in one shot.
 */
export function list(application_id: UUID): Promise<Array<EventType>> {
  return followAllPages<EventType>(
    () => listPage(application_id),
    (url) => listPage(application_id, url)
  );
}

export function get(id: UUID): Promise<EventType> {
  return unwrapResponse(http.get<EventType>(`/event_types/${id}`));
}
