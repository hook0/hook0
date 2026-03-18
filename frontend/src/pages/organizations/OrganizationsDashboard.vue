<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Building2, CreditCard, Users, FolderOpen, FileText, Database } from 'lucide-vue-next';

import { useOrganizationDetail } from './useOrganizationQueries';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { organizationSteps } from '@/pages/tutorial/TutorialService';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardSkeleton from '@/components/Hook0CardSkeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0TutorialWidget from '@/components/Hook0TutorialWidget.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';

const { t } = useI18n();
const route = useRoute();

const organizationId = computed(() => route.params.organization_id as string);

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

const widgetItems = computed(() => {
  if (!organization.value) return [];
  return organizationSteps(organization.value);
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
      <Hook0Card data-test="organization-dashboard-card">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm" class="org-dashboard__header-stack">
              <Hook0IconBadge variant="primary" size="sm">
                <Building2 :size="14" aria-hidden="true" />
              </Hook0IconBadge>
              <span class="org-dashboard__label">{{ t('organizations.title') }}</span>
              <span class="org-dashboard__name">{{ organization.name }}</span>
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
            </Hook0Stack>
          </template>
          <template #actions>
            <Hook0Button
              :to="{
                name: routes.OrganizationsDetail,
                params: { organization_id: $route.params.organization_id },
              }"
            >
              {{ t('common.settings') }}
            </Hook0Button>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent v-if="widgetItems.length > 0">
          <Hook0TutorialWidget :steps="widgetItems" />
        </Hook0CardContent>
      </Hook0Card>

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
              <Hook0Card class="org-dashboard__quota-card">
                <Hook0CardContent>
                  <Hook0Stack direction="row" align="center" gap="sm">
                    <Hook0IconBadge variant="primary" size="sm">
                      <Users :size="14" aria-hidden="true" />
                    </Hook0IconBadge>
                    <Hook0Stack direction="column" gap="none">
                      <span class="org-dashboard__quota-value">{{
                        organization.quotas.members_per_organization_limit
                      }}</span>
                      <span class="org-dashboard__quota-label">{{
                        t('organizations.consumptionMembers').toLowerCase()
                      }}</span>
                    </Hook0Stack>
                  </Hook0Stack>
                </Hook0CardContent>
              </Hook0Card>
              <Hook0Card class="org-dashboard__quota-card">
                <Hook0CardContent>
                  <Hook0Stack direction="row" align="center" gap="sm">
                    <Hook0IconBadge variant="primary" size="sm">
                      <FolderOpen :size="14" aria-hidden="true" />
                    </Hook0IconBadge>
                    <Hook0Stack direction="column" gap="none">
                      <span class="org-dashboard__quota-value">{{
                        organization.quotas.applications_per_organization_limit
                      }}</span>
                      <span class="org-dashboard__quota-label">{{
                        t('organizations.consumptionApplications').toLowerCase()
                      }}</span>
                    </Hook0Stack>
                  </Hook0Stack>
                </Hook0CardContent>
              </Hook0Card>
              <Hook0Card class="org-dashboard__quota-card">
                <Hook0CardContent>
                  <Hook0Stack direction="row" align="center" gap="sm">
                    <Hook0IconBadge variant="primary" size="sm">
                      <FileText :size="14" aria-hidden="true" />
                    </Hook0IconBadge>
                    <Hook0Stack direction="column" gap="none">
                      <span class="org-dashboard__quota-value">{{
                        organization.quotas.events_per_day_limit
                      }}</span>
                      <span class="org-dashboard__quota-label">{{
                        t('organizations.consumptionEventsPerDay').toLowerCase()
                      }}</span>
                    </Hook0Stack>
                  </Hook0Stack>
                </Hook0CardContent>
              </Hook0Card>
              <Hook0Card class="org-dashboard__quota-card">
                <Hook0CardContent>
                  <Hook0Stack direction="row" align="center" gap="sm">
                    <Hook0IconBadge variant="primary" size="sm">
                      <Database :size="14" aria-hidden="true" />
                    </Hook0IconBadge>
                    <Hook0Stack direction="column" gap="none">
                      <span class="org-dashboard__quota-value">{{
                        organization.quotas.days_of_events_retention_limit
                      }}</span>
                      <span class="org-dashboard__quota-label">{{
                        t('organizations.consumptionRetention').toLowerCase()
                      }}</span>
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

      <ApplicationsList :burst="$route.params.organization_id" />
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.org-dashboard__label {
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.5;
}

.org-dashboard__name {
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
  line-height: 1.5;
}

/* Wrap org header on mobile: label + name on separate lines */
@media (max-width: 767px) {
  .org-dashboard__header-stack {
    flex-wrap: wrap;
  }
}

.org-dashboard__quota-value {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 700;
  line-height: 1.5;
}

.org-dashboard__quota-label {
  color: var(--color-text-tertiary);
  font-size: 0.8125rem;
  font-weight: 400;
  line-height: 1.5;
}

.org-dashboard__quota-card :deep(.hook0-card-content) {
  padding: 0.75rem 1rem;
}
</style>
