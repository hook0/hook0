import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as MemberService from './MemberService';
import type { Invitation } from './MemberService';
import { memberKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useMemberList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => memberKeys.list(organizationId.value)),
    queryFn: () => MemberService.get(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useInviteMember() {
  return useInvalidatingMutation({
    mutationFn: (params: { organizationId: string; invitation: Invitation }) =>
      MemberService.invite(params.organizationId, params.invitation),
    invalidateKeys: memberKeys.all,
  });
}

export function useRevokeMember() {
  return useInvalidatingMutation({
    mutationFn: (params: { organizationId: string; userId: string }) =>
      MemberService.revoke(params.organizationId, params.userId),
    invalidateKeys: memberKeys.all,
  });
}

export function useEditMemberRole() {
  return useInvalidatingMutation({
    mutationFn: (params: { organizationId: string; userId: string; role: string }) =>
      MemberService.edit_role(params.organizationId, params.userId, params.role),
    invalidateKeys: memberKeys.all,
  });
}
