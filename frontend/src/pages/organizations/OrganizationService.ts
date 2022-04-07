import { AxiosResponse } from 'axios';
import http, { UUID } from '../../http';
import { definitions } from '@/types';

export type Organization = definitions['Organization'];
export type OrganizationPost = definitions['OrganizationnPost'];

type Problem = definitions['Problem'];

export function create(organization: OrganizationPost): Promise<Organization> {
  return http
    .post('/organizations', organization)
    .then((res: AxiosResponse<Organization>) => res.data);
}

export function list(): Promise<Array<Organization>> {
  return http.get('/organizations', {}).then((res: AxiosResponse<Array<Organization>>) => res.data);
}

export function get(id: UUID): Promise<Organization> {
  return http.get(`/organizations/${id}`).then((res: AxiosResponse<Organization>) => res.data);
}

export function update(
  organization_id: UUID,
  organization: OrganizationPost
): Promise<Organization> {
  return http
    .put(`/organizations/${organization_id}`, organization)
    .then((res: AxiosResponse<Organization>) => res.data);
}
