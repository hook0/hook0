import http from '@/http';
import {
  type CursorPage,
  type PaginationDirection,
  PARAM_NEXT_CURSOR,
  PARAM_PREV_CURSOR,
  parseLinkHeader,
} from '@/utils/pagination';

// TODO: replace with OpenAPI generated type when API spec is regenerated
export type HealthEvent = {
  health_event_id: string;
  subscription_id: string;
  status: 'warning' | 'disabled' | 'resolved';
  cause: 'auto' | 'manual';
  user_id: string | null;
  created_at: string;
};

export function listHealthEvents(
  subscriptionId: string,
  organizationId: string,
  cursor?: string | null,
  direction: PaginationDirection = 'forward'
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
