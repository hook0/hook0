import { useMutation, useQueryClient, useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as RetryScheduleService from './RetryScheduleService';
import type { RetrySchedulePost, RetrySchedulePut } from './RetryScheduleService';
import { retryScheduleKeys, subscriptionKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useRetryScheduleList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.list(organizationId.value)),
    queryFn: () => RetryScheduleService.list(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useRetryScheduleDetail(id: Ref<string>, organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.detail(id.value, organizationId.value)),
    queryFn: () => RetryScheduleService.get(id.value, organizationId.value),
    enabled: computed(() => !!id.value && !!organizationId.value),
  });
}

export function useCreateRetrySchedule() {
  return useInvalidatingMutation({
    mutationFn: (schedule: RetrySchedulePost) => RetryScheduleService.create(schedule),
    invalidateKeys: retryScheduleKeys.all,
  });
}

export function useUpdateRetrySchedule() {
  return useInvalidatingMutation({
    mutationFn: (params: {
      retryScheduleId: string;
      organizationId: string;
      schedule: RetrySchedulePut;
    }) =>
      RetryScheduleService.update(params.retryScheduleId, params.organizationId, params.schedule),
    invalidateKeys: retryScheduleKeys.all,
  });
}

export function useRemoveRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { retryScheduleId: string; organizationId: string }) =>
      RetryScheduleService.remove(params.retryScheduleId, params.organizationId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
      void queryClient.invalidateQueries({ queryKey: subscriptionKeys.all });
    },
  });
}
