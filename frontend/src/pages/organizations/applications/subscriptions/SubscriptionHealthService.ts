import http from '@/http';
import type { components } from '@/types';
import {
  type CursorPage,
  type PaginationDirection,
  PARAM_NEXT_CURSOR,
  PARAM_PREV_CURSOR,
  parseLinkHeader,
} from '@/utils/pagination';

export type HealthEvent = components['schemas']['HealthEvent'];

export function listHealthEvents(
  subscriptionId: string,
  organizationId: string,
  cursor: string | null,
  direction: PaginationDirection
): Promise<CursorPage<HealthEvent>> {
  const params: Record<string, string> = { organization_id: organizationId };

  if (cursor) {
    const paramName = direction === 'backward' ? PARAM_PREV_CURSOR : PARAM_NEXT_CURSOR;
    params[paramName] = cursor;
  }

  return http
    .get<HealthEvent[]>(`/subscriptions/${subscriptionId}/health_events`, {
      params,
    })
    .then((response) => {
      const links = parseLinkHeader(response.headers?.link ?? null);

      return {
        data: response.data,
        nextCursor: links.next,
        prevCursor: links.prev,
      };
    });
}
