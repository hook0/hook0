<script setup lang="ts">
import { computed, markRaw, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import type { Component } from 'vue';
import type { RouteLocationRaw } from 'vue-router';
import { CreditCard, Users, FolderOpen, FileText, Database, Settings, Box } from 'lucide-vue-next';

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
import Hook0Consumption from '@/components/Hook0Consumption.vue';
import type { ConsumptionQuota } from '@/components/consumption.types';
import Hook0Avatar from '@/components/Hook0Avatar.vue';
import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';

const { t } = useI18n();
const router = useRouter();
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

// Scroll targets for dashboard card links
const appsSectionRef = ref<HTMLElement>();
const chartSectionRef = ref<HTMLElement>();

function scrollToApps() {
  appsSectionRef.value?.scrollIntoView({ behavior: 'smooth' });
}
function scrollToChart() {
  chartSectionRef.value?.scrollIntoView({ behavior: 'smooth' });
}

// Consumptions computed from org detail
const consumptions = computed<ConsumptionQuota[]>(() => {
  if (!organization.value) return [];
  return [
    {
      icon: markRaw(Users),
      name: t('organizations.consumptionMembers'),
      description: t('organizations.consumptionMembersDesc'),
      consumption: organization.value.consumption.members || 0,
      quota: organization.value.quotas.members_per_organization_limit,
      to: { name: routes.OrganizationsTeam, params: { organization_id: organizationId.value } },
    },
    {
      icon: markRaw(Box),
      name: t('organizations.consumptionApplications'),
      description: t('organizations.consumptionApplicationsDesc'),
      consumption: organization.value.consumption.applications || 0,
      quota: organization.value.quotas.applications_per_organization_limit,
      onClick: scrollToApps,
    },
    {
      icon: markRaw(FileText),
      name: t('organizations.consumptionEventsPerDay'),
      description: t('organizations.consumptionEventsPerDayDesc'),
      consumption: organization.value.consumption.events_per_day || 0,
      quota: organization.value.quotas.events_per_day_limit,
      onClick: scrollToChart,
    },
    {
      icon: markRaw(Database),
      name: t('organizations.consumptionRetention'),
      description: t('organizations.consumptionRetentionDesc'),
      consumption: organization.value.quotas.days_of_events_retention_limit,
      quota: organization.value.quotas.days_of_events_retention_limit,
      displayValue: String(organization.value.quotas.days_of_events_retention_limit),
      displayUnit: 'days',
      onClick: scrollToChart,
    },
  ];
});

/** Quota cards shown in the developer-plan notice section. */
interface QuotaCard {
  icon: Component;
  value: number | undefined;
  label: string;
  to?: RouteLocationRaw;
  onClick?: () => void;
}

const quotaCards = computed<QuotaCard[]>(() => {
  if (!organization.value) return [];
  const q = organization.value.quotas;
  return [
    {
      icon: markRaw(Users),
      value: q.members_per_organization_limit,
      label: t('organizations.consumptionMembers'),
      to: { name: routes.OrganizationsTeam, params: { organization_id: organizationId.value } },
    },
    {
      icon: markRaw(FolderOpen),
      value: q.applications_per_organization_limit,
      label: t('organizations.consumptionApplications'),
      onClick: scrollToApps,
    },
    {
      icon: markRaw(FileText),
      value: q.events_per_day_limit,
      label: t('organizations.consumptionEventsPerDay'),
      onClick: scrollToChart,
    },
    {
      icon: markRaw(Database),
      value: q.days_of_events_retention_limit,
      label: t('organizations.consumptionRetention'),
      onClick: scrollToChart,
    },
  ];
});

function onQuotaCardActivate(card: QuotaCard) {
  if (card.to) {
    void router.push(card.to);
  } else if (card.onClick) {
    card.onClick();
  }
}

function onQuotaCardKeydown(event: KeyboardEvent, card: QuotaCard) {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    onQuotaCardActivate(card);
  }
}
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
          <Hook0Avatar
            :name="organization.name"
            size="lg"
            variant="square"
            gradient="linear-gradient(135deg, var(--color-primary), var(--color-primary-gradient-end))"
          />
          <div class="org-header__info">
            <div class="org-header__title-row">
              <span class="org-header__name" :title="organization.name">{{
                organization.name
              }}</span>
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
      <div ref="appsSectionRef">
        <ApplicationsList :burst="organizationId" />
      </div>

      <!-- Events per day chart -->
      <div ref="chartSectionRef">
        <EventsPerDayChartCard
          :title="t('organizations.inboundEventsTitle', { name: organization.name })"
          :entries="eventsPerDayData ?? []"
          :stacked="true"
          :from="eventsPerDayFrom"
          :to="eventsPerDayTo"
          :days="eventsPerDayDays"
          :quota-limit="organization.quotas.events_per_day_limit"
          @update:days="eventsPerDayDays = $event"
          @refresh="refetchEventsPerDay()"
        />
      </div>

      <!-- Usage / Quotas -->
      <Hook0Consumption
        :title="t('organizations.consumptionTitle', { name: organization.name })"
        entity-type="organization"
        :consumptions="consumptions"
      />

      <!-- Developer plan notice (shown only when on free plan) -->
      <Hook0Card v-if="pricingEnabled && !organization.plan">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="warning" size="md">
                <CreditCard :size="18" aria-hidden="true" />
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
                class="org-dashboard__quota-card org-dashboard__quota-card--clickable"
                tabindex="0"
                role="button"
                @click="onQuotaCardActivate(card)"
                @keydown="onQuotaCardKeydown($event, card)"
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
  min-width: 0;
}

.org-header__name {
  font-size: 1.0625rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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

  .org-header__actions :deep(.hook0-button) {
    width: 100%;
    justify-content: center;
  }
}

/* ---- Developer plan notice section ---- */
.org-dashboard__label {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
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

.org-dashboard__quota-card--clickable {
  cursor: pointer;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.org-dashboard__quota-card--clickable:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-sm);
}

.org-dashboard__quota-card--clickable:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}
</style>
