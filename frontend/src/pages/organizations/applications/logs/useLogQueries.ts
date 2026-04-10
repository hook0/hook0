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

export function useRetryDelivery(applicationId: Ref<string>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (requestAttemptId: string) =>
      LogService.retry(requestAttemptId, applicationId.value),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: logKeys.lists(),
      });
      void queryClient.invalidateQueries({
        queryKey: requestAttemptKeys.all,
      });
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
