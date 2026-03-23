import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionService from './SubscriptionService';
import type { SubscriptionPost, Subscription } from './SubscriptionService';
import { subscriptionKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useSubscriptionList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => subscriptionKeys.list(applicationId.value)),
    queryFn: () => SubscriptionService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

export function useSubscriptionDetail(id: Ref<string>) {
  return useQuery({
    queryKey: computed(() => subscriptionKeys.detail(id.value)),
    queryFn: () => SubscriptionService.get(id.value),
    enabled: computed(() => !!id.value),
  });
}

export function useCreateSubscription() {
  return useInvalidatingMutation({
    mutationFn: (subscription: SubscriptionPost) => SubscriptionService.create(subscription),
    invalidateKeys: subscriptionKeys.all,
  });
}

// Custom onSuccess: also sets query data for optimistic detail update
export function useUpdateSubscription() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { subscriptionId: string; subscription: SubscriptionPost }) =>
      SubscriptionService.update(params.subscriptionId, params.subscription),
    onSuccess: (data, variables) => {
      queryClient.setQueryData(subscriptionKeys.detail(variables.subscriptionId), data);
      void queryClient.invalidateQueries({ queryKey: subscriptionKeys.all });
    },
  });
}

export function useRemoveSubscription() {
  return useInvalidatingMutation({
    mutationFn: (params: { applicationId: string; subscriptionId: string }) =>
      SubscriptionService.remove(params.applicationId, params.subscriptionId),
    invalidateKeys: subscriptionKeys.all,
  });
}

export function useToggleSubscription() {
  return useInvalidatingMutation({
    mutationFn: (params: { subscriptionId: string; subscription: Subscription }) =>
      SubscriptionService.toggleEnable(params.subscriptionId, params.subscription),
    invalidateKeys: subscriptionKeys.all,
  });
}
