<script setup lang="ts">
import { useRoute } from 'vue-router';
import { onMounted, onUpdated, ref } from 'vue';

import Hook0Text from '@/components/Hook0Text.vue';
import { Problem, UUID } from '@/http';
import * as ApplicationService from './ApplicationService';
import { Application } from './ApplicationService';
import { routes } from '@/routes';
import EventTypesList from '@/pages/organizations/applications/event_types/EventTypesList.vue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';
import SubscriptionsList from '@/pages/organizations/applications/subscriptions/SubscriptionsList.vue';
import LogList from '@/pages/organizations/applications/logs/LogList.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import { push } from 'notivue';
import Hook0TutorialWidget from '@/components/Hook0TutorialWidget.vue';
import { Step } from '@/pages/tutorial/TutorialService';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';

const route = useRoute();

enum OnboardingStepStatus {
  ToDo = 'ToDo',
  Done = 'Done',
}

const application_id = ref<UUID | null>(null);
const application = ref({
  name: '',
  organization_id: '',
  onboarding_steps: {
    event_type: OnboardingStepStatus.ToDo,
    subscription: OnboardingStepStatus.ToDo,
    event: OnboardingStepStatus.ToDo,
  },
});

const widgetItems = ref<Step[]>([]);

function _load() {
  application_id.value = route.params.application_id as UUID;

  ApplicationService.get(application_id.value)
    .then((app: Application) => {
      application.value.name = app.name;
      application.value.organization_id = app.organization_id;
      application.value.onboarding_steps.event_type =
        app.onboarding_steps.event_type === 'Done'
          ? OnboardingStepStatus.Done
          : OnboardingStepStatus.ToDo;
      application.value.onboarding_steps.subscription =
        app.onboarding_steps.subscription === 'Done'
          ? OnboardingStepStatus.Done
          : OnboardingStepStatus.ToDo;
      application.value.onboarding_steps.event =
        app.onboarding_steps.event === 'Done'
          ? OnboardingStepStatus.Done
          : OnboardingStepStatus.ToDo;
    })
    .then(() => {
      if (
        application.value.onboarding_steps.event_type === OnboardingStepStatus.Done &&
        application.value.onboarding_steps.subscription === OnboardingStepStatus.Done &&
        application.value.onboarding_steps.event === OnboardingStepStatus.Done
      ) {
        return;
      }

      widgetItems.value = [
        {
          title: 'Create an event type',
          details:
            'Event types are categories of events. For each subscription, you will then be able choose among your declared event types to receive only the right events.',
          isActive: application.value.onboarding_steps.event_type === OnboardingStepStatus.Done,
          icon: 'folder-tree',
          route: {
            name: routes.TutorialCreateEventType,
            params: {
              organization_id: application.value.organization_id,
              application_id: application_id.value,
            },
          },
        },
        {
          title: 'Create a subscription',
          details: 'You can create as many subscriptions as you need.',
          isActive: application.value.onboarding_steps.subscription === OnboardingStepStatus.Done,
          icon: 'link',
          route: {
            name: routes.TutorialCreateSubscription,
            params: {
              organization_id: application.value.organization_id,
              application_id: application_id.value,
            },
          },
        },
        {
          title: 'Send an event',
          details: 'You can send as many events as you need.',
          isActive: application.value.onboarding_steps.event === OnboardingStepStatus.Done,
          icon: 'file-lines',
          route: {
            name: routes.TutorialSendEvent,
            params: {
              organization_id: application.value.organization_id,
              application_id: application_id.value,
            },
          },
        },
      ];
    })
    .catch(displayError);
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
      <Hook0CardContent v-if="widgetItems.length > 0">
        <Hook0CardContentLines>
          <Hook0CardContentLine type="full-width">
            <template #content>
              <Hook0TutorialWidget :steps="widgetItems" />
            </template>
          </Hook0CardContentLine>
        </Hook0CardContentLines>
      </Hook0CardContent>
    </Hook0Card>
    <EventTypesList :burst="$route.params.application_id"></EventTypesList>
    <EventsList :burst="$route.params.application_id" @event-sended="_load()"></EventsList>
    <SubscriptionsList :burst="$route.params.application_id"></SubscriptionsList>
    <LogList :burst="$route.params.application_id"></LogList>
  </div>
</template>
