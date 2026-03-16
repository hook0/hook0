<script setup lang="ts">
import { computed, h, markRaw, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Trash2 } from 'lucide-vue-next';

import { useEventTypeList, useDeactivateEventType } from './useEventTypeQueries';
import type { EventType } from './EventTypeService';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { push } from 'notivue';
import { usePermissions } from '@/composables/usePermissions';

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

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

// Permissions
const { canCreate, canDelete } = usePermissions();

const applicationId = computed(() => route.params.application_id as string);
const { data: eventTypes, isLoading, error, refetch } = useEventTypeList(applicationId);

const deactivateMutation = useDeactivateEventType();

const showDeactivateDialog = ref(false);
const eventTypeToDeactivate = ref<EventType | null>(null);

function handleDeactivate(row: EventType) {
  eventTypeToDeactivate.value = row;
  showDeactivateDialog.value = true;
}

function confirmDeactivate() {
  const row = eventTypeToDeactivate.value;
  showDeactivateDialog.value = false;
  eventTypeToDeactivate.value = null;
  if (!row) return;

  deactivateMutation.mutate(
    { applicationId: applicationId.value, eventTypeName: row.event_type_name },
    {
      onSuccess: () => {
        push.success({
          title: t('common.success'),
          message: t('eventTypes.deactivated', { name: row.event_type_name }),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

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
              value: t('eventTypes.deactivate'),
              icon: markRaw(Trash2),
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

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !eventTypes" data-test="event-types-card">
      <Hook0CardHeader>
        <template #header>{{ t('eventTypes.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (eventTypes is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="event-types-card">
        <Hook0CardHeader>
          <template #header>{{ t('eventTypes.title') }}</template>
          <template #subtitle>{{ t('eventTypes.subtitle') }}</template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="eventTypes.length > 0">
          <Hook0Table
            data-test="event-types-table"
            :columns="columns"
            :data="eventTypes"
            row-id-field="event_type_name"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('eventTypes.empty.title')"
            :description="t('eventTypes.empty.description')"
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

        <Hook0CardFooter v-if="eventTypes.length > 0 && canCreate('event_type')">
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
      :title="t('eventTypes.deactivate')"
      @close="
        showDeactivateDialog = false;
        eventTypeToDeactivate = null;
      "
      @confirm="confirmDeactivate()"
    >
      <p v-if="eventTypeToDeactivate">
        {{ t('eventTypes.confirmDeactivate', { name: eventTypeToDeactivate.event_type_name }) }}
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
