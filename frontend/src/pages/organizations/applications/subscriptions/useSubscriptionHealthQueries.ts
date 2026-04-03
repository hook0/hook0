import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

/** Fetch health event timeline for a subscription — warns/disables/resolves over time */
export function useSubscriptionHealthEvents(
  subscriptionId: Ref<string>,
  organizationId: Ref<string>
) {
  return useQuery({
    queryKey: computed(() => healthEventKeys.list(subscriptionId.value)),
    queryFn: () =>
      SubscriptionHealthService.listHealthEvents(subscriptionId.value, organizationId.value),
    enabled: computed(() => !!subscriptionId.value && !!organizationId.value),
  });
}
