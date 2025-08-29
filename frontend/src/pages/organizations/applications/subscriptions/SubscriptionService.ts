import { AxiosError, AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type Subscription = definitions['Subscription'];
export type SubscriptionPost = definitions['SubscriptionPost'];

export type SubscriptionEnableToggle = {
  is_enabled: boolean;
};

export function create(subscription: SubscriptionPost): Promise<Subscription> {
  return http.post('/subscriptions', subscription).then(
    (res: AxiosResponse<Subscription>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function remove(application_id: UUID, subscription_id: UUID): Promise<void> {
  return http
    .delete(`/subscriptions/${subscription_id}`, {
      params: {
        application_id,
      },
    })
    .then(
      (res: AxiosResponse<void>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function update(
  subscription_id: UUID,
  subscription: SubscriptionPost | SubscriptionEnableToggle
): Promise<Subscription> {
  return http.put(`/subscriptions/${subscription_id}`, subscription).then(
    (res: AxiosResponse<Subscription>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
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
    retry_config: subscription.retry_config,
  });
}

export function list(application_id: UUID): Promise<Array<Subscription>> {
  return http
    .get('/subscriptions', {
      params: {
        application_id: application_id,
      },
    })
    .then(
      (res: AxiosResponse<Array<Subscription>>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function get(id: UUID): Promise<Subscription> {
  return http.get(`/subscriptions/${id}`).then(
    (res: AxiosResponse<Subscription>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
