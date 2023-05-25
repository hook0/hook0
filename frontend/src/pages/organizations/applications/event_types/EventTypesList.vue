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
          <template #header> Event types </template>
          <template #subtitle>
            Each event sent through a webhook must have an event type.
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="event_types.length > 0">
          <hook0-table :context="this" :columnDefs="columnDefs" :rowData="event_types">
          </hook0-table>
        </hook0-card-content>

        <hook0-card-content v-else>
          <hook0-card-content-lines>
            <hook0-card-content-line type="full-width">
              <template #content>
                <hook0-text
                  >Your application will send events to Hook0 that will forward these events to
                  registered subscriptions (webhooks), each of these event must have a type (e.g.
                  "billing.invoice.created"), it's time to create your first event type!
                </hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content-lines>
        </hook0-card-content>

        <hook0-card-footer>
          <hook0-button
            class="primary"
            type="button"
            @click="$router.push({ name: routes.EventTypesNew })"
            >Create new event type
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
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import { ColDef } from '@ag-grid-community/core';
import * as EventTypeService from './EventTypeService';
import { UUID } from '@/http';
import { EventType } from './EventTypeService';

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
export default class EventTypesList extends Vue {
  private event_types$!: Promise<Array<EventType>>;
  public application_id: UUID | null = null;

  data() {
    return {
      routes: routes,
      event_types$: Promise.resolve(),
      columnDefs: [
        {
          field: 'event_type_name',
          suppressMovable: true,
          cellRenderer: 'Hook0TableCellCode',
          minWidth: 360,
          sortable: true,
          headerName: 'Name',
        },
        {
          suppressMovable: true,
          width: 100,
          headerName: 'Options',
          cellRenderer: 'Hook0TableCellLink',
          cellRendererParams: {
            value: 'Delete',
            icon: 'trash',
            onClick: (row: EventType): void => {
              if (confirm(`Are you sure to delete "${row.event_type_name}" event?`)) {
                EventTypeService.remove(this.application_id as string, row.event_type_name)
                  .then(() => {
                    // @TODO notify user of success
                    this._forceLoad();
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
      ] as Array<ColDef>,
    };
  }

  _forceLoad() {
    this.application_id = this.$route.params.application_id as UUID;
    this.event_types$ = EventTypeService.list(this.application_id);
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
