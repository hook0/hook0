<script setup lang="ts">
import { ColDef } from 'ag-grid-community';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import { UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import { routes } from '@/routes';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0TableCellIcon from '@/components/Hook0TableCellIcon.vue';
import * as LogService from './LogService';
import { RequestAttempt, RequestAttemptStatusType, RequestAttemptTypeFixed } from './LogService';
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
    field: 'status',
    suppressMovable: true,
    suppressSizeToFit: true,
    width: 75,
    sortable: true,
    headerName: 'Status',
    cellRenderer: Hook0TableCellIcon,
    cellRendererParams: {
      title(row: RequestAttemptTypeFixed) {
        switch (row.status.type) {
          // @todo(i18n) change when/if we want to support i18n
          case RequestAttemptStatusType.Waiting:
            return 'Request scheduled for later';
          case RequestAttemptStatusType.Pending:
            return 'Request waiting to be picked';
          case RequestAttemptStatusType.InProgress:
            return 'Request is currently being sent';
          case RequestAttemptStatusType.Failed:
            return 'Request had an error';
          case RequestAttemptStatusType.Successful:
            return 'Request successfully sent';
          default:
            return 'Unknown status';
        }
      },
      icon(row: RequestAttemptTypeFixed) {
        switch (row.status.type) {
          // @todo(i18n) change when/if we want to support i18n
          case RequestAttemptStatusType.Waiting:
            return 'fa-calendar';
          case RequestAttemptStatusType.Pending:
            return 'fa-pause';
          case RequestAttemptStatusType.InProgress:
            return 'fa-spinner';
          case RequestAttemptStatusType.Failed:
            return 'fa-xmark';
          case RequestAttemptStatusType.Successful:
            return 'fa-check';
          default:
            return 'fa-question';
        }
      },
    },
    // This seems useless but triggers a warning if not set
    valueFormatter: () => 'unreachable',
  },
  {
    field: 'event_id',
    headerName: 'Event ID',
    suppressMovable: true,
    resizable: true,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value(row: RequestAttempt) {
        return row.event_id;
      },

      to(row: RequestAttempt) {
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
    field: 'subscription id',
    suppressMovable: true,
    suppressSizeToFit: false,
    width: 150,
    sortable: true,
    resizable: true,
    headerName: 'Subscription',
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value(row: RequestAttemptTypeFixed) {
        return row.subscription.description;
      },
      to(row: RequestAttemptTypeFixed) {
        return {
          name: routes.SubscriptionsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            subscription_id: row.subscription.subscription_id,
          },
        };
      },
    },
  },
  {
    field: 'created_at',
    suppressMovable: true,
    suppressSizeToFit: true,
    width: 175,
    sortable: true,
    resizable: true,
    headerName: 'Created At',
    cellRenderer: Hook0TableCellDate,
  },
  {
    field: 'picked_at',
    suppressMovable: true,
    suppressSizeToFit: true,
    width: 175,
    sortable: true,
    resizable: true,
    headerName: 'Picked At',
    cellRenderer: Hook0TableCellDate,
    cellRendererParams: {
      defaultText: 'pending…',
    },
  },
];

const request_attempts$ = ref<Promise<Array<RequestAttemptTypeFixed>>>();
const application_id = ref<null | UUID>(null);

function _forceLoad() {
  application_id.value = route.params.application_id as UUID;
  request_attempts$.value = LogService.list(application_id.value);
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
  <Promised :promise="request_attempts$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="request_attempts">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>Request Attempts</template>
          <template #subtitle>
            Last webhooks sent by Hook0.
            <em>Items older than 7 days are not shown.</em>
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="request_attempts.length > 0">
          <Hook0Table
            :context="{ request_attempts$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="request_attempts"
          >
          </Hook0Table>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text class="center block" style="text-align: center"
                  >Hook0 did not send any requests. Did you setup a
                  <Hook0Button :to="{ name: routes.SubscriptionsList }">subscriptions</Hook0Button>
                  and
                  <a href="https://documentation.hook0.com/docs/getting-started" target="_blank"
                    >sent your first event</a
                  >?
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter></Hook0CardFooter>
      </Hook0Card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
</template>
