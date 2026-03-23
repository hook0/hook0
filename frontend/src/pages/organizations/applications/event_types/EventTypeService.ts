import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type EventType = definitions['EventType'];
export type EventTypePost = definitions['EventTypePost'];

export function create(event_type: EventTypePost): Promise<EventType> {
  return unwrapResponse(http.post<EventType>('/event_types', event_type));
}

export function deactivate(application_id: string, event_type_name: string): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/event_types/${event_type_name}`, {
      params: {
        application_id,
      },
    })
  );
}

export function list(application_id: UUID): Promise<Array<EventType>> {
  return unwrapResponse(
    http.get<Array<EventType>>('/event_types', {
      params: {
        application_id: application_id,
      },
    })
  );
}

export function get(id: UUID): Promise<EventType> {
  return unwrapResponse(http.get<EventType>(`/event_types/${id}`));
}
