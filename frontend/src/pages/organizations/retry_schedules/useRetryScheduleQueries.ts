import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import { retryScheduleKeys } from '@/queries/keys';
import * as RetryScheduleService from './RetryScheduleService';
import type { RetryScheduleCreatePayload, RetrySchedulePayload } from './retrySchedule.types';

export function useRetryScheduleList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.list(organizationId.value)),
    queryFn: () => RetryScheduleService.listSchedules(organizationId.value),
    enabled: computed(() => organizationId.value.length > 0),
  });
}

export function useRetryScheduleDetail(retryScheduleId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.detail(retryScheduleId.value)),
    queryFn: () => RetryScheduleService.getSchedule(retryScheduleId.value),
    enabled: computed(() => retryScheduleId.value.length > 0),
    // Prevent background refetch from clobbering in-flight edits on the form.
    staleTime: Infinity,
    refetchOnWindowFocus: false,
  });
}

export function useCreateRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (payload: RetryScheduleCreatePayload) =>
      RetryScheduleService.createSchedule(payload),
    onSuccess: (_created, payload) => {
      void queryClient.invalidateQueries({
        queryKey: retryScheduleKeys.list(payload.organization_id),
      });
    },
  });
}

export function useUpdateRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { retryScheduleId: string; payload: RetrySchedulePayload }) =>
      RetryScheduleService.updateSchedule(args.retryScheduleId, args.payload),
    onSuccess: (updated, args) => {
      // Refresh the affected detail entry and all lists (could be many orgs).
      void queryClient.invalidateQueries({
        queryKey: retryScheduleKeys.detail(args.retryScheduleId),
      });
      void queryClient.invalidateQueries({
        queryKey: retryScheduleKeys.list(updated.organization_id),
      });
    },
  });
}

export function useDeleteRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (retryScheduleId: string) => RetryScheduleService.deleteSchedule(retryScheduleId),
    onSuccess: () => {
      // Delete doesn't return the resource, so wide-invalidate by base key.
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
    },
  });
}
