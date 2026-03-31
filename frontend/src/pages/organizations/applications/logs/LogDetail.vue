<script setup lang="ts">
import { useI18n } from 'vue-i18n';

import { useRouteIds } from '@/composables/useRouteIds';
import { useRequestAttemptDetail } from './useLogQueries';
import LogDetailContent from './LogDetailContent.vue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();
const { requestAttemptId, applicationId } = useRouteIds();

const {
  data: attempt,
  isLoading,
  error,
  refetch,
} = useRequestAttemptDetail(requestAttemptId, applicationId);
</script>

<template>
  <Hook0PageLayout :title="t('logs.deliveryDetail')">
    <!-- Loading (also handles disabled query state) -->
    <Hook0Card v-if="isLoading || !attempt">
      <Hook0CardHeader>
        <template #header>{{ t('logs.deliveryDetail') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error -->
    <Hook0ErrorCard v-else-if="error" :error="error" @retry="void refetch()" />

    <!-- Data loaded -->
    <template v-else>
      <Hook0Stack data-test="log-detail-page" direction="column" gap="md">
        <Hook0Card>
          <Hook0CardContent>
            <LogDetailContent
              :attempt="attempt"
              :application-id="applicationId"
            />
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>
