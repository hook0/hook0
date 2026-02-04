<script setup lang="ts">
import { computed, h } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import { useEventTypeList, useDeactivateEventType } from './useEventTypeQueries';
import type { EventType } from './EventTypeService';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { push } from 'notivue';

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

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const applicationId = computed(() => route.params.application_id as string);
const { data: eventTypes, isLoading, error, refetch } = useEventTypeList(applicationId);

const deactivateMutation = useDeactivateEventType();

function handleDeactivate(row: EventType) {
  if (!confirm(t('eventTypes.confirmDeactivate', { name: row.event_type_name }))) return;

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
  {
    id: 'options',
    header: t('common.actions'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: t('eventTypes.deactivate'),
        icon: 'trash',
        dataTest: 'event-type-deactivate-button',
        onClick: () => handleDeactivate(info.row.original),
      }),
  },
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
            <template #action>
              <Hook0Button
                variant="primary"
                type="button"
                data-test="event-types-create-button"
                @click="void router.push({ name: routes.EventTypesNew })"
              >
                {{ t('eventTypes.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="eventTypes.length > 0">
          <Hook0Button
            variant="primary"
            type="button"
            data-test="event-types-create-button"
            @click="void router.push({ name: routes.EventTypesNew })"
          >
            {{ t('eventTypes.create') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped></style>
