import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];
import { useAuthStore } from '@/stores/auth';

export type Organization = definitions['Organization'];
export type OrganizationPost = definitions['OrganizationPost'];
export type OrganizationInfo = definitions['OrganizationInfo'];

export function create(organization: OrganizationPost): Promise<OrganizationInfo> {
  return unwrapResponse(http.post<OrganizationInfo>('/organizations', organization)).then(
    (organization) => {
      // we currently have to force the user token refresh so it contains the organization
      return useAuthStore()
        .refresh()
        .then(() => organization);
    }
  );
}

export function list(): Promise<Array<Organization>> {
  return unwrapResponse(http.get<Array<Organization>>('/organizations', {}));
}

export function get(id: UUID): Promise<OrganizationInfo> {
  return unwrapResponse(http.get<OrganizationInfo>(`/organizations/${id}`));
}

export function update(
  organization_id: UUID,
  organization: OrganizationPost
): Promise<Organization> {
  return unwrapResponse(
    http.put<Organization>(`/organizations/${organization_id}`, organization)
  ).then((organization) => {
    // we currently have to force the user token refresh so it contains the organization
    return useAuthStore()
      .refresh()
      .then(() => organization);
  });
}

export function remove(organization_id: UUID): Promise<void> {
  return unwrapResponse(http.delete<void>(`/organizations/${organization_id}`, {})).then(
    (organization) => {
      // we currently have to force the user token refresh so it contains the organization
      return useAuthStore()
        .refresh()
        .then(() => organization);
    }
  );
}
