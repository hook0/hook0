<script setup lang="ts">
import { computed, h, markRaw } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Trash2, FolderTree } from 'lucide-vue-next';
import { DOCS_EVENT_TYPES_URL, API_DOCS_EVENT_TYPES_URL } from '@/constants/externalLinks';

import { useEventTypeListInfinite, useDeactivateEventType } from './useEventTypeQueries';
import type { EventType } from './EventTypeService';
import { routes } from '@/routes';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityDelete } from '@/composables/useEntityDelete';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0DocButtons from '@/components/Hook0DocButtons.vue';
import Hook0PaginatedList from '@/components/Hook0PaginatedList.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

// Permissions
const { canCreate, canDelete } = usePermissions();

const { applicationId } = useRouteIds();
const eventTypesQuery = useEventTypeListInfinite(applicationId);
const { currentPageItems, isLoading, error, refetch, totalPagesSeen, currentPageIdx } =
  eventTypesQuery;

// "Empty list" detection: empty only when we've finished loading and the very
// first page came back empty.
const hasAnyItems = computed(
  () => totalPagesSeen.value > 0 && (currentPageItems.value.length > 0 || currentPageIdx.value > 0)
);

const deactivateMutation = useDeactivateEventType();

const {
  showDeleteDialog: showDeactivateDialog,
  entityToDelete: eventTypeToDeactivate,
  requestDelete: handleDeactivate,
  confirmDelete: confirmDeactivate,
} = useEntityDelete<EventType>({
  deleteFn: (row) =>
    deactivateMutation.mutateAsync({
      applicationId: applicationId.value,
      eventTypeName: row.event_type_name,
    }),
  successTitle: t('common.success'),
  successMessage: (row) => t('eventTypes.deactivated', { name: row.event_type_name }),
});

const columns: ColumnDef<EventType, unknown>[] = [
  {
    accessorKey: 'event_type_name',
    header: t('common.name'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellCode, { value: String(info.getValue()) }),
  },
  ...(canDelete('event_type')
    ? [
        {
          id: 'options',
          header: t('common.actions'),
          cell: (info: { row: { original: EventType } }) =>
            h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              variant: 'danger',
              dataTest: 'event-type-deactivate-button',
              onClick: () => handleDeactivate(info.row.original),
            }),
        },
      ]
    : []),
];
</script>

<template>
  <Hook0PageLayout :title="t('eventTypes.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and the first page is missing) -->
    <Hook0Card v-else-if="isLoading && totalPagesSeen === 0" data-test="event-types-card">
      <Hook0CardHeader>
        <template #header>{{ t('eventTypes.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded -->
    <template v-else>
      <Hook0Card data-test="event-types-card">
        <Hook0CardHeader>
          <template #header>{{ t('eventTypes.title') }}</template>
          <template #subtitle>{{ t('eventTypes.subtitle') }}</template>
          <template #actions>
            <Hook0DocButtons :doc-url="DOCS_EVENT_TYPES_URL" :api-url="API_DOCS_EVENT_TYPES_URL" />
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="hasAnyItems">
          <Hook0PaginatedList :query="eventTypesQuery">
            <template #default="{ items }">
              <Hook0Table
                data-test="event-types-table"
                :columns="columns"
                :data="items"
                row-id-field="event_type_name"
              />
            </template>
          </Hook0PaginatedList>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('eventTypes.empty.title')"
            :description="t('eventTypes.empty.description')"
            :icon="FolderTree"
          >
            <template v-if="canCreate('event_type')" #action>
              <Hook0Button
                variant="primary"
                type="button"
                data-test="event-types-create-button"
                @click="
                  void router.push({
                    name: routes.EventTypesNew,
                    params: {
                      organization_id: route.params.organization_id,
                      application_id: route.params.application_id,
                    },
                  })
                "
              >
                {{ t('eventTypes.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="hasAnyItems && canCreate('event_type')">
          <Hook0Button
            variant="primary"
            type="button"
            data-test="event-types-create-button"
            @click="
              void router.push({
                name: routes.EventTypesNew,
                params: {
                  organization_id: route.params.organization_id,
                  application_id: route.params.application_id,
                },
              })
            "
          >
            {{ t('eventTypes.create') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>

    <Hook0Dialog
      :open="showDeactivateDialog"
      variant="danger"
      :title="t('eventTypes.delete')"
      @close="
        showDeactivateDialog = false;
        eventTypeToDeactivate = null;
      "
      @confirm="confirmDeactivate()"
    >
      <p v-if="eventTypeToDeactivate">
        <i18n-t keypath="eventTypes.confirmDeactivate" tag="span">
          <template #name>
            <strong>{{ eventTypeToDeactivate.event_type_name }}</strong>
          </template>
        </i18n-t>
      </p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
/* Prevent deactivate button text from being clipped on narrow screens */
@media (max-width: 767px) {
  :deep([data-test='event-type-deactivate-button']) {
    font-size: 0;
  }

  :deep([data-test='event-type-deactivate-button'] svg) {
    font-size: 1rem;
  }
}
</style>
