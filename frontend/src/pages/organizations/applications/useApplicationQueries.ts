import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as ApplicationService from './ApplicationService';
import type { ApplicationPost } from './ApplicationService';
import { applicationKeys } from '@/queries/keys';

export function useApplicationList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => applicationKeys.list(organizationId.value)),
    queryFn: () => ApplicationService.list(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useApplicationDetail(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => applicationKeys.detail(applicationId.value)),
    queryFn: () => ApplicationService.get(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

export function useCreateApplication() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (application: ApplicationPost) => ApplicationService.create(application),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: applicationKeys.all });
    },
  });
}

export function useUpdateApplication() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { applicationId: string; application: ApplicationPost }) =>
      ApplicationService.update(params.applicationId, params.application),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: applicationKeys.all });
    },
  });
}

export function useRemoveApplication() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (applicationId: string) => ApplicationService.remove(applicationId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: applicationKeys.all });
    },
  });
}
