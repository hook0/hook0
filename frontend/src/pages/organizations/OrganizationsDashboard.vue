<template>
  <div>
    <hook0-card>
      <hook0-card-header>
        <template #header>
          <hook0-icon name="sitemap"></hook0-icon>
          Organization
          <hook0-text class="bold">{{ organization.name }}</hook0-text>
          <span
            v-if="organization.plan"
            class="ml-2 inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10"
            :title="'Plan: ' + organization.plan"
            >{{ organization.plan }}</span
          >
          <span
            v-else
            class="ml-2 inline-flex items-center rounded-md bg-gray-50 px-2 py-1 text-xs font-medium text-gray-600 ring-1 ring-inset ring-gray-500/10"
            title="Plan: Developer"
            >Developer</span
          >
        </template>
        <template #subtitle> </template>
        <template #actions>
          <hook0-button
            :to="{
              name: routes.OrganizationsDetail,
              params: { organization_id: $route.params.organization_id },
            }"
          >
            Settings
          </hook0-button>
        </template>
      </hook0-card-header>
    </hook0-card>
    <ApplicationsList :burst="$route.params.organization_id"> </ApplicationsList>
  </div>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import Hook0Text from '@/components/Hook0Text.vue';
import { isAxiosError, Problem, UUID } from '@/http';
import * as OrganizationService from '@/pages/organizations/OrganizationService';
import { Organization } from '@/pages/organizations/OrganizationService';
import { RouteParamsRaw } from 'vue-router';
import { AxiosError } from 'axios';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';
import { routes } from '@/routes';
import { Alert } from '@/components/Hook0Alert';

@Options({
  components: { ApplicationsList, Hook0CardContentLine, Hook0CardContent, Hook0Text },
})
export default class OrganizationsDashboard extends Vue {
  organization_id: UUID | null = null;

  organization = {
    name: '',
    plan: '',
  };

  routes = routes;

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _load() {
    if (this.organization_id !== this.$route.params.organization_id) {
      this.organization_id = this.$route.params.organization_id as UUID;
      OrganizationService.get(this.organization_id)
        .then((organization: Organization) => {
          this.organization.name = organization.name;
          this.organization.plan = organization.plan?.label || '';
        })
        .catch(this.displayError.bind(this));
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
      this.alert.title = 'An error occurred';
      this.alert.description = String(err);
    }
  }
}
</script>

<style scoped></style>
