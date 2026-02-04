import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as MemberService from './MemberService';
import type { Invitation } from './MemberService';
import { memberKeys } from '@/queries/keys';

export function useMemberList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => memberKeys.list(organizationId.value)),
    queryFn: () => MemberService.get(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useInviteMember() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { organizationId: string; invitation: Invitation }) =>
      MemberService.invite(params.organizationId, params.invitation),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: memberKeys.all });
    },
  });
}

export function useRevokeMember() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { organizationId: string; userId: string }) =>
      MemberService.revoke(params.organizationId, params.userId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: memberKeys.all });
    },
  });
}

export function useEditMemberRole() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { organizationId: string; userId: string; role: string }) =>
      MemberService.edit_role(params.organizationId, params.userId, params.role),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: memberKeys.all });
    },
  });
}
