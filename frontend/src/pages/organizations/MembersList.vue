<script setup lang="ts">
import { computed, h, ref } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import { useAuthStore } from '@/stores/auth';
import {
  useMemberList,
  useInviteMember,
  useRevokeMember,
  useEditMemberRole,
} from './useMemberQueries';
import type { User, Invitation } from './MemberService';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { push } from 'notivue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellSelect from '@/components/Hook0TableCellSelect.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();
const route = useRoute();

const organizationId = computed(() => route.params.organization_id as string);
const { data: members, isLoading, error, refetch } = useMemberList(organizationId);

const currentUser = computed(() => useAuthStore().userInfo);

const inviteMutation = useInviteMember();
const revokeMutation = useRevokeMember();
const editRoleMutation = useEditMemberRole();

const invitation = ref<Invitation>(emptyInvitation());

function emptyInvitation(): Invitation {
  return {
    email: '',
    role: '',
  };
}

const roleOptions: Hook0SelectSingleOption[] = [
  { label: '', value: '' },
  { label: t('members.roleEditor'), value: 'editor' },
  { label: t('members.roleViewer'), value: 'viewer' },
];

const cellRoleOptions = [
  { label: t('members.roleViewer'), value: 'viewer' },
  { label: t('members.roleEditor'), value: 'editor' },
];

function isCurrentUserRow(row: User): boolean {
  return currentUser.value !== null && row.email === currentUser.value.email;
}

function handleRoleChange(role: string, row: User) {
  if (!confirm(t('members.roleChangeConfirm', { email: row.email, role }))) return;

  if (row.role === role) {
    push.warning({
      title: t('common.warning'),
      message: t('members.roleAlreadySet', { email: row.email, role }),
      duration: 5000,
    });
    return;
  }

  editRoleMutation.mutate(
    { organizationId: organizationId.value, userId: row.user_id, role },
    {
      onSuccess: () => {
        push.success({
          title: t('common.success'),
          message: t('members.roleChanged', { email: row.email, role }),
          duration: 5000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function handleRevoke(row: User) {
  if (!confirm(t('members.revokeConfirm', { email: row.email }))) return;

  revokeMutation.mutate(
    { organizationId: organizationId.value, userId: row.user_id },
    {
      onSuccess: () => {
        push.success({
          title: t('common.success'),
          message: t('members.revoked'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function invite() {
  if (invitation.value.email === '' || invitation.value.role === '') return;

  inviteMutation.mutate(
    { organizationId: organizationId.value, invitation: invitation.value },
    {
      onSuccess: () => {
        invitation.value = emptyInvitation();
        push.success({
          title: t('common.success'),
          message: t('members.invited'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

const columns: ColumnDef<User, unknown>[] = [
  {
    accessorKey: 'name',
    header: t('members.name'),
    enableSorting: true,
    cell: (info) => {
      const row = info.row.original;
      const you = isCurrentUserRow(row) ? ` ${t('members.currentUser')}` : '';
      return `${row.first_name} ${row.last_name}${you}`;
    },
  },
  {
    accessorKey: 'email',
    header: t('members.email'),
    enableSorting: true,
  },
  {
    accessorKey: 'role',
    header: t('members.role'),
    enableSorting: true,
    cell: (info) => {
      const row = info.row.original;
      return h(Hook0TableCellSelect, {
        options: cellRoleOptions,
        modelValue: row.role,
        disabled: isCurrentUserRow(row),
        onChange: (role: string) => handleRoleChange(role, row),
      });
    },
  },
  {
    id: 'options',
    header: t('common.actions'),
    cell: (info) => {
      const row = info.row.original;
      return h(Hook0TableCellLink, {
        value: t('common.delete'),
        icon: 'trash',
        disabled: isCurrentUserRow(row),
        onClick: () => handleRevoke(row),
      });
    },
  },
];
</script>

<template>
  <Hook0PageLayout :title="t('members.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !members" data-test="members-card">
      <Hook0CardHeader>
        <template #header>{{ t('members.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (members is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="members-card">
        <Hook0CardHeader>
          <template #header>{{ t('members.title') }}</template>
          <template #subtitle>{{ t('members.subtitle') }}</template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="members.members.length > 0">
          <Hook0Table
            data-test="members-table"
            :columns="columns"
            :data="members.members"
            row-id-field="user_id"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('members.empty.title')"
            :description="t('members.empty.description')"
          />
        </Hook0CardContent>

        <Hook0Form data-test="members-invite-form" @submit="invite">
          <Hook0CardFooter>
            <Hook0Input
              v-model="invitation.email"
              type="email"
              :placeholder="t('members.emailPlaceholder')"
              required
              data-test="members-invite-email-input"
            />
            <Hook0Select
              v-model="invitation.role"
              :options="roleOptions"
              data-test="members-invite-role-select"
            />
            <Hook0Button
              variant="primary"
              submit
              :disabled="invitation.email === '' || invitation.role === ''"
              data-test="members-invite-button"
            >
              {{ t('members.invite') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Form>
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0 components handle all styling */
</style>
