import { AxiosError, AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type EventType = definitions['EventType'];
export type EventTypePost = definitions['EventTypePost'];

export function create(event_type: EventTypePost): Promise<EventType> {
  return http.post('/event_types', event_type).then(
    (res: AxiosResponse<EventType>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function remove(application_id: string, event_type_name: string): Promise<void> {
  return http
    .delete(`/event_types/${event_type_name}`, {
      params: {
        application_id,
      },
    })
    .then(
      (res: AxiosResponse<void>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function list(application_id: UUID): Promise<Array<EventType>> {
  return http
    .get('/event_types', {
      params: {
        application_id: application_id,
      },
    })
    .then(
      (res: AxiosResponse<Array<EventType>>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function get(id: UUID): Promise<EventType> {
  return http.get(`/event_types/${id}`).then(
    (res: AxiosResponse<EventType>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
