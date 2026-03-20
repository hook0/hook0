<script setup lang="ts">
import { computed, h, markRaw, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Trash2, UserPlus, Users } from 'lucide-vue-next';

import { useAuthStore } from '@/stores/auth';
import {
  useMemberList,
  useInviteMember,
  useRevokeMember,
  useEditMemberRole,
} from './useMemberQueries';
import type { User, Invitation } from './MemberService';
import { handleMutationError } from '@/utils/handleMutationError';
import { toast } from 'vue-sonner';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityDelete } from '@/composables/useEntityDelete';
import { useRouteIds } from '@/composables/useRouteIds';

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
import Hook0TableSkeleton from '@/components/Hook0TableSkeleton.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

const { organizationId } = useRouteIds();
const { data: members, isLoading, error, refetch } = useMemberList(organizationId);

const currentUser = computed(() => useAuthStore().userInfo);

const inviteMutation = useInviteMember();
const revokeMutation = useRevokeMember();
const editRoleMutation = useEditMemberRole();

const invitation = ref<Invitation>(emptyInvitation());

function emptyInvitation(): Invitation {
  return {
    email: '',
    role: 'editor',
  };
}

const roleOptions: Hook0SelectSingleOption[] = [
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

const showRoleChangeDialog = ref(false);
const roleChangeTarget = ref<{ user: User; role: string } | null>(null);

const {
  showDeleteDialog: showRevokeDialog,
  entityToDelete: revokeTarget,
  requestDelete: handleRevoke,
  confirmDelete: confirmRevoke,
} = useEntityDelete<User>({
  deleteFn: (row) =>
    revokeMutation.mutateAsync({ organizationId: organizationId.value, userId: row.user_id }),
  successTitle: t('common.success'),
  successMessage: t('members.revoked'),
});

function handleRoleChange(role: string, row: User) {
  if (row.role === role) {
    toast.warning(t('common.warning'), {
      description: t('members.roleAlreadySet', { email: row.email, role }),
      duration: 5000,
    });
    return;
  }

  roleChangeTarget.value = { user: row, role };
  showRoleChangeDialog.value = true;
}

function confirmRoleChange() {
  const target = roleChangeTarget.value;
  showRoleChangeDialog.value = false;
  roleChangeTarget.value = null;
  if (!target) return;

  editRoleMutation.mutate(
    { organizationId: organizationId.value, userId: target.user.user_id, role: target.role },
    {
      onSuccess: () => {
        toast.success(t('common.success'), {
          description: t('members.roleChanged', { email: target.user.email, role: target.role }),
          duration: 5000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
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
        toast.success(t('common.success'), {
          description: t('members.invited'),
          duration: 3000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
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
        disabled: isCurrentUserRow(row) || !canEdit('member'),
        onChange: (role: string) => handleRoleChange(role, row),
      });
    },
  },
  ...(canDelete('member')
    ? [
        {
          id: 'options',
          header: t('common.actions'),
          cell: (info: { row: { original: User } }) => {
            const row = info.row.original;
            return h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              variant: 'danger',
              disabled: isCurrentUserRow(row),
              onClick: () => handleRevoke(row),
            });
          },
        },
      ]
    : []),
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
        <Hook0TableSkeleton :columns="4" :rows="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (members is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Stack direction="column" gap="lg">
        <!-- Invite card (primary action — shown first) -->
        <Hook0Card v-if="canCreate('member')">
          <Hook0CardHeader>
            <template #header>
              <Hook0Stack direction="row" align="center" gap="sm">
                <UserPlus :size="18" aria-hidden="true" />
                {{ t('members.inviteTitle') }}
              </Hook0Stack>
            </template>
            <template #subtitle>{{ t('members.inviteSubtitle') }}</template>
          </Hook0CardHeader>
          <Hook0Form data-test="members-invite-form" @submit="invite">
            <Hook0CardFooter class="members-invite-footer">
              <div class="members-invite-footer__fields">
                <Hook0Input
                  v-model="invitation.email"
                  type="email"
                  class="members-invite-footer__email"
                  :placeholder="t('members.emailPlaceholder')"
                  required
                  data-test="members-invite-email-input"
                />
                <Hook0Select
                  v-model="invitation.role"
                  :options="roleOptions"
                  data-test="members-invite-role-select"
                />
              </div>
              <Hook0Button
                variant="primary"
                submit
                class="members-invite-footer__button"
                :disabled="invitation.email === '' || invitation.role === ''"
                data-test="members-invite-button"
              >
                {{ t('members.invite') }}
              </Hook0Button>
            </Hook0CardFooter>
          </Hook0Form>
        </Hook0Card>

        <!-- Members list -->
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
              :icon="Users"
            />
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>

    <Hook0Dialog
      :open="showRoleChangeDialog"
      variant="default"
      :title="t('members.roleChangeTitle')"
      @close="
        showRoleChangeDialog = false;
        roleChangeTarget = null;
      "
      @confirm="confirmRoleChange()"
    >
      <p v-if="roleChangeTarget">
        <i18n-t keypath="members.roleChangeConfirm" tag="span">
          <template #email>
            <strong>{{ roleChangeTarget.user.email }}</strong>
          </template>
          <template #role>
            <strong>{{ roleChangeTarget.role }}</strong>
          </template>
        </i18n-t>
      </p>
    </Hook0Dialog>

    <Hook0Dialog
      :open="showRevokeDialog"
      variant="danger"
      :title="t('members.revokeTitle')"
      @close="
        showRevokeDialog = false;
        revokeTarget = null;
      "
      @confirm="confirmRevoke()"
    >
      <p v-if="revokeTarget">
        <i18n-t keypath="members.revokeConfirm" tag="span">
          <template #email>
            <strong>{{ revokeTarget.email }}</strong>
          </template>
        </i18n-t>
      </p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
.members-invite-footer {
  flex-wrap: wrap;
}

.members-invite-footer__fields {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-left: auto;
  min-width: 0;
}

.members-invite-footer__email {
  flex: 1 1 0;
  min-width: 0;
  max-width: 20rem;
}

@media (max-width: 767px) {
  .members-invite-footer__fields {
    flex: 1 1 100%;
  }

  .members-invite-footer__button {
    flex: 1 1 100%;
  }
}
</style>
