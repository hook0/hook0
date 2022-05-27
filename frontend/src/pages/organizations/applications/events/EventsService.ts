import {AxiosResponse} from 'axios';
import http, {UUID} from '@/http';
import {definitions} from '@/types';


export type Event = definitions['Event'];
export type EventWithPayload = definitions['EventWithPayload'];

export function get(id: UUID, application_id: UUID): Promise<EventWithPayload> {
  return http.get(`/events/${id}`, {
    params: {
      application_id: application_id,
    }
  }).then((res: AxiosResponse<EventWithPayload>) => res.data);
}

export function list(application_id: UUID): Promise<Array<Event>> {
  return http.get('/events', {
    params: {
      application_id: application_id,
    },
  }).then((res: AxiosResponse<Array<Event>>) => res.data);
}
