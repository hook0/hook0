// HTTP service layer for subscription health events — timeline of status transitions (warning, disabled, resolved).
import http, { type UUID } from '@/http';
import { handleError, type Problem } from '@/http';
import type { AxiosError, AxiosResponse } from 'axios';

export type HealthEvent = {
  health_event_id: string;
  subscription_id: string;
  status: 'warning' | 'disabled' | 'resolved';
  cause: 'auto' | 'manual';
  user_id: string | null;
  created_at: string;
};

/**
 * A single page of health events.
 *
 * The backend uses cursor-based pagination: it returns the events as a JSON
 * array and the "next page" URL in a `Link: <url>; rel="next"` HTTP header.
 * This service unwraps that header into an explicit `next_cursor` field so
 * callers don't have to juggle axios response objects.
 */
export type HealthEventsPage = {
  items: HealthEvent[];
  next_cursor: string | null;
};

/** Extract a cursor token from the `Link: <url>; rel="next"` HTTP response header. */
function extractNextCursor(linkHeader: string | undefined | null): string | null {
  if (!linkHeader) return null;
  // Expected shape: `<https://.../health_events?organization_id=...&pagination_cursor=XYZ>; rel="next"`
  const match = /<([^>]+)>\s*;\s*rel="next"/.exec(linkHeader);
  if (!match) return null;
  const url = match[1];
  try {
    const parsed = new URL(url);
    return parsed.searchParams.get('pagination_cursor');
  } catch {
    return null;
  }
}

/** Fetch one page of the health event timeline for a subscription — ordered by created_at desc. */
export function listHealthEvents(
  subscriptionId: UUID,
  organizationId: UUID,
  params: { cursor?: string | null } = {}
): Promise<HealthEventsPage> {
  const queryParams: Record<string, string> = {
    organization_id: organizationId,
  };
  if (params.cursor != null && params.cursor !== '') {
    queryParams.pagination_cursor = params.cursor;
  }

  return http
    .get<HealthEvent[]>(`/subscriptions/${subscriptionId}/health_events`, {
      params: queryParams,
    })
    .then(
      (res: AxiosResponse<HealthEvent[]>) => {
        const linkHeader =
          (res.headers?.link as string | undefined) ??
          (res.headers?.Link as string | undefined) ??
          null;
        return {
          items: res.data,
          next_cursor: extractNextCursor(linkHeader),
        };
      },
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}
