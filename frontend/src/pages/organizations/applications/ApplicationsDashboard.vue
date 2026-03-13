<script setup lang="ts">
import { useRoute } from 'vue-router';
import { onMounted, onUpdated, ref, watch } from 'vue';
import { format, subDays } from 'date-fns';

import Hook0Text from '@/components/Hook0Text.vue';
import { Problem, UUID } from '@/http';
import * as ApplicationService from './ApplicationService';
import { ApplicationInfo } from './ApplicationService';
import * as EventsPerDayService from './EventsPerDayService';
import { EventsPerDayEntry } from './EventsPerDayService';
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
import { applicationSteps, Step } from '@/pages/tutorial/TutorialService';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0EventsPerDayChart from '@/components/Hook0EventsPerDayChart.vue';

const route = useRoute();

const application_id = ref<UUID | null>(null);
const application = ref({
  name: '',
  consumption: {
    events_per_day: 0,
  },
  quotas: {
    events_per_day_limit: 0,
  },
});

const widgetItems = ref<Step[]>([]);

const eventsPerDayDays = ref(30);
const eventsPerDayData = ref<EventsPerDayEntry[]>([]);
const eventsPerDayFrom = ref(format(subDays(new Date(), 30), 'yyyy-MM-dd'));
const eventsPerDayTo = ref(format(new Date(), 'yyyy-MM-dd'));

function loadEventsPerDay() {
  if (!application_id.value) return;
  EventsPerDayService.application(
    application_id.value,
    eventsPerDayFrom.value,
    eventsPerDayTo.value
  )
    .then((data: EventsPerDayEntry[]) => {
      eventsPerDayData.value = data;
    })
    .catch(displayError);
}

watch(eventsPerDayDays, (days) => {
  eventsPerDayFrom.value = format(subDays(new Date(), days), 'yyyy-MM-dd');
  eventsPerDayTo.value = format(new Date(), 'yyyy-MM-dd');
  loadEventsPerDay();
});

function _load() {
  application_id.value = route.params.application_id as UUID;

  ApplicationService.get(application_id.value)
    .then((app: ApplicationInfo) => {
      application.value.name = app.name;
      application.value.consumption.events_per_day = app.consumption.events_per_day || 0;
      application.value.quotas.events_per_day_limit = app.quotas.events_per_day_limit;
      widgetItems.value = applicationSteps(app);
    })
    .catch(displayError);

  loadEventsPerDay();
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
    <Hook0Card>
      <Hook0CardHeader>
        <template #header>
          <Hook0Icon name="chart-bar"></Hook0Icon>
          Application Consumption
        </template>
        <template #actions>
          <Hook0Button class="secondary" @click="_load()">
            <Hook0Icon name="arrows-rotate" class="mr-1"></Hook0Icon>
            Refresh
          </Hook0Button>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLines>
          <Hook0CardContentLine type="full-width">
            <template #content>
              <Hook0EventsPerDayChart
                :entries="eventsPerDayData"
                :stacked="false"
                :from="eventsPerDayFrom"
                :to="eventsPerDayTo"
                :days="eventsPerDayDays"
                :quota-limit="application.quotas.events_per_day_limit"
                @update:days="eventsPerDayDays = $event"
              />
            </template>
          </Hook0CardContentLine>
        </Hook0CardContentLines>
      </Hook0CardContent>
    </Hook0Card>
    <EventTypesList :burst="$route.params.application_id"></EventTypesList>
    <EventsList :burst="$route.params.application_id" @event-sent="_load()"></EventsList>
    <SubscriptionsList :burst="$route.params.application_id"></SubscriptionsList>
    <LogList :burst="$route.params.application_id"></LogList>
  </div>
</template>
