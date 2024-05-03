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
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import { Problem, UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import * as ApplicationSecretService from './ApplicationSecretService';
import { ApplicationSecret } from './ApplicationSecretService';
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
    field: 'token',
    suppressMovable: true,
    sortable: true,
    suppressSizeToFit: true,
    resizable: true,
    width: 345,
    cellRenderer: Hook0TableCellCode,
    cellRendererParams: {},
    headerName: 'Token',
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
    width: 105,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Delete',
      icon: 'trash',
      onClick: (row: ApplicationSecret): void => {
        if (confirm(`Are you sure to delete "${row.name as string}" API Key?`)) {
          ApplicationSecretService.remove(application_id.value as string, row.token)
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

const application_secrets$ = ref<Promise<Array<ApplicationSecret>>>();
const application_id = ref<null | UUID>(null);

function createNew(event: Event) {
  event.stopImmediatePropagation();
  event.preventDefault();

  const name = prompt('Create a new secret key, name?');
  if (!name) {
    return;
  }

  ApplicationSecretService.create({
    application_id: application_id.value as string,
    name: name,
  }).then(() => _forceLoad(), displayError);
}

function _forceLoad() {
  application_id.value = route.params.application_id as UUID;
  application_secrets$.value = ApplicationSecretService.list(application_id.value);
}

function _load() {
  if (application_id.value !== route.params.application_id) {
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
  <Promised :promise="application_secrets$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="application_secrets">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> API Keys </template>
          <template #subtitle>
            These keys will allow you to authenticate API requests to Hook0.
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="application_secrets.length > 0">
          <Hook0Table
            :context="{ application_secrets$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="application_secrets"
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
            >Create new API Key
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
