<template>
  <div>
    <hook0-card>
      <hook0-card-header>
        <template #header>
          <hook0-icon name="sitemap"></hook0-icon>
          Organization
          <hook0-text class="bold">{{ organization.name }}</hook0-text>
          <template v-if="isPricingEnabled">
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
    <hook0-card v-if="isPricingEnabled && !organization.plan">
      <hook0-card-header>
        <template #header>
          <hook0-icon name="money-check-dollar"></hook0-icon>
          Your organization is on the <strong>Developer</strong> plan!
        </template>
      </hook0-card-header>

      <hook0-card-content>
        <hook0-card-content-lines>
          <hook0-card-content-line type="full-width">
            <template #content>
              <hook0-text>You are currently limited to:</hook0-text>
              <hook0-list>
                <hook0-list-item>
                  <template #left>
                    <hook0-icon name="users" class="mr-1"></hook0-icon>
                    <hook0-text>
                      <strong>{{ organization.quotas.members_per_organization_limit }}</strong>
                      member{{ organization.quotas.members_per_organization_limit > 1 ? 's' : '' }}
                    </hook0-text>
                  </template>
                </hook0-list-item>
                <hook0-list-item>
                  <template #left>
                    <hook0-icon name="folder" class="mr-1"></hook0-icon>
                    <hook0-text>
                      <strong>{{ organization.quotas.applications_per_organization_limit }}</strong>
                      application{{
                        organization.quotas.applications_per_organization_limit > 1 ? 's' : ''
                      }}
                    </hook0-text>
                  </template>
                </hook0-list-item>
                <hook0-list-item>
                  <template #left>
                    <hook0-icon name="file-lines" class="mr-1"></hook0-icon>
                    <hook0-text>
                      <strong>{{ organization.quotas.events_per_day_limit }}</strong>
                      event{{ organization.quotas.events_per_day_limit > 1 ? 's' : '' }} per day
                    </hook0-text>
                  </template>
                </hook0-list-item>
                <hook0-list-item>
                  <template #left>
                    <hook0-icon name="database" class="mr-1"></hook0-icon>
                    <hook0-text>
                      <strong>{{ organization.quotas.days_of_events_retention_limit }}</strong>
                      day{{ organization.quotas.days_of_events_retention_limit > 1 ? 's' : '' }} of
                      event retention
                    </hook0-text>
                  </template>
                </hook0-list-item>
              </hook0-list>
            </template>
          </hook0-card-content-line>
        </hook0-card-content-lines>
      </hook0-card-content>

      <hook0-card-footer>
        <hook0-button
          class="secondary"
          type="button"
          href="https://www.hook0.com/#pricing"
          target="_blank"
          >Available plans</hook0-button
        >
        <hook0-button class="primary" type="button" href="mailto:support@hook0.com"
          >Subscribe to a better plan
        </hook0-button>
      </hook0-card-footer>
    </hook0-card>
    <ApplicationsList :burst="$route.params.organization_id"> </ApplicationsList>
  </div>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import Hook0Text from '@/components/Hook0Text.vue';
import { isAxiosError, Problem, UUID } from '@/http';
import * as OrganizationService from '@/pages/organizations/OrganizationService';
import { OrganizationInfo } from '@/pages/organizations/OrganizationService';
import { RouteParamsRaw } from 'vue-router';
import { AxiosError } from 'axios';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';
import { routes } from '@/routes';
import { Alert } from '@/components/Hook0Alert';
import { isPricingEnabled } from '@/pricing';

@Options({
  components: {
    ApplicationsList,
    Hook0CardContentLine,
    Hook0CardContent,
    Hook0Text,
    Hook0List,
    Hook0ListItem,
  },
})
export default class OrganizationsDashboard extends Vue {
  isPricingEnabled = isPricingEnabled();
  organization_id: UUID | null = null;

  organization = {
    name: '',
    plan: '',
    quotas: {
      members_per_organization_limit: 0,
      applications_per_organization_limit: 0,
      events_per_day_limit: 0,
      days_of_events_retention_limit: 0,
    },
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
        .then((organization: OrganizationInfo) => {
          this.organization.name = organization.name;
          this.organization.plan = organization.plan?.label || '';
          this.organization.quotas = organization.quotas;
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
