<script setup lang="ts">
import { computed, markRaw } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Component } from 'vue';
import { CreditCard, Users, FolderOpen, FileText, Database, Settings } from 'lucide-vue-next';

import { useRouteIds } from '@/composables/useRouteIds';
import { useOrganizationDetail } from './useOrganizationQueries';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { useEventsPerDay } from '@/pages/organizations/applications/useEventsPerDayQuery';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardSkeleton from '@/components/Hook0CardSkeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import EventsPerDayChartCard from '@/components/EventsPerDayChartCard.vue';
import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';

const { t } = useI18n();
const { organizationId } = useRouteIds();

const {
  data: organization,
  isLoading: orgLoading,
  isFetched: orgFetched,
  error: orgError,
  refetch: refetchOrg,
} = useOrganizationDetail(organizationId);

const { data: instanceConfig } = useInstanceConfig();

const pricingEnabled = computed(() => instanceConfig.value?.quota_enforcement ?? false);
const supportEmailAddress = computed(() => instanceConfig.value?.support_email_address ?? '');

// Events per day chart
const {
  days: eventsPerDayDays,
  from: eventsPerDayFrom,
  to: eventsPerDayTo,
  data: eventsPerDayData,
  refetch: refetchEventsPerDay,
} = useEventsPerDay('organization', organizationId);

/** First two letters of the organization name, uppercased. */
const orgInitials = computed(() => {
  if (!organization.value) return '';
  return organization.value.name.slice(0, 2).toUpperCase();
});

/** Summary subtitle: "X members · Y applications · Z events/day" (pluralized via i18n) */
const orgSubtitle = computed(() => {
  if (!organization.value) return '';
  const memberCount = organization.value.users?.length ?? 0;
  const appCount = organization.value.consumption?.applications ?? 0;
  const evtCount = organization.value.consumption?.events_per_day ?? 0;
  return [
    t('organizations.summaryMembers', { count: memberCount }, memberCount),
    t('organizations.summaryApplications', { count: appCount }, appCount),
    t('organizations.summaryEventsPerDay', { count: evtCount }, evtCount),
  ].join(' \u00B7 ');
});

/** Quota cards shown in the developer-plan notice section. */
const quotaCards = computed<{ icon: Component; value: number | undefined; label: string }[]>(() => {
  if (!organization.value) return [];
  const q = organization.value.quotas;
  return [
    {
      icon: markRaw(Users),
      value: q.members_per_organization_limit,
      label: t('organizations.consumptionMembers'),
    },
    {
      icon: markRaw(FolderOpen),
      value: q.applications_per_organization_limit,
      label: t('organizations.consumptionApplications'),
    },
    {
      icon: markRaw(FileText),
      value: q.events_per_day_limit,
      label: t('organizations.consumptionEventsPerDay'),
    },
    {
      icon: markRaw(Database),
      value: q.days_of_events_retention_limit,
      label: t('organizations.consumptionRetention'),
    },
  ];
});
</script>

<template>
  <Hook0PageLayout :title="t('organizations.dashboard')" data-test="org-dashboard-page">
    <!-- Loading (also shown when query is disabled and data is undefined) -->
    <Hook0CardSkeleton
      v-if="orgLoading || (!organization && !orgError && !orgFetched)"
      :lines="4"
    />

    <!-- Error -->
    <Hook0ErrorCard v-else-if="orgError" :error="orgError" @retry="refetchOrg()" />

    <!-- Not found (fetched but no data and no error) -->
    <Hook0ErrorCard
      v-else-if="!organization"
      :error="new Error(t('organizations.notFound'))"
      @retry="refetchOrg()"
    />

    <!-- Data loaded -->
    <template v-else-if="organization">
      <!-- Organization header card: Stacked Compact with Avatar -->
      <Hook0Card data-test="organization-dashboard-card">
        <div class="org-header">
          <div class="org-header__avatar">{{ orgInitials }}</div>
          <div class="org-header__info">
            <div class="org-header__title-row">
              <span class="org-header__name">{{ organization.name }}</span>
              <template v-if="pricingEnabled">
                <Hook0Badge
                  v-if="organization.plan"
                  variant="primary"
                  size="sm"
                  :title="`${t('organizations.plan')}: ${organization.plan.label}`"
                >
                  {{ organization.plan.label }}
                </Hook0Badge>
                <Hook0Badge
                  v-else
                  variant="default"
                  size="sm"
                  :title="`${t('organizations.plan')}: ${t('organizations.planDeveloper')}`"
                >
                  {{ t('organizations.planDeveloper') }}
                </Hook0Badge>
              </template>
            </div>
            <span class="org-header__subtitle">{{ orgSubtitle }}</span>
          </div>
          <div class="org-header__actions">
            <Hook0Button
              :to="{
                name: routes.OrganizationsDetail,
                params: { organization_id: organizationId },
              }"
            >
              <Settings :size="14" aria-hidden="true" />
              {{ t('common.settings') }}
            </Hook0Button>
          </div>
        </div>
      </Hook0Card>

      <!-- Applications list (moved up, before chart) -->
      <ApplicationsList :burst="organizationId" />

      <!-- Events per day chart -->
      <EventsPerDayChartCard
        :title="t('organizations.consumptionTitle', { name: organization.name })"
        :entries="eventsPerDayData ?? []"
        :stacked="true"
        :from="eventsPerDayFrom"
        :to="eventsPerDayTo"
        :days="eventsPerDayDays"
        :quota-limit="organization.quotas.events_per_day_limit"
        @update:days="eventsPerDayDays = $event"
        @refresh="refetchEventsPerDay()"
      />

      <!-- Developer plan notice (shown only when on free plan) -->
      <Hook0Card v-if="pricingEnabled && !organization.plan">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="warning" size="sm">
                <CreditCard :size="14" aria-hidden="true" />
              </Hook0IconBadge>
              <span class="org-dashboard__label">{{ t('organizations.developerPlanNotice') }}</span>
            </Hook0Stack>
          </template>
        </Hook0CardHeader>

        <Hook0CardContent>
          <Hook0Stack direction="column" gap="md">
            <span class="org-dashboard__label">{{ t('organizations.currentlyLimitedTo') }}</span>
            <Hook0Stack layout="grid" grid-size="compact" gap="sm">
              <Hook0Card
                v-for="card in quotaCards"
                :key="card.label"
                class="org-dashboard__quota-card"
              >
                <Hook0CardContent>
                  <Hook0Stack direction="row" align="center" gap="sm">
                    <Hook0IconBadge variant="primary" size="md">
                      <component :is="card.icon" :size="18" aria-hidden="true" />
                    </Hook0IconBadge>
                    <Hook0Stack direction="column" gap="none">
                      <span class="org-dashboard__quota-value">{{ card.value }}</span>
                      <span class="org-dashboard__quota-label">{{ card.label.toLowerCase() }}</span>
                    </Hook0Stack>
                  </Hook0Stack>
                </Hook0CardContent>
              </Hook0Card>
            </Hook0Stack>
          </Hook0Stack>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button type="button" href="https://www.hook0.com/#pricing" target="_blank">{{
            t('organizations.availablePlans')
          }}</Hook0Button>
          <Hook0Button
            v-if="supportEmailAddress"
            variant="primary"
            type="button"
            :href="`mailto:${supportEmailAddress}`"
            >{{ t('organizations.subscribeBetterPlan') }}</Hook0Button
          >
        </Hook0CardFooter>
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* ---- Organization header card ---- */
.org-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem;
}

.org-header__avatar {
  width: 2.75rem;
  height: 2.75rem;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-primary), #22c55e);
  color: #ffffff;
  font-size: 0.875rem;
  font-weight: 700;
  flex-shrink: 0;
  line-height: 1;
}

.org-header__info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex: 1;
  min-width: 0;
}

.org-header__title-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.org-header__name {
  font-size: 1.0625rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.3;
}

.org-header__subtitle {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.org-header__actions {
  flex-shrink: 0;
}

@media (max-width: 767px) {
  .org-header {
    flex-wrap: wrap;
  }

  .org-header__actions {
    width: 100%;
  }
}

/* ---- Developer plan notice section ---- */
.org-dashboard__label {
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.5;
}

.org-dashboard__quota-value {
  color: var(--color-text-primary);
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.2;
}

.org-dashboard__quota-label {
  color: var(--color-text-tertiary);
  font-size: 0.8125rem;
  font-weight: 400;
  line-height: 1.2;
}

.org-dashboard__quota-card :deep(.hook0-card-content) {
  padding: 0.75rem 1rem;
}
</style>
