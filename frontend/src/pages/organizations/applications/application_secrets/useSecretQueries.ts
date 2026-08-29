import { useQuery } from '@tanstack/vue-query';
import { computed, ref, type Ref } from 'vue';
import * as ApplicationSecretService from './ApplicationSecretService';
import type { ApplicationSecretPost } from './ApplicationSecretService';
import { secretKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

// `isEnabled` lets a caller that has no use for the list avoid the request: the send-event screen
// authenticates its examples with the session's own token during the onboarding tutorial, and with
// nothing at all on an instance that no longer accepts an application secret as a credential.
export function useSecretList(applicationId: Ref<string>, isEnabled: Ref<boolean> = ref(true)) {
  return useQuery({
    queryKey: computed(() => secretKeys.list(applicationId.value)),
    queryFn: () => ApplicationSecretService.list(applicationId.value),
    enabled: computed(() => isEnabled.value && !!applicationId.value),
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
