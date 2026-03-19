<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Rocket, FileText } from 'lucide-vue-next';

import { useApplicationDetail } from './useApplicationQueries';
import { useEventsPerDay } from './useEventsPerDayQuery';
import { applicationSteps } from '@/pages/tutorial/TutorialService';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardSkeleton from '@/components/Hook0CardSkeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0TutorialWidget from '@/components/Hook0TutorialWidget.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0EventsPerDayChart from '@/components/Hook0EventsPerDayChart.vue';

import EventTypesList from '@/pages/organizations/applications/event_types/EventTypesList.vue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';
import SubscriptionsList from '@/pages/organizations/applications/subscriptions/SubscriptionsList.vue';
import LogList from '@/pages/organizations/applications/logs/LogList.vue';

const { t } = useI18n();
const route = useRoute();

const applicationId = computed(() => route.params.application_id as string);

const {
  data: application,
  isLoading: appLoading,
  error: appError,
  refetch: refetchApp,
} = useApplicationDetail(applicationId);

const widgetItems = computed(() => {
  if (!application.value) return [];
  return applicationSteps(application.value);
});

// Events per day chart
const {
  days: eventsPerDayDays,
  from: eventsPerDayFrom,
  to: eventsPerDayTo,
  data: eventsPerDayData,
  refetch: refetchEventsPerDay,
} = useEventsPerDay('application', applicationId);
</script>

<template>
  <Hook0PageLayout :title="t('applications.dashboard')" data-test="app-dashboard-page">
    <!-- Loading -->
    <Hook0CardSkeleton v-if="appLoading" :lines="3" />

    <!-- Error -->
    <Hook0ErrorCard v-else-if="appError" :error="appError" @retry="refetchApp()" />

    <!-- Data loaded -->
    <template v-else-if="application">
      <Hook0Card data-test="application-dashboard-card">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" gap="sm" align="center">
              <Hook0IconBadge variant="primary">
                <Rocket :size="18" aria-hidden="true" />
              </Hook0IconBadge>
              <span class="app-dashboard__label">{{ t('applications.titleSingular') }}</span>
              <span class="app-dashboard__name">{{ application.name }}</span>
            </Hook0Stack>
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
              {{ t('common.settings') }}
            </Hook0Button>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent v-if="widgetItems.length > 0" data-test="app-dashboard-tutorial-widget">
          <Hook0TutorialWidget :steps="widgetItems" />
        </Hook0CardContent>
      </Hook0Card>

      <!-- Events per day chart -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="primary" size="sm">
                <FileText :size="14" aria-hidden="true" />
              </Hook0IconBadge>
              <span class="app-dashboard__label">{{
                t('applications.consumptionTitle', { name: application.name })
              }}</span>
            </Hook0Stack>
          </template>
          <template #actions>
            <Hook0Button @click="refetchEventsPerDay()">
              {{ t('common.refresh') }}
            </Hook0Button>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0EventsPerDayChart
            :entries="eventsPerDayData ?? []"
            :stacked="false"
            :from="eventsPerDayFrom"
            :to="eventsPerDayTo"
            :days="eventsPerDayDays"
            :quota-limit="application.quotas.events_per_day_limit"
            @update:days="eventsPerDayDays = $event"
          />
        </Hook0CardContent>
      </Hook0Card>

      <EventTypesList :burst="$route.params.application_id" />
      <EventsList :burst="$route.params.application_id" @event-sent="refetchApp()" />
      <SubscriptionsList :burst="$route.params.application_id" />
      <LogList :burst="$route.params.application_id" />
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.app-dashboard__label {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  line-height: 1.5;
}

.app-dashboard__name {
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
  line-height: 1.5;
}
</style>
