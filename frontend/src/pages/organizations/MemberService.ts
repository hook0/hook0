import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type Organization = definitions['OrganizationInfo'];
export type User = definitions['OrganizationInfo']['users'][number];
export type Invitation = definitions['UserInvitation'];

export type Members = {
  max: number;
  members: User[];
};

export function get(organization_id: UUID): Promise<Members> {
  return unwrapResponse(http.get<Organization>(`/organizations/${organization_id}`, {})).then(
    (org) => ({
      max: org.quotas.members_per_organization_limit,
      members: org.users,
    })
  );
}

export function invite(organization_id: UUID, invitation: Invitation): Promise<void> {
  return unwrapResponse(http.post<void>(`/organizations/${organization_id}/invite`, invitation));
}

export function revoke(organization_id: UUID, user_id: UUID): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/organizations/${organization_id}/invite`, { data: { user_id } })
  );
}

export function edit_role(organization_id: UUID, user_id: UUID, role: string): Promise<void> {
  return unwrapResponse(
    http.put<void>(`/organizations/${organization_id}/invite`, { user_id, role })
  );
}
