import http, { UUID } from '@/http';
import type { components } from '@/types';
import { formatBody } from './formatBody';

type definitions = components['schemas'];

export type Response = definitions['Response'];

export type ResponseWithFormattedBody = Response & {
  body_formatted: string | null;
};

export function get(responseId: UUID, applicationId: UUID): Promise<ResponseWithFormattedBody> {
  return http
    .get<Response>(`/responses/${responseId}`, {
      params: {
        application_id: applicationId,
      },
    })
    .then((res) => ({
      ...res.data,
      body_formatted: formatBody(res.data.body),
    }));
}
