<script setup lang="ts">
import { ColDef } from '@ag-grid-community/core';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import { Problem, UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import * as ServiceTokenService from './ServicesTokenService.ts';
import { ServiceToken } from './ServicesTokenService.ts';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import { push } from 'notivue';
import router from '@/router.ts';
import { routes } from '@/routes.ts';

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
        router
          .push({
            name: routes.ServiceTokenView,
            params: {
              organization_id: organization_id.value as string,
              service_token_id: row.token_id,
            },
          })
          .catch(displayError);
      },
    },
  },
  {
    suppressMovable: true,
    headerName: '',
    suppressSizeToFit: true,
    width: 95,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Edit',
      icon: 'pen',
      onClick: (row: ServiceToken): void => {
        const name = prompt('Edit the service token name?', row.name);
        if (!name) {
          return;
        }

        ServiceTokenService.update(row.token_id, {
          name: name,
          organization_id: organization_id.value as string,
        })
          .then(() => {
            _forceLoad();
          })
          .catch(displayError);
      },
    },
  },
  {
    suppressMovable: true,
    headerName: '',
    suppressSizeToFit: true,
    width: 105,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Delete',
      icon: 'trash',
      onClick: (row: ServiceToken): void => {
        if (confirm('Are you sure you want to delete this service token?\n\nEvery token derived from this token will be revoked as well.')) {
          ServiceTokenService.remove(row.token_id, organization_id.value as string)
            .then(() => {
              _forceLoad();
            })
            .catch(displayError);
        }
      },
    },
  },
];

const services_token$ = ref<Promise<Array<ServiceToken>>>();
const organization_id = ref<null | UUID>(null);

function createNew(event: Event) {
  event.stopImmediatePropagation();
  event.preventDefault();

  const name = prompt('Give a name to your new service token:');
  if (!name) {
    return;
  }

  ServiceTokenService.create({ name: name, organization_id: organization_id.value as string })
    .then(() => {
      _forceLoad();
    })
    .catch(displayError);
}

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  services_token$.value = ServiceTokenService.list(organization_id.value);
}

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    _forceLoad();
  }
}

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <Promised :promise="services_token$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="services_tokens">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Service Tokens </template>
          <template #subtitle>
            These keys will allow you to authenticate API requests with some services to Hook0.
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="services_tokens.length > 0">
          <Hook0Table
            :context="{ services_token$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="services_tokens"
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

        <Hook0CardFooter>
          <Hook0Button class="primary" type="button" @click="createNew"
            >Create new SERVICE TOKEN
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
