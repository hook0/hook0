<script setup lang="ts">
import { ColDef, ValueFormatterParams } from '@ag-grid-community/core';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import { UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import { routes } from '@/routes';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import * as EventsService from './EventsService';
import { Event } from './EventsService';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Error from '@/components/Hook0Error.vue';

const route = useRoute();

interface Props {
  // cache-burst
  burst?: string | string[];
}

defineProps<Props>();
const columnDefs: ColDef[] = [
  {
    field: 'received_at',
    suppressMovable: true,
    suppressSizeToFit: true,
    width: 175,
    sortable: true,
    resizable: true,
    headerName: 'Received At',
    cellRenderer: Hook0TableCellDate,
  },
  {
    field: 'event_type_name',
    headerName: 'Event Type',
    suppressMovable: true,
    resizable: true,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value(row: Event) {
        return row.event_type_name;
      },

      to(row: Event) {
        return {
          name: routes.EventsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            event_id: row.event_id,
          },
        };
      },
    },
  },
  {
    field: 'payload_content_type',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    cellRenderer: Hook0TableCellCode,
    headerName: 'Payload Type',
  },
  {
    field: 'labels',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Labels',
    cellRenderer: Hook0TableCellCode,
    cellRendererParams: {
      value(row: Event) {
        return Object.entries(row.labels as Record<string, string>)
          .map(([key, value]) => `${key}=${value}`)
          .join(' ');
      },
    },
  },
  {
    field: 'ip',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    suppressSizeToFit: true,
    width: 160,
    cellRenderer: Hook0TableCellCode,
    headerName: 'IP',
  },
  {
    field: 'metadata',
    suppressMovable: true,
    sortable: true,
    suppressSizeToFit: true,
    width: 95,
    headerName: 'Metadata',
    valueFormatter: (params: ValueFormatterParams<Event, Record<string, never>>) => {
      const number = Object.keys(params.value ?? {}).length;
      return number > 0 ? `✔ (${number})` : '❌';
    },
  },
];

const events$ = ref<Promise<Array<Event>>>();
const application_id = ref<null | UUID>(null);

function _forceLoad() {
  application_id.value = route.params.application_id as UUID;
  events$.value = EventsService.list(application_id.value);
}

function _load() {
  if (application_id.value !== route.params.application_id) {
    _forceLoad();
  }
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <Promised :promise="events$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="event_types">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Events </template>
          <template #subtitle>
            Events that Hook0 receive from your application and that Hook0 forwarded to
            subscriptions (webhooks).
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="event_types.length > 0">
          <Hook0Table
            :context="{ events$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="event_types"
          >
          </Hook0Table>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text class="center block" style="text-align: center"
                  >Your application did not send any events. Time to send the first one!
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter> </Hook0CardFooter>
      </Hook0Card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
</template>
