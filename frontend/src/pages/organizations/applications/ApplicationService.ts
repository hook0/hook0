import { AxiosError, AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type Application = definitions['Application'];
export type ApplicationPost = definitions['ApplicationPost'];

export function create(application: ApplicationPost): Promise<Application> {
  return http.post('/applications', application).then(
    (res: AxiosResponse<Application>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function list(organization_id: UUID): Promise<Array<Application>> {
  return http
    .get('/applications', {
      params: {
        organization_id: organization_id,
      },
    })
    .then(
      (res: AxiosResponse<Array<Application>>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function get(application_id: UUID): Promise<Application> {
  return http.get(`/applications/${application_id}`).then(
    (res: AxiosResponse<Application>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function update(application_id: UUID, application: ApplicationPost): Promise<Application> {
  return http.put(`/applications/${application_id}`, application).then(
    (res: AxiosResponse<Application>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function remove(application_id: UUID): Promise<void> {
  return http.delete(`/applications/${application_id}`, {}).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
