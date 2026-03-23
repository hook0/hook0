import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type Event = definitions['Event'];
export type EventWithPayload = definitions['EventWithPayload'];

export type EventWithPayloadDecoded = {
  payload_decoded: string;
};

const enum PayloadContentType {
  Text = 'text/plain',
  Json = 'application/json',
  Binary = 'application/octet-stream+base64',
}

function decode(payload: string, payload_content_type_name: string): string {
  switch (payload_content_type_name) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-enum-comparison
    case PayloadContentType.Text:
      return atob(payload);
    // eslint-disable-next-line @typescript-eslint/no-unsafe-enum-comparison
    case PayloadContentType.Json:
      return JSON.stringify(JSON.parse(atob(payload)), null, 4);
    // eslint-disable-next-line @typescript-eslint/no-unsafe-enum-comparison
    case PayloadContentType.Binary:
    default:
      return payload;
  }
}

export function get(
  id: UUID,
  application_id: UUID
): Promise<EventWithPayload & EventWithPayloadDecoded> {
  return http
    .get<EventWithPayload>(`/events/${id}`, {
      params: {
        application_id: application_id,
      },
    })
    .then((res) => {
      return {
        ...res.data,
        payload_decoded: decode(res.data.payload, res.data.payload_content_type),
      };
    });
}

export function list(application_id: UUID): Promise<Array<Event>> {
  return unwrapResponse(
    http.get<Array<Event>>('/events', {
      params: {
        application_id: application_id,
      },
    })
  );
}

export function send_json_event(
  application_id: UUID,
  event_id: UUID,
  event_type: string,
  labels: { [key: string]: string },
  occurred_at: Date,
  payload: string
): Promise<void> {
  const occurred_at_string = new Date(occurred_at).toISOString();

  return unwrapResponse(
    http.post<void>('/event', {
      application_id,
      event_id,
      event_type,
      labels,
      occurred_at: occurred_at_string,
      payload_content_type: 'application/json',
      payload,
    })
  );
}

export function replay(event_id: UUID, application_id: UUID): Promise<void> {
  return unwrapResponse(
    http.post<void>(`/events/${event_id}/replay`, {
      application_id,
    })
  );
}
