import { AxiosError, AxiosResponse } from 'axios';
import http, { Problem, UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type Organization = definitions['OrganizationInfo'];
export type User = definitions['OrganizationInfo']['users'][number];
export type Invitation = definitions['UserInvitation'];

export type Members = {
  max: number;
  members: User[];
};

export async function get(organization_id: UUID): Promise<Members> {
  const org = await http.get(`/organizations/${organization_id}`, {}).then(
    (res: AxiosResponse<Organization>) => res.data,
    (err: AxiosError<AxiosResponse<Problem, Problem>>) => Promise.reject(err.response?.data)
  );

  return {
    max: org.quotas.members_per_organization_limit,
    members: org.users,
  };
}

export async function invite(organization_id: UUID, invitation: Invitation): Promise<void> {
  return http.put(`/organizations/${organization_id}/invite`, invitation).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem, Problem>>) => Promise.reject(err.response?.data)
  );
}

export function revoke(organization_id: UUID, user_id: UUID): Promise<void> {
  return http.delete(`/organizations/${organization_id}/invite`, { data: { user_id } }).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem, Problem>>) => Promise.reject(err.response?.data)
  );
}
