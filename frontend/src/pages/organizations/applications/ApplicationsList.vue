<script setup lang="ts">
import { ColDef } from 'ag-grid-community';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import { Application } from './ApplicationService';
import * as ApplicationService from './ApplicationService';
import Hook0Button from '@/components/Hook0Button.vue';
import { routes } from '@/routes';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import { UUID } from '@/http';
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
    field: 'name',
    suppressMovable: true,
    headerName: 'Name',
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      to: (row: Application) => {
        return {
          name: routes.ApplicationsDashboard,
          params: {
            application_id: row.application_id,
            organization_id: row.organization_id,
          },
        };
      },
    },
  },
  {
    field: 'application_id',
    suppressMovable: true,
    cellRenderer: Hook0TableCellCode,
    headerName: 'Id',
  },
  {
    width: 105,
    suppressMovable: true,
    headerName: 'Options',
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Delete',
      icon: 'trash',
      onClick(row: Application) {
        if (confirm(`Are you sure to delete "${row.name}" application?`)) {
          ApplicationService.remove(row.application_id)
            .then(() => {
              // @TODO notify user of success
              _forceLoad();
            })
            // @TODO proper error management
            .catch((err: Error) => {
              alert(err);
              throw err;
            });
        }
      },
    },
  },
];

const applications$ = ref<Promise<Array<Application>>>();
const organization_id = ref<null | UUID>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  applications$.value = ApplicationService.list(route.params.organization_id as string);
}

function _load() {
  if (organization_id.value !== route.params.organization_id) {
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
  <Promised :promise="applications$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="applications">
      <Hook0Card data-test="applications-card">
        <Hook0CardHeader>
          <template #header> Applications </template>
          <template #subtitle>
            Each application send events to Hook0 API and Hook0 dispatch these extends to customers
            through webhooks.
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="applications.length > 0">
          <transition name="ease">
            <Hook0Table
              data-test="applications-table"
              :context="{ applications$, columnDefs }"
              :column-defs="columnDefs"
              :row-data="applications"
              row-id-field="application_id"
            >
            </Hook0Table>
          </transition>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text
                  >Start your journey by creating a Hook0 application. This application will have
                  API keys that will be required to send events to Hook0 API so it can dispatch
                  these events to your customers through webhooks.</Hook0Text
                >
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button
            class="primary"
            type="button"
            data-test="applications-create-button"
            :to="{
              name: routes.ApplicationsNew,
              params: {
                organization_id: $route.params.organization_id,
              },
            }"
            >Create new application
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
