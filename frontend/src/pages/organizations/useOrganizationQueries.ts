import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as OrganizationService from './OrganizationService';
import type { OrganizationPost } from './OrganizationService';
import { organizationKeys } from '@/queries/keys';

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
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (organization: OrganizationPost) => OrganizationService.create(organization),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: organizationKeys.all });
    },
  });
}

export function useUpdateOrganization() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { organizationId: string; organization: OrganizationPost }) =>
      OrganizationService.update(params.organizationId, params.organization),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: organizationKeys.all });
    },
  });
}

export function useRemoveOrganization() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (organizationId: string) => OrganizationService.remove(organizationId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: organizationKeys.all });
    },
  });
}
