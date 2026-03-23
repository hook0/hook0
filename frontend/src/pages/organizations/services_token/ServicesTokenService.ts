import type { components } from '@/types.ts';
import http, { UUID } from '@/http.ts';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type ServiceToken = definitions['ServiceToken'];
export type ServiceTokenPost = definitions['ServiceTokenPost'];

export function create(service_token: ServiceTokenPost): Promise<ServiceToken> {
  return unwrapResponse(http.post<ServiceToken>('/service_token', service_token));
}

export function list(organization_id: UUID): Promise<Array<ServiceToken>> {
  return unwrapResponse(
    http.get<Array<ServiceToken>>('/service_token', {
      params: {
        organization_id: organization_id,
      },
    })
  );
}

export function update(
  service_token_id: UUID,
  service_token: ServiceTokenPost
): Promise<ServiceToken> {
  return unwrapResponse(
    http.put<ServiceToken>(`/service_token/${service_token_id}`, service_token)
  );
}

export function remove(service_token_id: UUID, organization_id: UUID): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/service_token/${service_token_id}`, {
      params: {
        organization_id,
      },
    })
  );
}

export function get(service_token_id: UUID, organization_id: UUID): Promise<ServiceToken> {
  return unwrapResponse(
    http.get<ServiceToken>(`/service_token/${service_token_id}`, {
      params: {
        organization_id,
      },
    })
  );
}
