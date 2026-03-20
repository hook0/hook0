<script setup lang="ts">
import { h, markRaw, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { KeyRound, Trash2, BookOpen } from 'lucide-vue-next';
import { DOCS_API_KEYS_URL } from '@/constants/externalLinks';

import { useSecretList, useCreateSecret, useRemoveSecret } from './useSecretQueries';
import type { ApplicationSecret } from './ApplicationSecretService';
import { handleMutationError } from '@/utils/handleMutationError';
import { toast } from 'vue-sonner';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityDelete } from '@/composables/useEntityDelete';
import { useRouteIds } from '@/composables/useRouteIds';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0CopyField from '@/components/Hook0CopyField.vue';
import QuickReferenceCard from '@/components/QuickReferenceCard.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canDelete } = usePermissions();

const { organizationId, applicationId } = useRouteIds();
const { data: secrets, isLoading, error, refetch } = useSecretList(applicationId);

const createMutation = useCreateSecret();
const removeMutation = useRemoveSecret();

const showCreateDialog = ref(false);
const newSecretName = ref('');

const {
  showDeleteDialog,
  entityToDelete: secretToDelete,
  requestDelete: handleDelete,
  confirmDelete,
} = useEntityDelete<ApplicationSecret>({
  deleteFn: (row) =>
    removeMutation.mutateAsync({ applicationId: applicationId.value, token: row.token }),
  successTitle: t('common.success'),
  successMessage: t('apiKeys.deleted'),
  onSuccess: () => trackEvent('app-secret', 'delete'),
});

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
        toast.success(t('common.success'), {
          description: t('apiKeys.created'),
          duration: 3000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
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
    cell: (info) =>
      h(Hook0CopyField, {
        value: String(info.getValue()),
        maskable: true,
        copyMessage: t('apiKeys.tokenCopied'),
      }),
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue<string | null>() }),
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
              variant: 'danger',
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
      <Hook0Stack direction="column" gap="lg">
        <Hook0Card data-test="api-keys-card">
          <Hook0CardHeader>
            <template #header>{{ t('apiKeys.title') }}</template>
            <template #subtitle>
              {{ t('apiKeys.subtitle') }}
            </template>
            <template #actions>
              <Hook0Button variant="secondary" :href="DOCS_API_KEYS_URL" target="_blank">
                <template #left>
                  <BookOpen :size="14" aria-hidden="true" />
                </template>
                {{ t('common.documentation') }}
              </Hook0Button>
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
              :icon="KeyRound"
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

        <!-- Quick Reference -->
        <QuickReferenceCard
          :title="t('apiKeys.quickReference')"
          :subtitle="t('apiKeys.quickReferenceDescription')"
          :doc-label="t('apiKeys.documentationLink')"
          :api-label="t('apiKeys.apiReferenceLink')"
          :organization-id="organizationId"
          :application-id="applicationId"
        />
      </Hook0Stack>
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
        autofocus
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
      <i18n-t keypath="apiKeys.deleteConfirmNamed" tag="p">
        <template #name>
          &ldquo;<strong>{{ secretToDelete?.name ?? '' }}</strong
          >&rdquo;
        </template>
      </i18n-t>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>
