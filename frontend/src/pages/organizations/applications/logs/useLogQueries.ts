import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as LogService from './LogService';
import { logKeys } from '@/queries/keys';

export function useLogList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => logKeys.list(applicationId.value)),
    queryFn: () => LogService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

/** Fetch deliveries scoped to a single subscription — powers the subscription detail page's delivery table */
export function useLogListBySubscription(
  applicationId: Ref<string>,
  subscriptionId: Ref<string>
) {
  return useQuery({
    queryKey: computed(() =>
      logKeys.bySubscription(applicationId.value, subscriptionId.value)
    ),
    queryFn: () =>
      LogService.listBySubscription(applicationId.value, subscriptionId.value),
    enabled: computed(() => !!applicationId.value && !!subscriptionId.value),
  });
}
