import {AxiosResponse} from 'axios';
import http, {UUID} from '../../http';
import {definitions} from '@/types';

export type Application = definitions['Application'];
export type ApplicationPost = definitions['ApplicationPost'];


export function create(application: ApplicationPost): Promise<Application> {
  return http.post('/applications', application).then((res: AxiosResponse<Application>) => res.data);
}

export function list(organization_id: UUID): Promise<Array<Application>> {
  return http.get('/applications', {
    params: {
      organization_id: organization_id
    }
  }).then((res: AxiosResponse<Array<Application>>) => res.data);
}

export function get(application_id: UUID): Promise<Application> {
  return http.get(`/applications/${application_id}`).then((res: AxiosResponse<Application>) => res.data);
}

// export function edit(application_id: UUID, body: ApplicationPost) {
// }

// export function remove(application_id: UUID) {
// }
