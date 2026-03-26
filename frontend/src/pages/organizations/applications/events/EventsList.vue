<script setup lang="ts">
import { h, markRaw, ref } from 'vue';
import { useRoute } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { RefreshCw, ArrowDownToLine } from 'lucide-vue-next';
import { DOCS_EVENTS_URL, API_DOCS_EVENTS_URL } from '@/constants/externalLinks';

import { useEventList, useReplayEvent } from './useEventQueries';
import type { Event } from './EventsService';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import { toast } from 'vue-sonner';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellEventTypes from '@/components/Hook0TableCellEventTypes.vue';
import Hook0TableCellLabels from '@/components/Hook0TableCellLabels.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0Uuid from '@/components/Hook0Uuid.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import EventSidePanel from './EventSidePanel.vue';
import Hook0DocButtons from '@/components/Hook0DocButtons.vue';

const { t } = useI18n();
const route = useRoute();
const { applicationId } = useRouteIds();

// Side panel state
const sidePanelOpen = ref(false);
const selectedEventId = ref('');

function handleRowClick(row: Event) {
  selectedEventId.value = row.event_id;
  sidePanelOpen.value = true;
}

function closeSidePanel() {
  sidePanelOpen.value = false;
}

// Events list query
const { data: events, isLoading, error, refetch } = useEventList(applicationId);

// Mutations
const replayMutation = useReplayEvent();

function handleReplay(row: Event) {
  replayMutation.mutate(
    { eventId: row.event_id, applicationId: applicationId.value },
    {
      onSuccess: () => {
        toast.success(t('events.replay'), {
          description: t('events.replaySuccess'),
          duration: 5000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

const columns: ColumnDef<Event, unknown>[] = [
  {
    accessorKey: 'event_id',
    header: t('events.id'),
    cell: (info) =>
      h(
        Hook0Button,
        {
          variant: 'link',
          to: {
            name: routes.EventsDetail,
            params: { ...route.params, event_id: info.row.original.event_id },
          },
          onClick: (e: MouseEvent) => e.stopPropagation(),
          'data-test': 'event-id-link',
          style: 'color: var(--color-primary)',
        },
        () =>
          h(Hook0Uuid, {
            value: String(info.getValue()),
            truncated: true,
            style: 'color: inherit',
          })
      ),
  },
  {
    accessorKey: 'received_at',
    header: t('events.receivedAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue() as string | null }),
  },
  {
    accessorKey: 'event_type_name',
    header: t('events.type'),
    cell: (info) =>
      h(Hook0TableCellEventTypes, {
        value: [String(info.getValue())],
        to: {
          name: routes.EventTypesList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      }),
  },
  {
    accessorKey: 'labels',
    header: t('events.labels'),
    enableSorting: true,
    cell: (info) => {
      const labels = (info.row.original.labels ?? {}) as Record<string, string>;
      if (Object.keys(labels).length === 0) return '';
      return h(Hook0TableCellLabels, { value: labels });
    },
  },
  {
    id: 'options',
    header: t('common.actions'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: t('events.replay'),
        icon: markRaw(RefreshCw),
        onClick: () => handleReplay(info.row.original),
      }),
  },
];
</script>

<template>
  <Hook0PageLayout :title="t('events.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !events" data-test="events-card">
      <Hook0CardHeader>
        <template #header>{{ t('events.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (events is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="events-card">
        <Hook0CardHeader>
          <template #header>{{ t('events.title') }}</template>
          <template #subtitle>
            {{ t('events.subtitle') }}
          </template>
          <template #actions>
            <Hook0DocButtons :doc-url="DOCS_EVENTS_URL" :api-url="API_DOCS_EVENTS_URL" />
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="events.length > 0">
          <Hook0Table
            data-test="events-table"
            :columns="columns"
            :data="events"
            row-id-field="event_id"
            clickable-rows
            @row-click="handleRowClick"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('events.empty.title')"
            :description="t('events.empty.description')"
            :icon="ArrowDownToLine"
          >
            <template #action>
              <Hook0Button
                variant="primary"
                data-test="events-send-button"
                :to="{ name: routes.EventsSend, params: route.params }"
              >
                {{ t('events.sendEvent') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="events.length > 0">
          <Hook0Button
            variant="primary"
            data-test="events-send-button"
            :to="{ name: routes.EventsSend, params: route.params }"
          >
            {{ t('events.sendEvent') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>

    <!-- Event side panel -->
    <EventSidePanel
      :open="sidePanelOpen"
      :event-id="selectedEventId"
      :application-id="applicationId"
      @close="closeSidePanel"
    />
  </Hook0PageLayout>
</template>

<style scoped>
:deep(.table-cell-uuid-link) {
  text-decoration: none;
  color: inherit;
}

:deep(.table-cell-uuid-link:hover) {
  text-decoration: underline;
  text-underline-offset: 2px;
  color: var(--color-primary);
}

:deep(.table-cell-uuid-link:focus-visible) {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}
</style>
