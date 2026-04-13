import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

/**
 * Fetch one page of the health event timeline for a subscription.
 *
 * Cursor-based: pass a `cursor` ref — `null` means the first page, any other
 * value is an opaque token returned by the previous page's `next_cursor`.
 * The query key includes the cursor so changing pages triggers a refetch.
 */
export function useSubscriptionHealthEvents(
  subscriptionId: Ref<string>,
  organizationId: Ref<string>,
  cursor: Ref<string | null>
) {
  return useQuery({
    queryKey: computed(() =>
      healthEventKeys.list(subscriptionId.value, organizationId.value, cursor.value)
    ),
    queryFn: () =>
      SubscriptionHealthService.listHealthEvents(subscriptionId.value, organizationId.value, {
        cursor: cursor.value,
      }),
    enabled: computed(() => !!subscriptionId.value && !!organizationId.value),
  });
}
