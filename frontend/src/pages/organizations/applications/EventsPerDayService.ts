import { AxiosError, AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type EventsPerDayEntry = definitions['EventsPerDayEntry'];

export function application(
  application_id: UUID,
  from: string,
  to: string
): Promise<Array<EventsPerDayEntry>> {
  return http
    .get('/events_per_day/application', {
      params: {
        application_id,
        from,
        to,
      },
    })
    .then(
      (res: AxiosResponse<Array<EventsPerDayEntry>>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function organization(
  organization_id: UUID,
  from: string,
  to: string
): Promise<Array<EventsPerDayEntry>> {
  return http
    .get('/events_per_day/organization', {
      params: {
        organization_id,
        from,
        to,
      },
    })
    .then(
      (res: AxiosResponse<Array<EventsPerDayEntry>>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}
