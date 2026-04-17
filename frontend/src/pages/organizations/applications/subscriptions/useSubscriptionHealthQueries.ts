import { keepPreviousData, useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import type { PaginationStep } from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

const STALE_TIME_MS = 30_000;

export function useSubscriptionHealthEvents(
  subscriptionId: Ref<string>,
  organizationId: Ref<string>,
  pagination: Ref<PaginationStep>
) {
  return useQuery({
    queryKey: computed(() => [
      ...healthEventKeys.list(subscriptionId.value, organizationId.value),
      pagination.value,
    ]),
    queryFn: () =>
      SubscriptionHealthService.listHealthEvents(
        subscriptionId.value,
        organizationId.value,
        pagination.value
      ),
    enabled: computed(() => !!subscriptionId.value && !!organizationId.value),
    placeholderData: keepPreviousData,
    staleTime: STALE_TIME_MS,
  });
}
