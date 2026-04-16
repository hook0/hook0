// HTTP service layer for subscription CRUD. Wraps Axios calls consumed by TanStack Query composables.
import http, { type UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

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

/**
 * Flip a subscription's is_enabled flag.
 * The PUT endpoint expects the full resource body — we clone all fields and only flip is_enabled.
 *
 * @example
 * toggleEnable('sub-123', subscription) // => Promise<Subscription> with is_enabled toggled
 */
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

export function list(application_id: UUID): Promise<Subscription[]> {
  return unwrapResponse(
    http.get<Subscription[]>('/subscriptions', {
      params: {
        application_id: application_id,
      },
    })
  );
}

export function get(id: UUID): Promise<Subscription> {
  return unwrapResponse(http.get<Subscription>(`/subscriptions/${id}`));
}

/** Type guard: only HTTP targets have method+url */
export function targetIsHttp(
  target: unknown
): target is { type: 'http'; method: string; url: string } {
  return !!target && typeof target === 'object' && 'type' in target && target.type === 'http';
}
