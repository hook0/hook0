import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import { retryScheduleKeys } from '@/queries/keys';
import * as RetryScheduleService from './RetryScheduleService';
import type { RetryScheduleCreatePayload, RetrySchedulePayload } from './retrySchedule.types';

export function useRetryScheduleList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.list(organizationId.value)),
    queryFn: () => RetryScheduleService.listSchedules(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useRetryScheduleDetail(retryScheduleId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.detail(retryScheduleId.value)),
    queryFn: () => RetryScheduleService.getSchedule(retryScheduleId.value),
    enabled: computed(() => !!retryScheduleId.value),
  });
}

export function useCreateRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (payload: RetryScheduleCreatePayload) =>
      RetryScheduleService.createSchedule(payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
    },
  });
}

export function useUpdateRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { retryScheduleId: string; payload: RetrySchedulePayload }) =>
      RetryScheduleService.updateSchedule(args.retryScheduleId, args.payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
    },
  });
}

export function useDeleteRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (retryScheduleId: string) => RetryScheduleService.deleteSchedule(retryScheduleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
    },
  });
}
