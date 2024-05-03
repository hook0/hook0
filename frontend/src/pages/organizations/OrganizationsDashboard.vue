<script setup lang="ts">
import { useRoute } from 'vue-router';
import { onMounted, onUpdated, ref } from 'vue';

import Hook0Text from '@/components/Hook0Text.vue';
import { Problem, UUID } from '@/http';
import * as OrganizationService from '@/pages/organizations/OrganizationService';
import { OrganizationInfo } from '@/pages/organizations/OrganizationService';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';
import { routes } from '@/routes';
import { isPricingEnabled } from '@/pricing';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import MembersList from '@/pages/organizations/MembersList.vue';
import { push } from 'notivue';

const route = useRoute();
const pricingEnabled = isPricingEnabled();

const organization_id = ref<UUID | null>(null);
const organization = ref({
  name: '',
  plan: '',
  quotas: {
    members_per_organization_limit: 0,
    applications_per_organization_limit: 0,
    events_per_day_limit: 0,
    days_of_events_retention_limit: 0,
  },
});

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    organization_id.value = route.params.organization_id as UUID;

    OrganizationService.get(organization_id.value)
      .then((org: OrganizationInfo) => {
        organization.value.name = org.name;
        organization.value.plan = org.plan?.label || '';
        organization.value.quotas = org.quotas;
      })
      .catch(displayError);
  }
}

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <div>
    <Hook0Card>
      <Hook0CardHeader>
        <template #header>
          <Hook0Icon name="sitemap"></Hook0Icon>
          Organization
          <Hook0Text class="bold">{{ organization.name }}</Hook0Text>
          <template v-if="pricingEnabled">
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
          <Hook0Button
            :to="{
              name: routes.OrganizationsDetail,
              params: { organization_id: $route.params.organization_id },
            }"
          >
            Settings
          </Hook0Button>
        </template>
      </Hook0CardHeader>
    </Hook0Card>

    <Hook0Card v-if="pricingEnabled && !organization.plan">
      <Hook0CardHeader>
        <template #header>
          <Hook0Icon name="money-check-dollar"></Hook0Icon>
          Your organization is on the <strong>Developer</strong> plan!
        </template>
      </Hook0CardHeader>

      <Hook0CardContent>
        <Hook0CardContentLines>
          <Hook0CardContentLine type="full-width">
            <template #content>
              <Hook0Text>You are currently limited to:</Hook0Text>
              <Hook0List>
                <Hook0ListItem>
                  <template #left>
                    <Hook0Icon name="users" class="mr-1"></Hook0Icon>
                    <Hook0Text>
                      <strong>{{ organization.quotas.members_per_organization_limit }}</strong>
                      member{{ organization.quotas.members_per_organization_limit > 1 ? 's' : '' }}
                    </Hook0Text>
                  </template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>
                    <Hook0Icon name="folder" class="mr-1"></Hook0Icon>
                    <Hook0Text>
                      <strong>{{ organization.quotas.applications_per_organization_limit }}</strong>
                      application{{
                        organization.quotas.applications_per_organization_limit > 1 ? 's' : ''
                      }}
                    </Hook0Text>
                  </template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>
                    <Hook0Icon name="file-lines" class="mr-1"></Hook0Icon>
                    <Hook0Text>
                      <strong>{{ organization.quotas.events_per_day_limit }}</strong>
                      event{{ organization.quotas.events_per_day_limit > 1 ? 's' : '' }} per day
                    </Hook0Text>
                  </template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>
                    <Hook0Icon name="database" class="mr-1"></Hook0Icon>
                    <Hook0Text>
                      <strong>{{ organization.quotas.days_of_events_retention_limit }}</strong>
                      day{{ organization.quotas.days_of_events_retention_limit > 1 ? 's' : '' }} of
                      event retention
                    </Hook0Text>
                  </template>
                </Hook0ListItem>
              </Hook0List>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContentLines>
      </Hook0CardContent>

      <Hook0CardFooter>
        <Hook0Button
          class="secondary"
          type="button"
          href="https://www.hook0.com/#pricing"
          target="_blank"
          >Available plans</Hook0Button
        >
        <Hook0Button class="primary" type="button" href="mailto:support@hook0.com"
          >Subscribe to a better plan
        </Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>

    <MembersList
      v-if="organization.quotas.members_per_organization_limit > 1"
      :burst="$route.params.organization_id"
    >
    </MembersList>

    <ApplicationsList :burst="$route.params.organization_id"> </ApplicationsList>
  </div>
</template>
