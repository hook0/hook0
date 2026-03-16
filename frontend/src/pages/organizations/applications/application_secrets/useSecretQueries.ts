import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as ApplicationSecretService from './ApplicationSecretService';
import type { ApplicationSecretPost } from './ApplicationSecretService';
import { secretKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useSecretList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => secretKeys.list(applicationId.value)),
    queryFn: () => ApplicationSecretService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

export function useCreateSecret() {
  return useInvalidatingMutation({
    mutationFn: (secret: ApplicationSecretPost) => ApplicationSecretService.create(secret),
    invalidateKeys: secretKeys.all,
  });
}

export function useUpdateSecret() {
  return useInvalidatingMutation({
    mutationFn: (params: { token: string; secret: ApplicationSecretPost }) =>
      ApplicationSecretService.update(params.token, params.secret),
    invalidateKeys: secretKeys.all,
  });
}

export function useRemoveSecret() {
  return useInvalidatingMutation({
    mutationFn: (params: { applicationId: string; token: string }) =>
      ApplicationSecretService.remove(params.applicationId, params.token),
    invalidateKeys: secretKeys.all,
  });
}
