import http from '@/http';
import type { components } from '@/types';
import {
  type CursorPage,
  type PaginationDirection,
  PARAM_CURSOR,
  PARAM_DIRECTION,
  parseLinkHeader,
} from '@/utils/pagination';

export type HealthEvent = components['schemas']['HealthEvent'];

export type PaginationStep = { cursor: string; direction: PaginationDirection } | null;

export function listHealthEvents(
  subscriptionId: string,
  organizationId: string,
  pagination: PaginationStep
): Promise<CursorPage<HealthEvent>> {
  const params: Record<string, string> = { organization_id: organizationId };

  if (pagination) {
    params[PARAM_CURSOR] = pagination.cursor;
    params[PARAM_DIRECTION] = pagination.direction;
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
