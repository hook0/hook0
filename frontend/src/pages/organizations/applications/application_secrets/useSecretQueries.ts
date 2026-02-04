import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as ApplicationSecretService from './ApplicationSecretService';
import type { ApplicationSecretPost } from './ApplicationSecretService';
import { secretKeys } from '@/queries/keys';

export function useSecretList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => secretKeys.list(applicationId.value)),
    queryFn: () => ApplicationSecretService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

export function useCreateSecret() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (secret: ApplicationSecretPost) => ApplicationSecretService.create(secret),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: secretKeys.all });
    },
  });
}

export function useUpdateSecret() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { token: string; secret: ApplicationSecretPost }) =>
      ApplicationSecretService.update(params.token, params.secret),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: secretKeys.all });
    },
  });
}

export function useRemoveSecret() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { applicationId: string; token: string }) =>
      ApplicationSecretService.remove(params.applicationId, params.token),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: secretKeys.all });
    },
  });
}
