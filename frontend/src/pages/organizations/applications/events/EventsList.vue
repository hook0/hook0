<template>
  <Promised :promise="event_types$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <hook0-loader></hook0-loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="event_types">
      <hook0-card>
        <hook0-card-header>
          <template #header>
            Events
          </template>
          <template #subtitle>
            Events that Hook0 receive from your application and that Hook0 forwarded to subscriptions (webhooks).
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="event_types.length > 0">
          <hook0-table
            :context="this"
            :columnDefs="columnDefs"
            :rowData="event_types"
          >
          </hook0-table>
        </hook0-card-content>

        <hook0-card-content v-else>
          <hook0-card-content-lines>
            <hook0-card-content-line type="full-width">
              <template #content>
                <hook0-text class="center block" style="text-align: center">Your application did not send any events.
                  Time to send the first one!
                </hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content-lines>
        </hook0-card-content>

        <hook0-card-footer>
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
import {ColDef} from "@ag-grid-community/core";
import * as EventsService from "./EventsService";
import {Event} from "./EventsService";
import {UUID} from "@/http";
import {Application} from "@/pages/organizations/applications/ApplicationService";
import * as ApplicationService from "@/pages/organizations/applications/ApplicationService";

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
export default class EventsList extends Vue {
  private event_types$ !: Promise<Array<Event>>;
  public application_id: UUID | null = null;


  data() {
    // eslint-disable-next-line
    const ctx = this;
    return {
      routes: routes,
      event_types$: Promise.resolve(),
      columnDefs: [
        {
          field: 'event_type_name',
          headerName: 'Event Type',
          suppressMovable: true,
          cellRenderer: "Hook0TableCellLink",
          cellRendererParams: {
            value(row: Event) {
              return row.event_type_name;
            },

            to(row: Event) {
              return {
                name: routes.EventsDetail,
                params: {
                  // eslint-disable-next-line
                  application_id: ctx.$route.params.application_id,
                  // eslint-disable-next-line
                  organization_id: ctx.$route.params.organization_id,
                  // eslint-disable-next-line
                  event_id: row.event_id
                }
              };
            }
          }
        }, {
          field: 'ip',
          suppressMovable: true,
          sortable: true,
          headerName: 'IP'
        }, {
          field: 'labels',
          suppressMovable: true,
          sortable: true,
          headerName: 'Labels'
        }, {
          field: 'payload_content_type_name',
          suppressMovable: true,
          sortable: true,
          headerName: 'Payload type'
        }, {
          field: 'received_at',
          suppressMovable: true,
          minWidth: 360,
          sortable: true,
          headerName: 'Received At'
        }] as Array<ColDef>
    }
  }

  _forceLoad() {
    this.application_id = this.$route.params.application_id as UUID;
    this.event_types$ = EventsService.list(this.application_id);
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

