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

export type PaginationStep = { cursor: string; direction: PaginationDirection } | null;

export function listHealthEvents(
  subscriptionId: string,
  organizationId: string,
  pagination: PaginationStep
): Promise<CursorPage<HealthEvent>> {
  const params: Record<string, string> = { organization_id: organizationId };

  if (pagination) {
    const paramName = pagination.direction === 'backward' ? PARAM_PREV_CURSOR : PARAM_NEXT_CURSOR;
    params[paramName] = pagination.cursor;
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
