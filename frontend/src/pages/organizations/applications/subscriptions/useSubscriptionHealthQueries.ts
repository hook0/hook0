import { keepPreviousData, useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import type { HealthWindow } from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

const STALE_TIME_MS = 30_000;

export function useSubscriptionHealthTimeline(
  subscriptionId: Ref<string>,
  window: Ref<HealthWindow>
) {
  return useQuery({
    queryKey: computed(() => healthEventKeys.timeline(subscriptionId.value, window.value)),
    queryFn: () => SubscriptionHealthService.getHealthTimeline(subscriptionId.value, window.value),
    enabled: computed(() => !!subscriptionId.value),
    placeholderData: keepPreviousData,
    staleTime: STALE_TIME_MS,
  });
}
