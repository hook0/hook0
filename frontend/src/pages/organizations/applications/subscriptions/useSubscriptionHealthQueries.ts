import { keepPreviousData, useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

const STALE_TIME_MS = 30_000;

export function useSubscriptionHealthEvents(
  subscriptionId: Ref<string>,
  organizationId: Ref<string>,
  cursor: Ref<string | null>
) {
  return useQuery({
    queryKey: computed(() => [
      ...healthEventKeys.list(subscriptionId.value, organizationId.value),
      cursor.value,
    ]),
    queryFn: () =>
      SubscriptionHealthService.listHealthEvents(
        subscriptionId.value,
        organizationId.value,
        cursor.value
      ),
    enabled: computed(() => !!subscriptionId.value && !!organizationId.value),
    placeholderData: keepPreviousData,
    staleTime: STALE_TIME_MS,
  });
}
