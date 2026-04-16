import http from '@/http';

export type HealthEvent = {
  health_event_id: string;
  subscription_id: string;
  status: 'warning' | 'disabled' | 'resolved';
  cause: 'auto' | 'manual';
  user_id: string | null;
  created_at: string;
};

export type HealthEventPage = {
  data: HealthEvent[];
  nextCursor: string | null;
  prevCursor: string | null;
};

/**
 * Safely parses the Link header to extract pagination cursors.
 * Uses a dummy base URL so relative URLs don't crash the parser.
 */
function parseLinkHeader(linkHeader: string | null): {
  next: string | null;
  prev: string | null;
} {
  const result = { next: null as string | null, prev: null as string | null };

  if (!linkHeader) {
    return result;
  }

  for (const part of linkHeader.split(',')) {
    const match = part.match(/<([^>]+)>;\s*rel="(\w+)"/);
    if (!match) continue;

    const [, urlString, rel] = match;

    try {
      // Dummy base ensures relative URLs don't throw
      const parsedUrl = new URL(urlString, 'http://x');

      if (rel === 'next') {
        result.next = parsedUrl.searchParams.get('pagination_cursor');
      } else if (rel === 'prev') {
        result.prev = parsedUrl.searchParams.get('pagination_before_cursor');
      }
    } catch (e) {
      // Ignore malformed link rather than crash the app
      console.error('Failed to parse link header URL', e);
    }
  }

  return result;
}

export function listHealthEvents(
  subscriptionId: string,
  organizationId: string,
  cursor?: string | null,
  direction: 'forward' | 'backward' = 'forward'
): Promise<HealthEventPage> {
  const params: Record<string, string> = { organization_id: organizationId };

  if (cursor) {
    if (direction === 'backward') {
      params.pagination_before_cursor = cursor;
    } else {
      params.pagination_cursor = cursor;
    }
  }

  return http
    .get<HealthEvent[]>(`/subscriptions/${subscriptionId}/health_events`, {
      params,
    })
    .then((response) => {
      const linkHeader = response.headers?.link ?? null;
      const links = parseLinkHeader(linkHeader);

      return {
        data: response.data,
        nextCursor: links.next,
        prevCursor: links.prev,
      };
    });
}
