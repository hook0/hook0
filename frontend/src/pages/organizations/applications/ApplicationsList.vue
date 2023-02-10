<template>
  <Promised :promise="applications$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <hook0-loader></hook0-loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="applications">
      <hook0-card>
        <hook0-card-header>
          <template #header> Applications </template>
          <template #subtitle>
            Each application send events to Hook0 API and Hook0 dispatch these extends to customers
            through webhooks
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="applications.length > 0">
          <transition name="ease">
            <hook0-table :context="this" :columnDefs="columnDefs" :rowData="applications">
            </hook0-table>
          </transition>
        </hook0-card-content>

        <hook0-card-content v-else>
          <hook0-card-content-lines>
            <hook0-card-content-line type="full-width">
              <template #content>
                <hook0-text
                  >Start your journey by creating a Hook0 application. This application will have
                  API keys that will be required to send events to Hook0 API so it can dispatch
                  these events to your customers through webhooks.</hook0-text
                >
              </template>
            </hook0-card-content-line>
          </hook0-card-content-lines>
        </hook0-card-content>

        <hook0-card-footer>
          <hook0-button
            class="primary"
            type="button"
            :to="{
              name: routes.ApplicationsNew,
              params: {
                organization_id: $route.params.organization_id,
              },
            }"
            >Create new application
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
import { Application } from './ApplicationService';
import * as ApplicationService from './ApplicationService';
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
import { VueElement } from 'vue';
import { ColDef } from '@ag-grid-community/core';
import { UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';

@Options({
  components: {
    Hook0Text,
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
export default class ApplicationList extends Vue {
  private applications$!: Promise<Array<Application>>;
  private organization_id: null | UUID = null;

  private routes = routes;

  data() {
    return {
      applications$: Promise.resolve(),
      columnDefs: [
        {
          field: 'name',
          suppressMovable: true,
          headerName: 'Name',
          cellRenderer: 'Hook0TableCellLink',
          cellRendererParams: {
            to: (row: Application) => {
              return {
                name: routes.ApplicationsDashboard,
                params: {
                  application_id: row.application_id,
                  organization_id: row.organization_id,
                },
              };
            },
          },
        },
        {
          field: 'application_id',
          suppressMovable: true,
          headerName: 'Id',
        },
        {
          suppressMovable: true,
          headerName: 'Options',
          cellRenderer: 'Hook0TableCellLink',
          cellRendererParams: {
            value: 'Delete',
            icon: 'trash',
            onClick(row: Application) {
              if (confirm(`Are you sure to delete "${row.name}" application?`)) {
                ApplicationService.remove(row.application_id)
                  .then(() => {
                    // @TODO notify user of success
                    // eslint-disable-next-line
                    this._forceLoad();
                  })
                  // @TODO proper error management
                  .catch((err: Error) => {
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

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _forceLoad() {
    this.organization_id = this.$route.params.organization_id as UUID;
    this.applications$ = ApplicationService.list(this.$route.params.organization_id as string);
  }

  _load() {
    if (this.organization_id !== this.$route.params.organization_id) {
      this._forceLoad();
    }
  }
}
</script>
