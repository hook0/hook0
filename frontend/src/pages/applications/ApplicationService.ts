import {AxiosResponse} from 'axios';
import http, {UUID} from '../../http';
import {definitions} from '@/types';

type Application = definitions['Application'];
type ApplicationPost = definitions['ApplicationPost'];

export default {
  create(application: ApplicationPost) {
    return http.post('/applications', application).then((res: AxiosResponse<any>) => res.data);
  },

  list(organization_id: UUID): Promise<Array<Application>> {
    return http.get('/applications', {
      params:{
        organization_id: organization_id
      }
    }).then((res: AxiosResponse<any>) => res.data);
  },

  get(application_id: UUID): Promise<Application> {
    return http.get(`/applications/${application_id}`).then((res: AxiosResponse<any>) => res.data);
  },

  edit(application_id: UUID, body: ApplicationPost) {
  },

  remove(application_id: UUID) {
  }
}
