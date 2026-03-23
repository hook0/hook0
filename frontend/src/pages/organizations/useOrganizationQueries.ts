import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as OrganizationService from './OrganizationService';
import type { OrganizationPost } from './OrganizationService';
import { organizationKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useOrganizationList() {
  return useQuery({
    queryKey: organizationKeys.lists(),
    queryFn: () => OrganizationService.list(),
  });
}

export function useOrganizationDetail(id: Ref<string>) {
  return useQuery({
    queryKey: computed(() => organizationKeys.detail(id.value)),
    queryFn: () => OrganizationService.get(id.value),
    enabled: computed(() => !!id.value),
  });
}

export function useCreateOrganization() {
  return useInvalidatingMutation({
    mutationFn: (organization: OrganizationPost) => OrganizationService.create(organization),
    invalidateKeys: organizationKeys.all,
  });
}

export function useUpdateOrganization() {
  return useInvalidatingMutation({
    mutationFn: (params: { organizationId: string; organization: OrganizationPost }) =>
      OrganizationService.update(params.organizationId, params.organization),
    invalidateKeys: organizationKeys.all,
  });
}

export function useRemoveOrganization() {
  return useInvalidatingMutation({
    mutationFn: (organizationId: string) => OrganizationService.remove(organizationId),
    invalidateKeys: organizationKeys.all,
  });
}
