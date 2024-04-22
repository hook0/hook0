<script setup lang="ts">
import { useRoute } from 'vue-router';
import { onMounted, onUpdated, ref } from 'vue';

import Hook0Text from '@/components/Hook0Text.vue';
import { Problem, UUID } from '@/http';
import * as ApplicationService from './ApplicationService';
import { Application } from './ApplicationService';
import { Alert } from '@/components/Hook0Alert';
import { routes } from '@/routes';
import EventTypesList from '@/pages/organizations/applications/event_types/EventTypesList.vue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';
import SubscriptionsList from '@/pages/organizations/applications/subscriptions/SubscriptionsList.vue';
import LogList from '@/pages/organizations/applications/logs/LogList.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';

const route = useRoute();

const application_id = ref<UUID | null>(null);
const application = ref({
  name: '',
});
const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function _load() {
  if (application_id.value !== route.params.application_id) {
    application_id.value = route.params.application_id as UUID;

    ApplicationService.get(application_id.value)
      .then((app: Application) => {
        application.value.name = app.name;
      })
      .catch(displayError);
  }
}

function displayError(err: unknown) {
  console.error(err);
  alert.value.visible = true;

  const problem: Problem = err as Problem;
  alert.value.type = problem.status >= 500 ? 'alert' : 'warning';
  alert.value.title = problem.title;
  alert.value.description = problem.detail;
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
          <Hook0Icon name="rocket"></Hook0Icon>
          Application
          <Hook0Text class="bold">{{ application.name }}</Hook0Text>
          dashboard
        </template>
        <template #subtitle>
          here Hook0 will display metrics about webhooks & events usage
        </template>
        <template #actions>
          <Hook0Button
            :to="{
              name: routes.ApplicationsDetail,
              params: {
                organization_id: $route.params.organization_id,
                application_id: $route.params.application_id,
              },
            }"
          >
            Settings
          </Hook0Button>
        </template>
      </Hook0CardHeader>
    </Hook0Card>
    <EventTypesList :burst="$route.params.application_id"></EventTypesList>
    <EventsList :burst="$route.params.application_id"></EventsList>
    <SubscriptionsList :burst="$route.params.application_id"></SubscriptionsList>
    <LogList :burst="$route.params.application_id"></LogList>
  </div>
</template>
