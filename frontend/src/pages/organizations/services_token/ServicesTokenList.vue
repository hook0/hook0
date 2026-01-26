<script setup lang="ts">
import { ColDef } from 'ag-grid-community';
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
import { useTracking } from '@/composables/useTracking';

const route = useRoute();

// Analytics tracking
const { trackEvent } = useTracking();

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
            trackEvent('service-token', 'update', 'success');
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
        if (
          confirm(
            'Are you sure you want to delete this service token?\n\nEvery token derived from this token will be revoked as well.'
          )
        ) {
          ServiceTokenService.remove(row.token_id, organization_id.value as string)
            .then(() => {
              trackEvent('service-token', 'delete', 'success');
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
    .then((_resp) => {
      trackEvent('service-token', 'create', 'success');
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
      <!-- Service Tokens List -->
      <Hook0Card class="mb-4" data-test="service-tokens-card">
        <Hook0CardHeader>
          <template #header> Service Tokens </template>
          <template #subtitle>
            Service tokens are organization-wide API keys that allow external programs to send API
            requests to Hook0 without relying on user credentials.
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="services_tokens.length > 0">
          <Hook0Table
            data-test="service-tokens-table"
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
                <Hook0Text>
                  You don't have any service tokens yet. Create one to start integrating with
                  Hook0's API or to connect an AI assistant.
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button class="primary" type="button" data-test="service-tokens-create-button" @click="createNew">
            <Hook0Icon name="plus" class="mr-1"></Hook0Icon>
            Create Service Token
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>

      <!-- AI Integration Banner -->
      <Hook0Card>
        <div
          class="bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 rounded-t-lg px-6 py-4"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-3">
              <div class="bg-white/20 rounded-full p-2">
                <Hook0Icon name="robot" class="text-white text-xl"></Hook0Icon>
              </div>
              <div>
                <h3 class="text-white font-semibold text-lg">Use Hook0 with AI Assistants</h3>
                <p class="text-white/80 text-sm">
                  Connect Claude, ChatGPT, or any MCP-compatible AI to manage your webhooks
                </p>
              </div>
            </div>
            <Hook0Button
              class="bg-white text-indigo-600 hover:bg-indigo-50 font-medium px-4 py-2 rounded-lg shadow-sm"
              href="https://documentation.hook0.com/reference/mcp-for-ia-assistant"
              target="_blank"
            >
              <Hook0Icon name="book" class="mr-2"></Hook0Icon>
              Setup Guide
            </Hook0Button>
          </div>
        </div>
        <Hook0CardContent>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <div class="flex flex-col space-y-3">
                  <Hook0Text class="text-gray-600">
                    With <strong>Hook0 MCP Server</strong>, you can let AI assistants interact with
                    your webhook infrastructure using natural language:
                  </Hook0Text>
                  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-2">
                    <div class="flex items-start space-x-2">
                      <Hook0Icon name="check" class="text-green-500 mt-1"></Hook0Icon>
                      <span class="text-sm text-gray-600"
                        >List and inspect events, subscriptions, and delivery history</span
                      >
                    </div>
                    <div class="flex items-start space-x-2">
                      <Hook0Icon name="check" class="text-green-500 mt-1"></Hook0Icon>
                      <span class="text-sm text-gray-600"
                        >Create and manage webhook subscriptions via conversation</span
                      >
                    </div>
                    <div class="flex items-start space-x-2">
                      <Hook0Icon name="check" class="text-green-500 mt-1"></Hook0Icon>
                      <span class="text-sm text-gray-600"
                        >Debug failed deliveries and retry them instantly</span
                      >
                    </div>
                  </div>
                  <div
                    class="mt-4 p-3 bg-amber-50 border border-amber-200 rounded-lg flex items-start space-x-2"
                  >
                    <Hook0Icon name="key" class="text-amber-600 mt-0.5"></Hook0Icon>
                    <span class="text-sm text-amber-800">
                      <strong>To get started:</strong> Create a service token above and use it as
                      your <code class="bg-amber-100 px-1 rounded">HOOK0_API_TOKEN</code> in your
                      MCP configuration.
                    </span>
                  </div>
                </div>
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
