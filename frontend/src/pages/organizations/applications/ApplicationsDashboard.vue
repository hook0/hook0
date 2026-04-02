<script setup lang="ts">
import { computed, markRaw } from 'vue';
import { useI18n } from 'vue-i18n';
import { Rocket, FileText, Database } from 'lucide-vue-next';

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
import EventsPerDayChartCard from '@/components/EventsPerDayChartCard.vue';
import Hook0Consumption from '@/components/Hook0Consumption.vue';
import type { ConsumptionQuota } from '@/components/consumption.types';
import { useRouteIds } from '@/composables/useRouteIds';

const { t } = useI18n();
const { organizationId, applicationId } = useRouteIds();

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

// Consumptions computed from app detail
const consumptions = computed<ConsumptionQuota[]>(() => {
  if (!application.value) return [];
  return [
    {
      icon: markRaw(FileText),
      name: t('applications.consumptionEventsPerDay'),
      consumption: application.value.consumption.events_per_day || 0,
      quota: application.value.quotas.events_per_day_limit,
    },
    {
      icon: markRaw(Database),
      name: t('applications.consumptionRetention'),
      description: t('applications.consumptionRetentionDesc'),
      consumption: application.value.quotas.days_of_events_retention_limit,
      quota: application.value.quotas.days_of_events_retention_limit,
      displayValue: String(application.value.quotas.days_of_events_retention_limit),
      displayUnit: t('common.days', application.value.quotas.days_of_events_retention_limit ?? 0),
    },
  ];
});
</script>

<template>
  <Hook0PageLayout :title="t('applications.dashboard')" data-test="application-dashboard">
    <!-- Loading -->
    <Hook0CardSkeleton v-if="appLoading || (!application && !appError)" :lines="3" />

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
              <div class="app-dashboard__title-group">
                <span class="app-dashboard__label">{{ t('applications.titleSingular') }}</span>
                <span class="app-dashboard__name" data-test="application-dashboard-name">
                  {{ application.name }}
                </span>
              </div>
            </Hook0Stack>
          </template>
          <template #actions>
            <Hook0Button
              :to="{
                name: routes.ApplicationsDetail,
                params: {
                  organization_id: organizationId,
                  application_id: applicationId,
                },
              }"
              data-test="application-dashboard-settings-link"
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
      <EventsPerDayChartCard
        :title="t('applications.inboundEventsTitle', { name: application.name })"
        :entries="eventsPerDayData ?? []"
        :stacked="false"
        :from="eventsPerDayFrom"
        :to="eventsPerDayTo"
        :days="eventsPerDayDays"
        :quota-limit="application.quotas.events_per_day_limit"
        @update:days="eventsPerDayDays = $event"
        @refresh="refetchEventsPerDay()"
      />

      <!-- Usage / Quotas -->
      <Hook0Consumption
        :title="t('applications.consumptionTitle', { name: application.name })"
        entity-type="application"
        :consumptions="consumptions"
      />
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.app-dashboard__title-group {
  display: flex;
  align-items: baseline;
  gap: 0.375rem;
}

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
