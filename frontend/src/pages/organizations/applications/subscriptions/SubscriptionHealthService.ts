import http from '@/http';
import type { components, operations } from '@/types';

export type HealthTimeline = components['schemas']['HealthTimeline'];
export type HealthEvent = HealthTimeline['events'][number];
export type HealthBucket = HealthTimeline['buckets'][number];

export type HealthWindow = NonNullable<
  operations['subscriptionHealthEvents.list']['parameters']['query']
>['window'];
export const HEALTH_WINDOWS = ['24h', '7d', '30d'] as const satisfies readonly HealthWindow[];

export function getHealthTimeline(
  subscriptionId: string,
  window: HealthWindow
): Promise<HealthTimeline> {
  return http
    .get<HealthTimeline>(`/subscriptions/${subscriptionId}/health_events`, {
      params: { window },
    })
    .then((response) => response.data);
}
