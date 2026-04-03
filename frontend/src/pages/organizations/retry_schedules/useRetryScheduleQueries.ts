// TanStack Query composables for retry schedule CRUD.
//
// How it works:
// 1. useRetryScheduleList/Detail wrap useQuery with reactive keys that auto-refetch on org/id change
// 2. useCreate/Update use useInvalidatingMutation to auto-bust the list cache on success
// 3. useRemove additionally invalidates subscription queries — subscriptions reference schedules via FK

import { useMutation, useQueryClient, useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as RetryScheduleService from './RetryScheduleService';
import type { RetrySchedulePost, RetrySchedulePut } from './RetryScheduleService';
import { retryScheduleKeys, subscriptionKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

type UpdateRetryScheduleParams = {
  retryScheduleId: string;
  organizationId: string;
  schedule: RetrySchedulePut;
};

type RemoveRetryScheduleParams = {
  retryScheduleId: string;
  organizationId: string;
};

/**
 * Reactive query for the full list of retry schedules in an organization.
 *
 * @example
 * const { data, isLoading } = useRetryScheduleList(organizationId)
 * // data.value => RetrySchedule[] | undefined
 */
export function useRetryScheduleList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.list(organizationId.value)),
    queryFn: () => RetryScheduleService.list(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

/**
 * Reactive query for a single retry schedule, used by the edit form.
 *
 * @example
 * const { data } = useRetryScheduleDetail(retryScheduleId, organizationId)
 * // data.value => RetrySchedule | undefined
 */
export function useRetryScheduleDetail(id: Ref<string>, organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.detail(id.value, organizationId.value)),
    queryFn: () => RetryScheduleService.get(id.value, organizationId.value),
    enabled: computed(() => !!id.value && !!organizationId.value),
  });
}

/**
 * Mutation for creating a retry schedule. Invalidates the list cache on success.
 *
 * @example
 * const mutation = useCreateRetrySchedule()
 * mutation.mutate(payload, { onSuccess: () => ... })
 */
export function useCreateRetrySchedule() {
  return useInvalidatingMutation({
    mutationFn: (schedule: RetrySchedulePost) => RetryScheduleService.create(schedule),
    invalidateKeys: retryScheduleKeys.all,
  });
}

/**
 * Mutation for updating a retry schedule. Invalidates the list cache on success.
 *
 * @example
 * const mutation = useUpdateRetrySchedule()
 * mutation.mutate({ retryScheduleId, organizationId, schedule: payload })
 */
export function useUpdateRetrySchedule() {
  return useInvalidatingMutation({
    mutationFn: (params: UpdateRetryScheduleParams) =>
      RetryScheduleService.update(params.retryScheduleId, params.organizationId, params.schedule),
    invalidateKeys: retryScheduleKeys.all,
  });
}

/**
 * Mutation for deleting a retry schedule. Invalidates both schedule and subscription caches.
 *
 * @example
 * const mutation = useRemoveRetrySchedule()
 * mutation.mutateAsync({ retryScheduleId, organizationId })
 */
export function useRemoveRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: RemoveRetryScheduleParams) =>
      RetryScheduleService.remove(params.retryScheduleId, params.organizationId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
      // Subscriptions display their linked schedule name — stale cache would show a dangling reference
      void queryClient.invalidateQueries({ queryKey: subscriptionKeys.all });
    },
  });
}
