<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';
import { isAxiosError } from '@/http';

import { useRouteIds } from '@/composables/useRouteIds';
import { usePermissions } from '@/composables/usePermissions';
import { useRequestAttemptDetail } from './useLogQueries';
import LogDetailContent from './LogDetailContent.vue';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();
const route = useRoute();
const { requestAttemptId, applicationId } = useRouteIds();
usePermissions();

const {
  data: attempt,
  isLoading,
  error,
  refetch,
} = useRequestAttemptDetail(requestAttemptId, applicationId);

const is404 = computed(() => {
  if (!error.value) return false;
  return isAxiosError(error.value) && error.value.response?.status === 404;
});
</script>

<template>
  <Hook0PageLayout :title="t('logs.deliveryDetail')">
    <!-- Error first -->
    <template v-if="error && !isLoading">
      <Hook0ErrorCard :error="is404 ? t('logs.deliveryNotFound') : error" @retry="void refetch()" />
      <Hook0Button
        variant="secondary"
        :to="{
          name: routes.LogsList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        }"
      >
        {{ t('logs.backToList') }}
      </Hook0Button>
    </template>

    <!-- Loading (also handles disabled query state) -->
    <Hook0Card v-else-if="isLoading || !attempt">
      <Hook0CardHeader>
        <template #header>{{ t('logs.deliveryDetail') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded -->
    <template v-else>
      <Hook0Stack data-test="log-detail-page" direction="column" gap="md">
        <Hook0Card>
          <Hook0CardContent>
            <LogDetailContent :attempt="attempt" :application-id="applicationId" />
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>
