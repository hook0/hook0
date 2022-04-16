<template>
  <div>
    <hook0-card class="mb-4">
      <hook0-card-header>
        <template #header>
          Application
          <hook0-text class="bold">{{ application.name }}</hook0-text>
          dashboard
        </template>
        <template #subtitle>
          (Coming soon) here Hook0 will display metrics about webhooks & events usage
        </template>
        <template #actions>
          <hook0-button
            :to="{ name: routes.ApplicationsDetail, params:{
            organization_id: $route.params.organization_id,
             application_id: $route.params.application_id
          }}">
            Edit
          </hook0-button>
        </template>

      </hook0-card-header>

    </hook0-card>
    <EventTypesList :burst="$route.params.application_id"></EventTypesList>
  </div>
</template>

<script lang="ts">
import {Options, Vue} from "vue-class-component";
import Hook0Text from "@/components/Hook0Text.vue";
import {isAxiosError, Problem, UUID} from "@/http";
import * as ApplicationService from "./ApplicationService";
import {Application} from "./ApplicationService";
import {RouteParamsRaw} from "vue-router";
import {AxiosError} from "axios";
import {Alert} from "@/components/Hook0Alert";
import {routes} from "@/routes";
import EventTypesList from "@/pages/organizations/applications/event_types/EventTypesList.vue";

@Options({
  components: {EventTypesList, Hook0Text},
})
export default class ApplicationsDashboard extends Vue {
  application_id: UUID | undefined;

  private routes = routes;

  application = {
    name: '',
  };

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _load() {
    if (this.application_id !== this.$route.params.application_id) {
      // @ts-ignore
      this.application_id = this.$route.params.application_id as UUID;

      // @ts-ignore
      ApplicationService.get(this.application_id).then((application: Application) => {
        this.application.name = application.name;
      }).catch(this.displayError.bind(this));
    }
  }

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

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
      this.alert.title = "An error occurred";
      this.alert.description = String(err);
    }
  }
};


</script>

<style scoped>
</style>
