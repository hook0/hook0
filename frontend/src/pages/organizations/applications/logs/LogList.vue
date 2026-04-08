<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';

import { Send } from 'lucide-vue-next';
import { DOCS_LOGS_URL, API_DOCS_LOGS_URL } from '@/constants/externalLinks';

import { useLogList } from './useLogQueries';
import { routes } from '@/routes';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';
import { usePermissions } from '@/composables/usePermissions';

import DeliverySplitView from './DeliverySplitView.vue';
import Hook0DocButtons from '@/components/Hook0DocButtons.vue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';

const { t } = useI18n();
const route = useRoute();
const { organizationId, applicationId } = useRouteIds();
const { data: requestAttempts, isLoading, error, refetch } = useLogList(applicationId);
const { data: organization } = useOrganizationDetail(organizationId);
const { canCreate } = usePermissions();

const retentionDays = computed(() => {
  const days = organization.value?.quotas.days_of_events_retention_limit;
  // API uses INT32_MAX (2147483647) as sentinel for "unlimited retention"
  if (!days || days >= 3650) return null;
  return days;
});
</script>

<template>
  <Hook0PageLayout :title="t('logs.title')">
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="void refetch()" />

    <Hook0Card v-else-if="isLoading || !requestAttempts" data-test="logs-card">
      <Hook0CardHeader>
        <template #header>{{ t('logs.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <template v-else>
      <Hook0Card data-test="logs-card">
        <Hook0CardHeader>
          <template #header>{{ t('logs.title') }}</template>
          <template #subtitle>
            {{ t('logs.subtitle') }}
            <Hook0HelpText v-if="retentionDays" tone="emphasis">{{
              t('logs.subtitleRetention', { days: retentionDays })
            }}</Hook0HelpText>
          </template>
          <template #actions>
            <Hook0DocButtons :doc-url="DOCS_LOGS_URL" :api-url="API_DOCS_LOGS_URL" />
          </template>
        </Hook0CardHeader>
      </Hook0Card>

      <DeliverySplitView
        v-if="requestAttempts.length > 0"
        :deliveries="requestAttempts"
        :application-id="applicationId"
      />

      <Hook0Card v-else>
        <Hook0CardContent>
          <Hook0EmptyState
            :title="t('logs.empty.title')"
            :description="t('logs.empty.description')"
            :icon="Send"
          >
            <template v-if="canCreate('subscription')" #action>
              <Hook0Button
                variant="primary"
                :to="{
                  name: routes.SubscriptionsNew,
                  params: {
                    organization_id: route.params.organization_id,
                    application_id: route.params.application_id,
                  },
                }"
              >
                {{ t('subscriptions.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* Align doc buttons to top when subtitle wraps */
:deep(.hook0-card-header__container) {
  align-items: flex-start;
}
</style>
