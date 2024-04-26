import { AxiosError, AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];
import { refresh } from '@/iam';

export type Organization = definitions['Organization'];
export type OrganizationPost = definitions['OrganizationPost'];
export type OrganizationInfo = definitions['OrganizationInfo'];

export function create(organization: OrganizationPost): Promise<OrganizationInfo> {
  return http
    .post('/organizations', organization)
    .then(
      (res: AxiosResponse<OrganizationInfo>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    )
    .then((organization) => {
      // we currently have to force the JWT refresh so it contains the organization
      return refresh().then(() => organization);
    });
}

export function list(): Promise<Array<Organization>> {
  return http.get('/organizations', {}).then(
    (res: AxiosResponse<Array<Organization>>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function get(id: UUID): Promise<OrganizationInfo> {
  return http.get(`/organizations/${id}`).then(
    (res: AxiosResponse<OrganizationInfo>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function update(
  organization_id: UUID,
  organization: OrganizationPost
): Promise<Organization> {
  return http
    .put(`/organizations/${organization_id}`, organization)
    .then(
      (res: AxiosResponse<Organization>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    )
    .then((organization) => {
      // we currently have to force the JWT refresh so it contains the organization
      return refresh().then(() => organization);
    });
}

export function remove(organization_id: UUID): Promise<void> {
  return http
    .delete(`/organizations/${organization_id}`, {})
    .then(
      (res: AxiosResponse<void>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    )
    .then((organization) => {
      // we currently have to force the JWT refresh so it contains the organization
      return refresh().then(() => organization);
    });
}
