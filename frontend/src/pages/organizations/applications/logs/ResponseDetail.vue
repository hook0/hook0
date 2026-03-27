<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';

import { useRouteIds } from '@/composables/useRouteIds';
import { useResponseDetail } from './useResponseQueries';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Uuid from '@/components/Hook0Uuid.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';

const SENSITIVE_HEADERS = new Set([
  'cookie',
  'set-cookie',
  'authorization',
  'www-authenticate',
  'proxy-authorization',
  'proxy-authenticate',
]);

const { t } = useI18n();
const route = useRoute();
const { responseId, applicationId } = useRouteIds();

const { data: response, isLoading, error, refetch } = useResponseDetail(responseId, applicationId);

const eventId = computed(() => {
  const q = route.query.event_id;
  return typeof q === 'string' ? q : '';
});

const filteredHeaders = computed(() => {
  if (!response.value?.headers) return null;
  const entries = Object.entries(response.value.headers).filter(
    ([key]) => !SENSITIVE_HEADERS.has(key.toLowerCase())
  );
  if (entries.length === 0) return null;
  return Object.fromEntries(entries);
});

function statusCodeClass(code: number | undefined | null): string {
  if (!code) return 'response-status--unknown';
  if (code >= 200 && code < 300) return 'response-status--success';
  if (code >= 300 && code < 400) return 'response-status--warning';
  return 'response-status--error';
}
</script>

<template>
  <Hook0PageLayout :title="t('responses.detail')">
    <!-- Loading skeleton -->
    <Hook0Card v-if="isLoading || !response">
      <Hook0CardHeader>
        <template #header>{{ t('responses.detail') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error state -->
    <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />

    <!-- Data loaded -->
    <template v-else-if="response">
      <Hook0Stack data-test="response-detail-page" direction="column" gap="md">
        <!-- Response Summary -->
        <Hook0Card data-test="response-detail-card">
          <Hook0CardHeader>
            <template #header>{{ t('responses.summary') }}</template>
            <template #actions>
              <Hook0Button
                v-if="eventId"
                variant="secondary"
                :to="{
                  name: routes.EventsDetail,
                  params: {
                    organization_id: route.params.organization_id,
                    application_id: route.params.application_id,
                    event_id: eventId,
                  },
                }"
                data-test="response-event-link"
              >
                {{ t('logs.eventId') }}
              </Hook0Button>
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label>{{ t('responses.id') }}</template>
              <template #content>
                <Hook0Uuid :value="response.response_id" />
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>{{ t('responses.httpStatusCode') }}</template>
              <template #content>
                <span
                  v-if="response.http_code"
                  class="response-status"
                  :class="statusCodeClass(response.http_code)"
                  data-test="response-status-code"
                >
                  {{ response.http_code }}
                </span>
                <span v-else class="response-detail__muted">&mdash;</span>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>{{ t('responses.elapsedTime') }}</template>
              <template #content>
                <span v-if="response.elapsed_time_ms != null" data-test="response-elapsed-time">
                  {{ t('responses.elapsedTimeMs', { ms: response.elapsed_time_ms }) }}
                </span>
                <span v-else class="response-detail__muted">&mdash;</span>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine v-if="response.response_error_name">
              <template #label>{{ t('responses.error') }}</template>
              <template #content>
                <Hook0Code inline :code="response.response_error_name" data-test="response-error" />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>

        <!-- Response Headers -->
        <Hook0Card data-test="response-headers-card">
          <Hook0CardHeader>
            <template #header>{{ t('responses.headers') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <template v-if="filteredHeaders">
              <Hook0CardContentLine v-for="(value, key) in filteredHeaders" :key="key">
                <template #label>{{ key }}</template>
                <template #content>
                  <Hook0Code inline :code="String(value)" />
                </template>
              </Hook0CardContentLine>
            </template>
            <template v-else>
              <Hook0CardContentLine>
                <template #label>{{ t('responses.noHeaders') }}</template>
              </Hook0CardContentLine>
            </template>
          </Hook0CardContent>
        </Hook0Card>

        <!-- Response Body -->
        <Hook0Card data-test="response-body-card">
          <Hook0CardHeader>
            <template #header>{{ t('responses.body') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <template v-if="response.body_formatted">
              <Hook0Code :code="response.body_formatted" />
            </template>
            <template v-else>
              <Hook0CardContentLine>
                <template #label>{{ t('responses.noBody') }}</template>
              </Hook0CardContentLine>
            </template>
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.response-status {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.8125rem;
  font-weight: 600;
  font-family: var(--font-mono);
  letter-spacing: 0.02em;
}

.response-status--success {
  background-color: var(--color-success-light);
  color: var(--color-success);
}

.response-status--warning {
  background-color: var(--color-warning-light);
  color: var(--color-warning);
}

.response-status--error {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.response-status--unknown {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-muted);
}

.response-detail__muted {
  color: var(--color-text-tertiary);
}
</style>
