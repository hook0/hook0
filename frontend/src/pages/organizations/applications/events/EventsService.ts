import {AxiosResponse} from 'axios';
import http, {UUID} from '@/http';
import type {components} from '@/types';

type definitions = components['schemas'];


export type Event = definitions['Event'];
export type EventWithPayload = definitions['EventWithPayload'];

export type EventWithPayloadDecoded = {
    payload_decoded: string
}

const enum PayloadContentType {
    Text = "text/plain",
    Json = "application/json",
    Binary = "application/octet-stream+base64",
}

function decode(payload: string, payload_content_type_name: string) {

    switch (payload_content_type_name) {
        case PayloadContentType.Text:
            return atob(payload);
        case PayloadContentType.Json:
            return atob(payload);
        case PayloadContentType.Binary:
        default:
            return payload;
    }
}

export function get(id: UUID, application_id: UUID): Promise<EventWithPayload & EventWithPayloadDecoded> {
    return http.get(`/events/${id}`, {
        params: {
            application_id: application_id,
        }
    }).then((res: AxiosResponse<EventWithPayload>) => {
        return {
            ...res.data,
            payload_decoded: decode(res.data.payload, res.data.payload_content_type)
        };
    });
}

export function list(application_id: UUID): Promise<Array<Event>> {
    return http.get('/events', {
        params: {
            application_id: application_id,
        },
    }).then((res: AxiosResponse<Array<Event>>) => res.data);
}
