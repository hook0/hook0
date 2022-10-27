<template>
  <Promised :promise="subscriptions$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <hook0-loader></hook0-loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="subscriptions">
      <hook0-card>
        <hook0-card-header>
          <template #header>
            Subscriptions
          </template>
          <template #subtitle>
            List all subscriptions created by customers against the application events
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="subscriptions.length > 0">
          <hook0-table
              :context="this"
              :columnDefs="columnDefs"
              :rowData="subscriptions"
          >
          </hook0-table>
        </hook0-card-content>

        <hook0-card-content v-else>
          <hook0-card-content-lines>
            <hook0-card-content-line type="full-width">
              <template #content>
                <hook0-text>Your application will send events to Hook0 that will forward these events to registered
                  subscriptions (webhooks), it's time to create your first subscription!
                </hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content-lines>

        </hook0-card-content>

        <hook0-card-footer>
          <hook0-button class="primary" type="button" @click="$router.push({name:routes.SubscriptionsNew})">Create new
            subscription (webhook)
          </hook0-button>
        </hook0-card-footer>
      </hook0-card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <hook0-error :error="error"></hook0-error>
    </template>
  </Promised>
</template>

<script lang="ts">
import {Options, Vue} from 'vue-class-component';
import Hook0Button from "@/components/Hook0Button.vue";
import {routes} from "@/routes";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0Input from "@/components/Hook0Input.vue";
import Hook0Table from "@/components/Hook0Table.vue";
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import {ColDef} from "@ag-grid-community/core";
import * as SubscriptionService from "./SubscriptionService";
import {UUID} from "@/http";
import {Subscription, SubscriptionPostFixed, Target, toggleEnable} from "./SubscriptionService";
import {Application} from "@/pages/organizations/applications/ApplicationService";

@Options({
  components: {
    Hook0CardContentLine,
    Hook0CardContent,
    Hook0CardFooter,
    Hook0CardHeader,
    Hook0Card,
    Hook0Input,
    Hook0Button,
    Hook0Table
  },
  props: {
    // cache-burst
    burst: {
      type: String,
      required: false
    }
  }
})
export default class SubscriptionsList extends Vue {
  private subscriptions$ !: Promise<Array<Subscription>>;
  public application_id: UUID | null = null;


  data() {
    return {
      routes: routes,
      subscriptions$: Promise.resolve(),
      columnDefs: [
        {
          field: 'is_enabled',
          suppressMovable: true,
          sortable: true,
          resizable: true,
          width: 200,
          headerName: 'Enabled',
          cellRenderer: "Hook0TableCellLink",
          cellRendererParams: {
            value: (subscription: Subscription) => subscription.is_enabled ? 'Enabled' : 'Disabled',
            icon: (subscription: Subscription) => subscription.is_enabled ? 'toggle-on' : 'toggle-of',
            onClick: (row: SubscriptionPostFixed): void => {
              // eslint-disable-next-line
              SubscriptionService.toggleEnable(this.application_id as string, row)
                  .then(() => {
                    // @TODO notify user of success
                    this._forceLoad();
                  })
                  // @TODO proper error management
                  .catch(err => {
                    alert(err);
                    throw err;
                  });
            }
          }
        },
        {
          field: 'description',
          suppressMovable: true,
          sortable: true,
          resizable: true,
          minWidth: 200,
          headerName: 'Description',
          cellRenderer: "Hook0TableCellLink",
          cellRendererParams: {
            to: (row: Subscription) => {
              return {
                name: routes.SubscriptionsDetail,
                params: {
                  application_id: this.$route.params.application_id,
                  organization_id: this.$route.params.organization_id,
                  subscription_id: row.subscription_id,
                }
              }
            }
          }
        },
        {
          field: 'event_types',
          suppressMovable: true,
          sortable: true,
          resizable: true,
          minWidth: 200,
          headerName: 'event_types'
        }, {
          field: 'label_key',
          suppressMovable: true,
          sortable: true,
          resizable: true,
          headerName: 'Label key',
        }, {
          field: 'label_value',
          suppressMovable: true,
          sortable: true,
          resizable: true,
          headerName: 'Label value',
        }, {
          field: 'target',
          suppressMovable: true,
          sortable: true,
          headerName: 'Target',
          minWidth: 200,
          valueFormatter: (a) => {
            // @todo set another cellrenderer
            return JSON.stringify(a.value);
          }
        }, {
          suppressMovable: true,
          headerName: 'Options',
          cellRenderer: "Hook0TableCellLinks",
          maxWidth: 200,
          cellRendererParams: {
            parameters: [{
              value: 'Delete',
              icon: 'trash',
              onClick: (row: Subscription): void => {
                if (confirm(`Are you sure to delete ${row.description ? `"${row.description}"` : 'this'} subscription?`)) {
                  SubscriptionService.remove(this.application_id as string, row.subscription_id)
                      .then(() => {
                        // @TODO notify user of success
                        this._forceLoad();
                      })
                      // @TODO proper error management
                      .catch(err => {
                        alert(err);
                        throw err;
                      });
                }
              }
            }]
          }
        }] as Array<ColDef>
    }
  }

  _forceLoad() {
    this.application_id = this.$route.params.application_id as UUID;
    this.subscriptions$ = SubscriptionService.list(this.application_id);
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
};
</script>

