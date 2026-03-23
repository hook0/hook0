import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as ApplicationService from './ApplicationService';
import type { ApplicationPost } from './ApplicationService';
import { applicationKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

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
  return useInvalidatingMutation({
    mutationFn: (application: ApplicationPost) => ApplicationService.create(application),
    invalidateKeys: applicationKeys.all,
  });
}

export function useUpdateApplication() {
  return useInvalidatingMutation({
    mutationFn: (params: { applicationId: string; application: ApplicationPost }) =>
      ApplicationService.update(params.applicationId, params.application),
    invalidateKeys: applicationKeys.all,
  });
}

export function useRemoveApplication() {
  return useInvalidatingMutation({
    mutationFn: (applicationId: string) => ApplicationService.remove(applicationId),
    invalidateKeys: applicationKeys.all,
  });
}
