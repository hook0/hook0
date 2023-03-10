<template>
  <Promised :promise="request_attempts$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <hook0-loader></hook0-loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="request_attempts">
      <hook0-card>
        <hook0-card-header>
          <template #header> Logs (request attempts)</template>
          <template #subtitle>
            Requests that Hook0 sent to
            <hook0-button :to="{ name: routes.SubscriptionsList }">subscriptions</hook0-button>
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="request_attempts.length > 0">
          <hook0-table :context="this" :columnDefs="columnDefs" :rowData="request_attempts">
          </hook0-table>
        </hook0-card-content>

        <hook0-card-content v-else>
          <hook0-card-content-lines>
            <hook0-card-content-line type="full-width">
              <template #content>
                <hook0-text class="center block" style="text-align: center"
                  >Hook0 did not send any requests. Did you setup a
                  <hook0-button :to="routes.SubscriptionsList">subscriptions</hook0-button>
                  and
                  <a href="https://documentation.hook0.com/docs/getting-started" target="_blank"
                    >sent your first event</a
                  >?
                </hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content-lines>
        </hook0-card-content>

        <hook0-card-footer></hook0-card-footer>
      </hook0-card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <hook0-error :error="error"></hook0-error>
    </template>
  </Promised>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import Hook0Button from '@/components/Hook0Button.vue';
import { routes } from '@/routes';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import { ColDef, ValueFormatterParams } from '@ag-grid-community/core';
import * as LogService from './LogService';
import {
  RequestAttempt,
  RequestAttemptStatus,
  RequestAttemptStatusType,
  RequestAttemptTypeFixed,
} from './LogService';
import { UUID } from '@/http';
import { format, formatDistance, formatRelative, parseISO, subDays } from 'date-fns';

@Options({
  components: {
    Hook0CardContentLine,
    Hook0CardContent,
    Hook0CardFooter,
    Hook0CardHeader,
    Hook0Card,
    Hook0Input,
    Hook0Button,
    Hook0Table,
  },
  props: {
    // cache-burst
    burst: {
      type: String,
      required: false,
    },
  },
})
export default class EventsList extends Vue {
  private request_attempts$!: Promise<Array<RequestAttemptTypeFixed>>;
  public application_id: UUID | null = null;

  data() {
    // eslint-disable-next-line
    const ctx = this;
    return {
      routes: routes,
      request_attempts$: Promise.resolve(),
      columnDefs: [
        {
          field: 'status',
          suppressMovable: true,
          suppressSizeToFit: true,
          width: 20,
          sortable: true,
          headerName: 'Status',
          cellRenderer: 'Hook0TableCellIcon',
          cellRendererParams: {
            title(row: RequestAttemptTypeFixed) {
              switch (row.status.type) {
                // @todo(i18n) change when/if we want to support i18n
                case RequestAttemptStatusType.Waiting:
                  return 'Event waiting to be picked';
                case RequestAttemptStatusType.Pending:
                  return 'Event picked and waiting to be dispatched';
                case RequestAttemptStatusType.InProgress:
                  return 'Event is currently being sent';
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

                case RequestAttemptStatusType.Waiting: // waiting to be picked
                case RequestAttemptStatusType.Pending: // event picked and waiting to be dispatched
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
        },
        {
          field: 'created_at',
          suppressMovable: true,
          suppressSizeToFit: true,
          width: 150,
          sortable: true,
          headerName: 'Created At',
          valueFormatter: (date: ValueFormatterParams<RequestAttemptTypeFixed, string>) =>
            formatDistance(parseISO(date.value), new Date(), { addSuffix: true }),
        },
        {
          field: 'picked_at',
          suppressMovable: true,
          suppressSizeToFit: true,
          width: 150,
          sortable: true,
          headerName: 'Picked At',
          valueFormatter: (date: ValueFormatterParams<RequestAttemptTypeFixed, string>) =>
            formatDistance(parseISO(date.value), new Date(), { addSuffix: true }),
        },
        {
          field: 'subscription id',
          suppressMovable: true,
          suppressSizeToFit: false,
          width: 150,
          sortable: true,
          headerName: 'Subscription',
          cellRenderer: 'Hook0TableCellLink',
          cellRendererParams: {
            value(row: RequestAttemptTypeFixed) {
              return row.subscription.description;
            },
            to(row: RequestAttemptTypeFixed) {
              return {
                name: routes.SubscriptionsDetail,
                params: {
                  // eslint-disable-next-line
                  application_id: ctx.$route.params.application_id,
                  // eslint-disable-next-line
                  organization_id: ctx.$route.params.organization_id,
                  // eslint-disable-next-line
                  subscription_id: row.subscription.subscription_id,
                },
              };
            },
          },
        },
      ] as Array<ColDef>,
    };
  }

  _forceLoad() {
    this.application_id = this.$route.params.application_id as UUID;
    this.request_attempts$ = LogService.list(this.application_id);
  }

  _load() {
    // @ts-ignore
    if (this.application_id !== this.$route.params.application_id) {
      this._forceLoad();
    }
  }

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }
}
</script>
