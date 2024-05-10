<script setup lang="ts">
import { ColDef } from '@ag-grid-community/core';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import { UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import * as ServiceTokenService from './ServicesTokenService.ts';
import { ServiceToken } from './ServicesTokenService.ts';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import { push } from 'notivue';

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
    sortable: true,
    resizable: true,
    headerName: 'Name',
  },
  {
    field: 'biscuit',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Biscuit token',
  },
  {
    field: 'created_at',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Created At',
    cellRenderer: Hook0TableCellDate,
  },
  {
    suppressMovable: true,
    headerName: 'Options',
    suppressSizeToFit: true,
    width: 95,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Show',
      icon: 'eye',
      onClick: (row: ServiceToken): void => {
        push.warning({
          title: 'Service Token',
          message: `Token: ${row.token_id}\nName: ${row.name}`,
        });
      },
    },
  },
];

const service_token$ = ref<Promise<ServiceToken>>();
const organization_id = ref<null | UUID>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  let service_token = route.params.service_token as UUID;
  service_token$.value = ServiceTokenService.get(service_token);
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
  <Promised :promise="service_token$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="service_tokens">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Service Token </template>
          <template #subtitle> Some information about the organization service token ... </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="service_tokens.length > 0">
          <Hook0Table
            :context="{ service_token$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="service_tokens"
          >
          </Hook0Table>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text
                  >Hook0 authenticates your API requests using your application’s API keys. If you
                  don’t include your key when making an API request, or use an incorrect or outdated
                  one, Hook0 returns a 401 -
                  <Hook0Text class="code">Unauthorized HTTP response code</Hook0Text>
                  .
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>
      </Hook0Card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
</template>
