// HTTP service layer for subscription health events — timeline of status transitions (warning, disabled, resolved).
import http, { type UUID } from '@/http';
import { unwrapResponse } from '@/utils/unwrapResponse';

export type HealthEvent = {
  health_event_id: string;
  subscription_id: string;
  status: 'warning' | 'disabled' | 'resolved';
  source: 'system' | 'user';
  user_id: string | null;
  created_at: string;
};

/** Fetch the health event timeline for a subscription — ordered by created_at desc */
export function listHealthEvents(
  subscriptionId: UUID,
  organizationId: UUID
): Promise<HealthEvent[]> {
  return unwrapResponse(
    http.get<HealthEvent[]>(`/subscriptions/${subscriptionId}/health_events`, {
      params: { organization_id: organizationId },
    })
  );
}
