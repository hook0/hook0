import http from '@/http';
import type { components } from '@/types';
import { type CursorPage, PARAM_CURSOR, parseLinkHeader } from '@/utils/pagination';

export type HealthEvent = components['schemas']['HealthEvent'];

export function listHealthEvents(
  subscriptionId: string,
  organizationId: string,
  cursor: string | null
): Promise<CursorPage<HealthEvent>> {
  const params: Record<string, string> = { organization_id: organizationId };

  if (cursor) {
    params[PARAM_CURSOR] = cursor;
  }

  return http
    .get<HealthEvent[]>(`/subscriptions/${subscriptionId}/health_events`, {
      params,
    })
    .then((response) => {
      const links = parseLinkHeader(response.headers.link ?? null);

      return {
        data: response.data,
        nextCursor: links.next,
        prevCursor: links.prev,
      };
    });
}
