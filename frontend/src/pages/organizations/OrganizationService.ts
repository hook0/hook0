import {AxiosResponse} from 'axios';
import http, {UUID} from '../../http';
import {definitions} from '@/types';

export type Organization = definitions['Organization'];

export default {
  /*export function create(application: OrganizationPost) {
    return http.post('/organizations', application).then((res: AxiosResponse<any>) => res.data);
  }*/


  /*export function get(application_id: UUID): Promise<Organization> {
    return http.get(`/organizations/${application_id}`).then((res: AxiosResponse<any>) => res.data);
  }

  export function edit(application_id: UUID, body: OrganizationPost) {}

  export function remove(application_id: UUID) {}
  */
}

export function list(): Promise<Array<Organization>> {
  return http.get('/organizations').then((res: AxiosResponse<any>) => res.data);
}
