import { AxiosError, type AxiosResponse } from 'axios';
import http, { handleError, Problem, UUID } from '@/http';
import type { components } from '@/types';

type Definitions = components['schemas'];

export type EventsPerDayEntry = Definitions['EventsPerDayEntry'];

/** Fetch events-per-day from a given endpoint with params. */
function fetchEventsPerDay(
  endpoint: string,
  params: Record<string, string>
): Promise<EventsPerDayEntry[]> {
  return http.get<EventsPerDayEntry[]>(endpoint, { params }).then(
    (res) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

/** Fetch events-per-day aggregated across all apps of an organization. */
export function organization(
  organization_id: UUID,
  from: string,
  to: string
): Promise<EventsPerDayEntry[]> {
  return fetchEventsPerDay('/events_per_day/organization', { organization_id, from, to });
}

/** Fetch events-per-day for a single application. */
export function application(
  application_id: UUID,
  from: string,
  to: string
): Promise<EventsPerDayEntry[]> {
  return fetchEventsPerDay('/events_per_day/application', { application_id, from, to });
}
