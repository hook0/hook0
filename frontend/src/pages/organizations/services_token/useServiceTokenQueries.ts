import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as ServicesTokenService from './ServicesTokenService';
import type { ServiceTokenPost } from './ServicesTokenService';
import { serviceTokenKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useServiceTokenList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => serviceTokenKeys.list(organizationId.value)),
    queryFn: () => ServicesTokenService.list(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useServiceTokenDetail(tokenId: Ref<string>, organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => serviceTokenKeys.detail(tokenId.value, organizationId.value)),
    queryFn: () => ServicesTokenService.get(tokenId.value, organizationId.value),
    enabled: computed(() => !!tokenId.value && !!organizationId.value),
  });
}

export function useCreateServiceToken() {
  return useInvalidatingMutation({
    mutationFn: (token: ServiceTokenPost) => ServicesTokenService.create(token),
    invalidateKeys: serviceTokenKeys.all,
  });
}

export function useUpdateServiceToken() {
  return useInvalidatingMutation({
    mutationFn: (params: { tokenId: string; token: ServiceTokenPost }) =>
      ServicesTokenService.update(params.tokenId, params.token),
    invalidateKeys: serviceTokenKeys.all,
  });
}

export function useRemoveServiceToken() {
  return useInvalidatingMutation({
    mutationFn: (params: { tokenId: string; organizationId: string }) =>
      ServicesTokenService.remove(params.tokenId, params.organizationId),
    invalidateKeys: serviceTokenKeys.all,
  });
}
