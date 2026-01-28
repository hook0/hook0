<script setup lang="ts">
import { ColDef, ValueFormatterParams, ValueGetterParams } from 'ag-grid-community';
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
import Hook0Button from '@/components/Hook0Button.vue';
import * as SubscriptionService from './SubscriptionService';
import { Subscription } from './SubscriptionService';
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
    field: 'is_enabled',
    suppressMovable: true,
    sortable: true,
    suppressSizeToFit: true,
    width: 115,
    headerName: 'Enabled',
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: (subscription: Subscription) => (subscription.is_enabled ? 'Enabled' : 'Disabled'),
      icon: (subscription: Subscription) => (subscription.is_enabled ? 'toggle-on' : 'toggle-off'),
      onClick: (row: Subscription): void => {
        // If disabling, ask for confirmation
        if (row.is_enabled) {
          const subscriptionName = row.description || 'this subscription';
          if (
            !confirm(
              `Are you sure you want to disable ${subscriptionName}? All pending and scheduled webhook deliveries will be marked as failed and stay in this state even if you re-enable the subscription later.`
            )
          ) {
            return;
          }
        }
        SubscriptionService.toggleEnable(row.subscription_id, row)
          .then(() => {
            // @TODO notify user of success
            _forceLoad();
          })
          // @TODO proper error management
          .catch((err) => {
            alert(err);
            throw err;
          });
      },
    },
  },
  {
    field: 'description',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    minWidth: 100,
    headerName: 'Description',
    valueGetter: (params: ValueGetterParams<Subscription, string>) => {
      return params.data?.description ?? '[no description]';
    },
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      to: (row: Subscription) => {
        return {
          name: routes.SubscriptionsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            subscription_id: row.subscription_id,
          },
        };
      },
    },
  },
  {
    field: 'event_types',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    minWidth: 200,
    headerName: 'Event Types',
    valueFormatter: (params: ValueFormatterParams<Subscription, string[]>) => {
      return params.value?.join(', ') ?? '';
    },
  },
  {
    field: 'labels',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    width: 100,
    headerName: 'Labels',
    cellRenderer: Hook0TableCellCode,
    cellRendererParams: {
      value(row: Subscription) {
        return Object.entries(row.labels as Record<string, string>)
          .map(([key, value]) => `${key}=${value}`)
          .join(' ');
      },
    },
    // This seems useless but triggers a warning if not set
    valueFormatter: () => 'unreachable',
  },
  {
    field: 'target',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Target',
    minWidth: 300,
    cellRenderer: Hook0TableCellCode,
    cellRendererParams: {
      value: (data: Subscription) => {
        const target = (data.target as unknown) ?? {};
        if (targetIsHttp(target)) {
          return `${target.method} ${target.url}`;
        } else {
          return JSON.stringify(data.target);
        }
      },
    },
    // This seems useless but triggers a warning if not set
    valueFormatter: () => 'unreachable',
  },
  {
    field: 'updated_at',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Updated',
    width: 180,
    valueFormatter: (params: ValueFormatterParams<Subscription, string>) => {
      if (!params.value) return '';
      const date = new Date(params.value);
      return date.toLocaleString();
    },
  },
  {
    suppressMovable: true,
    headerName: 'Options',
    cellRenderer: Hook0TableCellLink,
    suppressSizeToFit: true,
    maxWidth: 105,
    cellRendererParams: {
      value: 'Delete',
      icon: 'trash',
      onClick: (row: Subscription): void => {
        if (
          confirm(
            `Are you sure to delete ${
              row.description ? `"${row.description}"` : 'this'
            } subscription?`
          )
        ) {
          SubscriptionService.remove(application_id.value as string, row.subscription_id)
            .then(() => {
              // @TODO notify user of success
              _forceLoad();
            })
            // @TODO proper error management
            .catch((err) => {
              alert(err);
              throw err;
            });
        }
      },
    },
  },
];

const subscriptions$ = ref<Promise<Array<Subscription>>>();
const application_id = ref<null | UUID>(null);

function targetIsHttp(target: object): target is { type: string; method: string; url: string } {
  return target && 'type' in target && target.type === 'http';
}

function _forceLoad() {
  application_id.value = route.params.application_id as UUID;
  subscriptions$.value = SubscriptionService.list(application_id.value);
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
  <Promised :promise="subscriptions$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="subscriptions">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Subscriptions </template>
          <template #subtitle>
            List all subscriptions created by customers against the application events.
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="subscriptions.length > 0">
          <Hook0Table
            :context="{ subscriptions$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="subscriptions"
          >
          </Hook0Table>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text
                  >Your application will send events to Hook0 that will forward these events to
                  registered subscriptions (webhooks), it's time to create your first subscription!
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button
            class="primary"
            type="button"
            @click="$router.push({ name: routes.SubscriptionsNew })"
            >Create new subscription (webhook)
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
