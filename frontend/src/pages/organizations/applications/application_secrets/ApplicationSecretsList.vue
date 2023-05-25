<template>
  <Promised :promise="application_secrets$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <hook0-loader></hook0-loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="application_secrets">
      <hook0-card>
        <hook0-card-header>
          <template #header> API Keys </template>
          <template #subtitle>
            These keys will allow you to authenticate API requests to Hook0.
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="application_secrets.length > 0">
          <hook0-table :context="ctx" :columnDefs="columnDefs" :rowData="application_secrets">
          </hook0-table>
        </hook0-card-content>

        <hook0-card-content v-else>
          <hook0-card-content-lines>
            <hook0-card-content-line type="full-width">
              <template #content>
                <hook0-text
                  >Hook0 authenticates your API requests using your application’s API keys. If you
                  don’t include your key when making an API request, or use an incorrect or outdated
                  one, Hook0 returns a 401 -
                  <hook0-text class="code">Unauthorized HTTP response code</hook0-text>
                  .
                </hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content-lines>
        </hook0-card-content>

        <hook0-card-footer>
          <hook0-button class="primary" type="button" @click="createNew"
            >Create new API Key
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
import { routes } from '@/routes';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import { ColDef } from '@ag-grid-community/core';
import * as ApplicationSecretService from './ApplicationSecretService';
import { isAxiosError, Problem, UUID } from '@/http';
import { ApplicationSecret } from './ApplicationSecretService';
import { AxiosError } from 'axios';
import { Alert } from '@/components/Hook0Alert';

@Options({
  components: {
    Hook0Card,
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
export default class ApplicationSecretsList extends Vue {
  private application_secrets$!: Promise<Array<ApplicationSecret>>;
  public application_id: UUID | null = null;

  public ctx = this;

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

  createNew(event: Event) {
    event.stopImmediatePropagation();
    event.preventDefault();

    const name = prompt('Create a new secret key, name?');
    if (!name) {
      return;
    }

    ApplicationSecretService.create({
      application_id: this.application_id as string,
      name: name,
    }).then(() => this._forceLoad(), this.displayError.bind(this));
  }

  data() {
    return {
      routes: routes,
      application_secrets$: Promise.resolve(),
      columnDefs: [
        {
          field: 'name',
          suppressMovable: true,
          sortable: true,
          headerName: 'Name',
        },
        {
          field: 'token',
          suppressMovable: true,
          sortable: true,
          cellRenderer: 'Hook0TableCellCode',
          cellRendererParams: {},
          headerName: 'Token',
          width: 240,
        },
        {
          field: 'created_at',
          suppressMovable: true,
          sortable: true,
          headerName: 'Created At',
        },
        {
          suppressMovable: true,
          headerName: 'Options',
          width: 100,
          cellRenderer: 'Hook0TableCellLink',
          cellRendererParams: {
            value: 'Delete',
            icon: 'trash',
            onClick: (row: ApplicationSecret): void => {
              if (confirm(`Are you sure to delete "${row.name as string}" API Key?`)) {
                ApplicationSecretService.remove(this.application_id as string, row.token)
                  .then(() => {
                    debugger;
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
    this.application_secrets$ = ApplicationSecretService.list(this.application_id);
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

  displayError(err: AxiosError | unknown) {
    console.error(err);
    this.alert.visible = true;

    if (isAxiosError(err) && err.response) {
      const problem: Problem = err.response.data as Problem;
      this.alert.type = problem.status >= 500 ? 'alert' : 'warning';
      this.alert.title = problem.title;
      this.alert.description = problem.detail;
    } else {
      this.alert.type = 'alert';
      this.alert.title = 'An error occurred';
      this.alert.description = String(err);
    }
  }
}
</script>
