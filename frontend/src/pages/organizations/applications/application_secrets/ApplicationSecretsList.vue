<script setup lang="ts">
import { computed, h, markRaw, ref } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Trash2 } from 'lucide-vue-next';

import { useSecretList, useCreateSecret, useRemoveSecret } from './useSecretQueries';
import type { ApplicationSecret } from './ApplicationSecretService';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { push } from 'notivue';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0Input from '@/components/Hook0Input.vue';

const { t } = useI18n();
const route = useRoute();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canDelete } = usePermissions();

const applicationId = computed(() => route.params.application_id as string);
const { data: secrets, isLoading, error, refetch } = useSecretList(applicationId);

const createMutation = useCreateSecret();
const removeMutation = useRemoveSecret();

const showCreateDialog = ref(false);
const newSecretName = ref('');

const showDeleteDialog = ref(false);
const secretToDelete = ref<ApplicationSecret | null>(null);

function createNew(event: MouseEvent) {
  event.stopImmediatePropagation();
  event.preventDefault();
  newSecretName.value = '';
  showCreateDialog.value = true;
}

function confirmCreate() {
  const name = newSecretName.value.trim();
  showCreateDialog.value = false;
  newSecretName.value = '';
  if (!name) return;

  createMutation.mutate(
    { application_id: applicationId.value, name },
    {
      onSuccess: () => {
        trackEvent('app-secret', 'create');
        push.success({
          title: t('common.success'),
          message: t('apiKeys.create'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function handleDelete(row: ApplicationSecret) {
  secretToDelete.value = row;
  showDeleteDialog.value = true;
}

function confirmDelete() {
  const row = secretToDelete.value;
  showDeleteDialog.value = false;
  secretToDelete.value = null;
  if (!row) return;

  removeMutation.mutate(
    { applicationId: applicationId.value, token: row.token },
    {
      onSuccess: () => {
        trackEvent('app-secret', 'delete');
        push.success({
          title: t('common.success'),
          message: t('apiKeys.deleted'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

const columns: ColumnDef<ApplicationSecret, unknown>[] = [
  {
    accessorKey: 'name',
    header: t('common.name'),
    enableSorting: true,
  },
  {
    accessorKey: 'token',
    header: t('apiKeys.token'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellCode, { value: String(info.getValue()) }),
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue() as string | null }),
  },
  ...(canDelete('application_secret')
    ? [
        {
          id: 'options',
          header: t('common.actions'),
          cell: (info: { row: { original: ApplicationSecret } }) =>
            h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              dataTest: 'api-key-delete-button',
              onClick: () => handleDelete(info.row.original),
            }),
        },
      ]
    : []),
];
</script>

<template>
  <Hook0PageLayout :title="t('apiKeys.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !secrets" data-test="api-keys-card">
      <Hook0CardHeader>
        <template #header>{{ t('apiKeys.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (secrets is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="api-keys-card">
        <Hook0CardHeader>
          <template #header>{{ t('apiKeys.title') }}</template>
          <template #subtitle>
            {{ t('apiKeys.subtitle') }}
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="secrets.length > 0">
          <Hook0Table
            data-test="api-keys-table"
            :columns="columns"
            :data="secrets"
            row-id-field="token"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('apiKeys.empty.title')"
            :description="t('apiKeys.empty.description')"
          >
            <template v-if="canCreate('application_secret')" #action>
              <Hook0Button
                variant="primary"
                type="button"
                data-test="api-keys-create-button"
                @click="createNew"
              >
                {{ t('apiKeys.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="secrets && secrets.length > 0 && canCreate('application_secret')">
          <Hook0Button
            variant="primary"
            type="button"
            data-test="api-keys-create-button"
            @click="createNew"
          >
            {{ t('apiKeys.create') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>

    <Hook0Dialog
      :open="showCreateDialog"
      variant="default"
      :title="t('apiKeys.create')"
      @close="
        showCreateDialog = false;
        newSecretName = '';
      "
      @confirm="confirmCreate()"
    >
      <Hook0Input
        v-model="newSecretName"
        :placeholder="t('common.name')"
        :label="t('common.name')"
        data-test="api-key-name-input"
        @keydown.enter="confirmCreate()"
      />
    </Hook0Dialog>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('apiKeys.delete')"
      @close="
        showDeleteDialog = false;
        secretToDelete = null;
      "
      @confirm="confirmDelete()"
    >
      <p>{{ t('apiKeys.deleteConfirm') }}</p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped></style>
