<template>
  <div>
    <form @submit="upsert">
      <hook0-card>
        <hook0-card-header>
          <template #header v-if="isNew"> Create new application </template>
          <template #header v-else> Edit application </template>
          <template #subtitle>
            An application emit events that are consumed by customers through webhooks
          </template>
        </hook0-card-header>
        <hook0-card-content>
          <hook0-card-content-line>
            <template #label> Application Name </template>
            <template #content>
              <hook0-input
                type="text"
                v-model="application.name"
                placeholder="my awesome api - production"
                required
              >
                <template #helpText
                  >Name of your company's product or API. Don't forget also to specify the
                  environment, for example: "facebook-production"
                </template>
              </hook0-input>
            </template>
          </hook0-card-content-line>
        </hook0-card-content>

        <hook0-card-content v-if="alert.visible">
          <hook0-alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></hook0-alert>
        </hook0-card-content>
        <hook0-card-footer>
          <hook0-button class="secondary" type="button" @click="cancel2()">Cancel</hook0-button>
          <hook0-button class="primary" type="button" @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </hook0-button>
        </hook0-card-footer>
      </hook0-card>
    </form>

    <ApplicationsRemove
      v-if="!isNew"
      :application-id="application_id"
      :application-name="application.name"
    ></ApplicationsRemove>
  </div>
</template>

<script lang="ts">
import { AxiosError } from 'axios';
import * as ApplicationService from './ApplicationService';
import { Application } from './ApplicationService';
import { Options, Vue } from 'vue-class-component';
import { routes } from '@/routes';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { isAxiosError, Problem, UUID } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import ApplicationsRemove from '@/pages/organizations/applications/ApplicationsRemove.vue';

@Options({
  components: {
    ApplicationsRemove,
    Hook0Alert,
  },
})
export default class ApplicationEdit extends Vue {
  private isNew = true;

  application_id: UUID | null = null;

  routes = routes;

  application = {
    name: '',
  };

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _load() {
    if (this.application_id !== this.$route.params.application_id) {
      this.application_id = this.$route.params.application_id as UUID;
      this.isNew = !this.application_id;

      if (!this.isNew) {
        ApplicationService.get(this.application_id)
          .then((application: Application) => {
            this.application.name = application.name;
          })
          .catch(this.displayError.bind(this));
      }
    }
  }

  cancel2() {
    this.$router
      .push({
        name: routes.OrganizationsDashboard,
        params: {
          organization_id: this.$route.params.organization_id,
        },
      })
      .catch((err) => {
        // do nothing
      });
  }

  upsert(e: Event) {
    e.preventDefault();
    e.stopImmediatePropagation();

    this.alert.visible = false; // reset alert

    if (this.isNew) {
      ApplicationService.create({
        name: this.application.name,
        organization_id: this.$route.params.organization_id as string,
      }).then((_resp: any) => {
        this.cancel2();
      }, this.displayError.bind(this));
      return;
    }

    ApplicationService.update(this.application_id as UUID, {
      name: this.application.name,
      organization_id: this.$route.params.organization_id as string,
    }).then((_resp: any) => {
      this.cancel2();
    }, this.displayError.bind(this));
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

<style scoped>
</style>
