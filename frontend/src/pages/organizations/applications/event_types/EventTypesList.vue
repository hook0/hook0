<script setup lang="ts">
import { ColDef } from '@ag-grid-community/core';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import Hook0Button from '@/components/Hook0Button.vue';
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
import * as EventTypeService from './EventTypeService';
import { EventType } from './EventTypeService';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Error from '@/components/Hook0Error.vue';

const router = useRouter();
const route = useRoute();

interface Props {
  // cache-burst
  burst?: string | string[];
}

defineProps<Props>();
const columnDefs: ColDef[] = [
  {
    field: 'event_type_name',
    suppressMovable: true,
    cellRenderer: Hook0TableCellCode,
    minWidth: 360,
    sortable: true,
    headerName: 'Name',
  },
  {
    suppressMovable: true,
    suppressSizeToFit: true,
    width: 105,
    headerName: 'Options',
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Delete',
      icon: 'trash',
      onClick: (row: EventType): void => {
        if (confirm(`Are you sure to delete "${row.event_type_name}" event?`)) {
          EventTypeService.remove(application_id.value as string, row.event_type_name)
            .then(() => {
              // @TODO notify user of success
              _forceLoad();
            })
            // @TODO proper error management
            .catch((err) => {
              window.alert(err);
              throw err;
            });
        }
      },
    },
  },
];

const event_types$ = ref<Promise<Array<EventType>>>();
const application_id = ref<null | UUID>(null);

function _forceLoad() {
  application_id.value = route.params.application_id as UUID;
  event_types$.value = EventTypeService.list(application_id.value);
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
  <Promised :promise="event_types$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="event_types">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>Event Types</template>
          <template #subtitle>
            Each event sent through a webhook must have an event type.
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="event_types.length > 0">
          <Hook0Table
            :context="{ event_types$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="event_types"
          >
          </Hook0Table>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text
                  >Your application will send events to Hook0 that will forward these events to
                  registered subscriptions (webhooks), each of these event must have a type (e.g.
                  "billing.invoice.created"), it's time to create your first event type!
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button
            class="primary"
            type="button"
            @click="router.push({ name: routes.EventTypesNew })"
            >Create new event type
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
</template>
