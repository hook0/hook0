import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import type { PaginationDirection } from '@/utils/pagination';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

export function useSubscriptionHealthEvents(
  subscriptionId: Ref<string>,
  organizationId: Ref<string>,
  cursor: Ref<string | null>,
  direction: Ref<PaginationDirection>
) {
  return useQuery({
    queryKey: computed(() => [
      ...healthEventKeys.list(subscriptionId.value, organizationId.value),
      cursor.value,
      direction.value,
    ]),
    queryFn: () =>
      SubscriptionHealthService.listHealthEvents(
        subscriptionId.value,
        organizationId.value,
        cursor.value,
        direction.value
      ),
    enabled: computed(() => !!subscriptionId.value && !!organizationId.value),
  });
}
