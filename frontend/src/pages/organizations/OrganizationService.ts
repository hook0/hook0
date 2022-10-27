import {AxiosResponse} from 'axios';
import http, {UUID} from '@/http';
import type {components} from '@/types';

type definitions = components['schemas'];
import {onTokenExpired} from "@/iam";

export type Organization = definitions['Organization'];
export type OrganizationPost = definitions['OrganizationPost'];
export type OrganizationInfo = definitions['OrganizationInfo'];

type Problem = definitions['Problem'];

export function create(organization: OrganizationPost): Promise<OrganizationInfo> {
    return http
        .post('/organizations', organization)
        .then((res: AxiosResponse<OrganizationInfo>) => res.data)
        .then((organization) => {
            // we currently have to force the JWT refresh so it contains the organization
            return onTokenExpired().then(() => organization);
        })
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

export function remove(organization_id: UUID): Promise<void> {
    return http.delete(`/organizations/${organization_id}`, {}).then((res: AxiosResponse<void>) => res.data);
}
