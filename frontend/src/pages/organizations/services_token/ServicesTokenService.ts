import type { components } from '@/types.ts';
import { AxiosError, AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http.ts';

type definitions = components['schemas'];

export type ServiceToken = definitions['ServiceToken'];
export type ServiceTokenPost = definitions['ServiceTokenPost'];

export function create(service_token: ServiceTokenPost): Promise<ServiceToken> {
  return http.post('/service_token', service_token).then(
    (res: AxiosResponse<ServiceToken>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function list(organization_id: UUID): Promise<Array<ServiceToken>> {
  return http
    .get('/service_token', {
      params: {
        organization_id: organization_id,
      },
    })
    .then(
      (res: AxiosResponse<Array<ServiceToken>>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function update(
  service_token_id: UUID,
  service_token: ServiceTokenPost
): Promise<ServiceToken> {
  return http.put(`/service_token/${service_token_id}`, service_token).then(
    (res: AxiosResponse<ServiceToken>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function remove(service_token_id: UUID, organization_id: UUID): Promise<void> {
  return http
    .delete(`/service_token/${service_token_id}`, {
      params: {
        organization_id,
      },
    })
    .then(
      (res: AxiosResponse<void>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function get(service_token_id: UUID): Promise<ServiceToken> {
  return http.get(`/service_token/${service_token_id}`).then(
    (res: AxiosResponse<ServiceToken>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}