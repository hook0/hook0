import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';
import { followAllPages, unwrapCursorPage } from '@/utils/cursorPagination';
import type { CursorPage } from '@/composables/useCursorInfiniteQuery';

type definitions = components['schemas'];

export type Subscription = definitions['Subscription'];
export type SubscriptionPost = definitions['SubscriptionPost'];

export type SubscriptionEnableToggle = {
  is_enabled: boolean;
};

export function create(subscription: SubscriptionPost): Promise<Subscription> {
  return unwrapResponse(http.post<Subscription>('/subscriptions', subscription));
}

export function remove(application_id: UUID, subscription_id: UUID): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/subscriptions/${subscription_id}`, {
      params: {
        application_id,
      },
    })
  );
}

export function update(
  subscription_id: UUID,
  subscription: SubscriptionPost | SubscriptionEnableToggle
): Promise<Subscription> {
  return unwrapResponse(http.put<Subscription>(`/subscriptions/${subscription_id}`, subscription));
}

export function toggleEnable(
  subscription_id: UUID,
  subscription: Subscription
): Promise<Subscription> {
  return update(subscription_id, {
    application_id: subscription.application_id,

    event_types: subscription.event_types,
    target: subscription.target,

    is_enabled: !subscription.is_enabled,

    description: subscription.description,
    metadata: subscription.metadata,

    labels: subscription.labels,
  });
}

/**
 * Fetch one page of subscriptions.
 * - If `cursor` is undefined → fetches the first page using `application_id`.
 * - If `cursor` is a fully-qualified URL (extracted from the `Link` header of
 *   a previous response) → axios follows it directly.
 */
export function listPage(application_id: UUID, cursor?: string): Promise<CursorPage<Subscription>> {
  if (cursor) {
    return unwrapCursorPage(http.get<Subscription[]>(cursor));
  }
  return unwrapCursorPage(
    http.get<Subscription[]>('/subscriptions', {
      params: {
        application_id: application_id,
      },
    })
  );
}

/**
 * Backward-compatible shim that follows all cursors and returns a flat array.
 */
export function list(application_id: UUID): Promise<Array<Subscription>> {
  return followAllPages<Subscription>(
    () => listPage(application_id),
    (url) => listPage(application_id, url)
  );
}

export function get(id: UUID): Promise<Subscription> {
  return unwrapResponse(http.get<Subscription>(`/subscriptions/${id}`));
}
