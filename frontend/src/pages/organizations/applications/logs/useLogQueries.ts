import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as LogService from './LogService';
import { logKeys, requestAttemptKeys } from '@/queries/keys';

export function useLogList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => logKeys.list(applicationId.value)),
    queryFn: () => LogService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}


/** Fetch deliveries scoped to a single subscription — powers the subscription detail page's delivery table */
export function useLogListBySubscription(applicationId: Ref<string>, subscriptionId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => logKeys.bySubscription(applicationId.value, subscriptionId.value)),
    queryFn: () => LogService.listBySubscription(applicationId.value, subscriptionId.value),
    enabled: computed(() => !!applicationId.value && !!subscriptionId.value),
  });
}

export function useRetryDelivery() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (requestAttemptId: string) => LogService.retry(requestAttemptId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: logKeys.all });
    },
  });
}

export function useRequestAttemptDetail(requestAttemptId: Ref<string>, applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() =>
      requestAttemptKeys.detail(requestAttemptId.value, applicationId.value)
    ),
    queryFn: () => LogService.getById(requestAttemptId.value, applicationId.value),
    enabled: computed(() => !!requestAttemptId.value && !!applicationId.value),
  });
}
