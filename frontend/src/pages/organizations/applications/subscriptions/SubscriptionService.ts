import { AxiosResponse } from 'axios';
import http, { UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type Subscription = definitions['Subscription'];
export type SubscriptionPost = definitions['SubscriptionPost'];

export type SubscriptionEnableToggle = {
  is_enabled: boolean;
};

export type Target = HttpTarget;

export type HttpTarget = {
  type: 'http';
  method: string;
  url: string;
  headers: Record<string, string>;
};

// Paperclip does not support anyOf at the moment so the resulting typescript type for SubscriptionPost.target is "string"
// Here is a temporary work-around
export interface SubscriptionFixed extends Omit<Subscription, 'target'> {
  target: Target;
}

// Paperclip does not support anyOf at the moment so the resulting typescript type for SubscriptionPost.target is "string"
// Here is a temporary work-around
export interface SubscriptionPostFixed extends Omit<SubscriptionPost, 'target'> {
  target: Target;
}

export function create(subscription: SubscriptionPostFixed): Promise<Subscription> {
  return http
    .post('/subscriptions', subscription)
    .then((res: AxiosResponse<Subscription>) => res.data);
}

export function remove(application_id: UUID, subscription_id: UUID): Promise<void> {
  return http
    .delete(`/subscriptions/${subscription_id}`, {
      params: {
        application_id,
      },
    })
    .then((res: AxiosResponse<void>) => res.data);
}

export function update(
  subscription_id: UUID,
  subscription: SubscriptionPostFixed | SubscriptionEnableToggle
): Promise<Subscription> {
  return http
    .put(`/subscriptions/${subscription_id}`, subscription)
    .then((res: AxiosResponse<Subscription>) => res.data);
}

export function toggleEnable(
  subscription_id: UUID,
  subscription: SubscriptionFixed
): Promise<Subscription> {
  return update(subscription_id, {
    application_id: subscription.application_id,

    event_types: subscription.event_types,
    target: subscription.target,

    is_enabled: !subscription.is_enabled,

    description: subscription.description,
    metadata: subscription.metadata,

    label_key: subscription.label_key,
    label_value: subscription.label_value,
  });
}

export function list(application_id: UUID): Promise<Array<Subscription>> {
  return http
    .get('/subscriptions', {
      params: {
        application_id: application_id,
      },
    })
    .then((res: AxiosResponse<Array<Subscription>>) => res.data);
}

export function get(id: UUID): Promise<Subscription> {
  return http.get(`/subscriptions/${id}`).then((res: AxiosResponse<Subscription>) => res.data);
}
