<script setup lang="ts">
import { h, markRaw } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Trash2 } from 'lucide-vue-next';

import { useApplicationList, useRemoveApplication } from './useApplicationQueries';
import type { Application } from './ApplicationService';
import { routes } from '@/routes';
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
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';

const { t } = useI18n();
const router = useRouter();

// Permissions
const { canCreate, canDelete } = usePermissions();

const { organizationId } = useRouteIds();
const { data: applications, isLoading, error, refetch } = useApplicationList(organizationId);

const removeMutation = useRemoveApplication();

const {
  showDeleteDialog,
  entityToDelete: applicationToDelete,
  requestDelete,
  confirmDelete,
} = useEntityDelete<Application>({
  deleteFn: (app) => removeMutation.mutateAsync(app.application_id),
  successTitle: t('applications.deleted'),
  successMessage: (app) => t('applications.deletedMessage', { name: app.name }),
  onSuccess: () => void refetch(),
});

const columns: ColumnDef<Application, unknown>[] = [
  {
    accessorKey: 'name',
    header: t('common.name'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: String(info.getValue()),
        to: {
          name: routes.ApplicationsDashboard,
          params: {
            application_id: info.row.original.application_id,
            organization_id: info.row.original.organization_id,
          },
        },
      }),
  },
  {
    accessorKey: 'application_id',
    header: t('applications.id'),
    cell: (info) => h(Hook0TableCellCode, { value: String(info.getValue()) }),
  },
  ...(canDelete('application')
    ? [
        {
          id: 'options',
          header: t('common.actions'),
          cell: (info: { row: { original: Application } }) =>
            h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              onClick: () => requestDelete(info.row.original),
            }),
        },
      ]
    : []),
];
</script>

<template>
  <Hook0PageLayout
    :title="t('applications.title')"
    :description="t('applications.empty.description')"
  >
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !applications" data-test="applications-card">
      <Hook0CardHeader>
        <template #header>{{ t('applications.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (applications is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="applications-card">
        <Hook0CardHeader>
          <template #header>{{ t('applications.title') }}</template>
          <template #subtitle>
            {{ t('applications.subtitle') }}
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="applications.length > 0">
          <Hook0Table
            data-test="applications-table"
            :columns="columns"
            :data="applications"
            row-id-field="application_id"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('applications.empty.title')"
            :description="t('applications.empty.description')"
          >
            <template v-if="canCreate('application')" #action>
              <Hook0Button
                variant="primary"
                type="button"
                data-test="applications-create-button"
                @click="
                  void router.push({
                    name: routes.ApplicationsNew,
                    params: { organization_id: organizationId },
                  })
                "
              >
                {{ t('applications.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="applications.length > 0 && canCreate('application')">
          <Hook0Button
            variant="primary"
            type="button"
            data-test="applications-create-button"
            :to="{
              name: routes.ApplicationsNew,
              params: { organization_id: organizationId },
            }"
          >
            {{ t('applications.create') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('applications.delete')"
      @close="
        showDeleteDialog = false;
        applicationToDelete = null;
      "
      @confirm="confirmDelete()"
    >
      <p v-if="applicationToDelete">
        <i18n-t keypath="applications.deleteConfirm" tag="span">
          <template #name>
            <strong>{{ applicationToDelete.name }}</strong>
          </template>
        </i18n-t>
      </p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped></style>
