import http, { UUID } from '@/http';
import type { components } from '@/types';

type definitions = components['schemas'];

export type Response = definitions['Response'];

export type ResponseWithFormattedBody = Response & {
  body_formatted: string | null;
};

function formatBody(body: string | undefined): string | null {
  if (!body) return null;
  try {
    return JSON.stringify(JSON.parse(body), null, 4);
  } catch {
    return body;
  }
}

export function get(responseId: UUID, applicationId: UUID): Promise<ResponseWithFormattedBody> {
  return http
    .get<Response>(`/responses/${responseId}`, {
      params: {
        application_id: applicationId,
      },
    })
    .then((res) => {
      return {
        ...res.data,
        body_formatted: formatBody(res.data.body),
      };
    });
}
