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
          <template #header>
            Applications
          </template>
          <template #subtitle>
            Each application send events to Hook0 API and Hook0 dispatch these extends to customers through webhooks
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="applications.length > 0">
          <transition name="ease">
            <hook0-table
              :columnDefs="columnDefs"
              :rowData="applications"
            >
            </hook0-table>
          </transition>
        </hook0-card-content>

        <hook0-card-footer>
          <hook0-button class="primary" type="button"
                        :to="{
            name: routes.ApplicationsNew,
            params:{
            organization_id: $route.params.organization_id
          }}">Create new
            application
          </hook0-button>
        </hook0-card-footer>
      </hook0-card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <p>Error: {{ error.message }}</p>
    </template>
  </Promised>
</template>

<script lang="ts">
import {Application, list} from './ApplicationService';
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
import {VueElement} from "vue";
import {ColDef} from "@ag-grid-community/core";
import {UUID} from "@/http";

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
})
export default class ApplicationList extends Vue {
  private applications$ !: Promise<Array<Application>>;
  private organization_id: null | UUID = null;

  private routes = routes;

  private columnDefs: Array<ColDef> = [
    {
      field: 'name',
      suppressMovable: true,
      headerName: 'Name',
      cellRenderer: "Hook0TableCellLink",
      cellRendererParams: {
        to: (row: Application) => {
          return {
            name: routes.ApplicationsDashboard,
            params: {
              application_id: row.application_id,
              organization_id: row.organization_id,
            }
          }
        }
      }

    }, {
      field: 'application_id',
      suppressMovable: true,
      headerName: 'Id',
    }, {
      suppressMovable: true,
      headerName: 'Options',
    }];


  data() {
    return {
      applications$: Promise.resolve(),
    }
  }

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }


  _load() {
    if (this.organization_id !== this.$route.params.organization_id) {
      this.organization_id = this.$route.params.organization_id as UUID;
      this.applications$ = list(this.$route.params.organization_id as string);
    }
  }
};
</script>

