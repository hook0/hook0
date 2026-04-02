<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';

import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Section from '@/components/Hook0Section.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import LogLifecycle from './LogLifecycle.vue';

import { useEventDetail } from '@/pages/organizations/applications/events/useEventQueries';
import { useSubscriptionDetail } from '@/pages/organizations/applications/subscriptions/useSubscriptionQueries';
import { useResponseDetail } from './useResponseQueries';
import { filterSensitiveHeaders } from './responseHeaders';
import { statusCodeClass } from './responseStatus';
import { routes } from '@/routes';

import type { RequestAttemptExtended } from './LogService';
import { RequestAttemptStatusType } from './LogService';

type Props = {
  attempt: RequestAttemptExtended;
  applicationId: string;
};

const props = defineProps<Props>();
const { t } = useI18n();
const route = useRoute();

// "Sent" = HTTP request dispatched; Waiting (retry queue) and Pending (not yet picked) have not made a network call
const isSent = computed(() => {
  const type = props.attempt.status.type;
  return type !== RequestAttemptStatusType.Waiting && type !== RequestAttemptStatusType.Pending;
});

const eventIdRef = computed(() => props.attempt.event_id);
const applicationIdRef = toRef(props, 'applicationId');
const responseIdRef = computed(() => props.attempt.response_id ?? '');

const {
  data: eventData,
  isLoading: eventLoading,
  error: eventError,
  refetch: eventRefetch,
} = useEventDetail(eventIdRef, applicationIdRef);
const subscriptionIdRef = computed(() => props.attempt.subscription.subscription_id);
const { data: subscriptionData } = useSubscriptionDetail(subscriptionIdRef);

const {
  data: responseData,
  isLoading: responseLoading,
  error: responseError,
  refetch: responseRefetch,
} = useResponseDetail(responseIdRef, applicationIdRef);

const filteredHeaders = computed(() => {
  if (!responseData.value) return null;
  return filterSensitiveHeaders(responseData.value.headers);
});

const hasHiddenHeaders = computed(() => {
  if (!responseData.value?.headers) return false;
  const original = Object.keys(responseData.value.headers).length;
  const filtered = filteredHeaders.value ? Object.keys(filteredHeaders.value).length : 0;
  return filtered < original;
});

const isErrorResponse = computed(() => {
  const code = responseData.value?.http_code ?? props.attempt.http_response_status;
  return code != null && code >= 400;
});
</script>

<template>
  <Hook0Section v-if="eventError && !eventLoading">
    <Hook0ErrorCard :error="eventError" @retry="void eventRefetch()" />
  </Hook0Section>

  <template v-else-if="eventLoading || !eventData">
    <!-- Metadata skeleton -->
    <Hook0Section>
      <div class="log-detail__skeleton-meta">
        <div v-for="i in 2" :key="i" class="log-detail__skeleton-meta-row">
          <Hook0Skeleton size="text" style="width: 5rem" />
          <Hook0Skeleton size="text-truncated" style="width: 10rem" />
        </div>
      </div>
    </Hook0Section>
    <!-- Lifecycle skeleton -->
    <Hook0Section>
      <Hook0Skeleton size="text" style="width: 5rem; margin-bottom: 0.75rem" />
      <div v-for="i in 5" :key="i" style="display: flex; gap: 0.625rem; margin-bottom: 0.75rem">
        <Hook0Skeleton size="text" style="width: 1rem; height: 1rem; border-radius: 50%" />
        <div style="flex: 1">
          <Hook0Skeleton size="text" style="width: 6rem" />
          <Hook0Skeleton size="text" style="width: 12rem; margin-top: 0.25rem" />
        </div>
      </div>
    </Hook0Section>
    <!-- Request/Response skeleton -->
    <Hook0Section>
      <Hook0Skeleton size="text" style="width: 4rem; margin-bottom: 0.75rem" />
      <Hook0Skeleton size="block" />
    </Hook0Section>
  </template>

  <!-- Event loaded -->
  <template v-else>
    <!-- Metadata (no title) -->
    <Hook0Section>
      <Hook0CardContentLine type="split">
        <template #label>{{ t('logs.event') }}</template>
        <template #content>
          <Hook0Button
            variant="link"
            size="sm"
            :to="{
              name: routes.EventsDetail,
              params: { ...route.params, event_id: eventData.event_id },
            }"
            class="log-detail__meta-link log-detail__meta-link--mono"
          >
            {{ eventData.event_type_name }}
          </Hook0Button>
        </template>
      </Hook0CardContentLine>
      <Hook0CardContentLine type="split">
        <template #label>{{ t('logs.subscription') }}</template>
        <template #content>
          <Hook0Button
            variant="link"
            size="sm"
            :to="{
              name: routes.SubscriptionsDetail,
              params: {
                organization_id: route.params.organization_id,
                application_id: route.params.application_id,
                subscription_id: attempt.subscription.subscription_id,
              },
            }"
            class="log-detail__meta-link"
          >
            {{ attempt.subscription.description ?? attempt.subscription.subscription_id }}
          </Hook0Button>
        </template>
      </Hook0CardContentLine>
    </Hook0Section>

    <!-- Lifecycle -->
    <Hook0Section :title="t('logs.lifecycle.title')">
      <LogLifecycle :event="eventData" :attempt="attempt" />
    </Hook0Section>

    <!-- Labels -->
    <Hook0Section
      v-if="eventData.labels && Object.keys(eventData.labels as Record<string, string>).length > 0"
      :title="t('events.labels')"
    >
      <div class="log-detail__labels">
        <span
          v-for="(val, key) in eventData.labels as Record<string, string>"
          :key="String(key)"
          class="log-detail__label-badge"
        >
          {{ key }}={{ val }}
        </span>
      </div>
    </Hook0Section>

    <!-- Request -->
    <Hook0Section :title="t('logs.request')" :separator="true">
      <div class="log-detail__grid">
        <template v-if="subscriptionData?.target">
          <span class="log-detail__grid-label">{{ t('logs.httpMethod') }}</span>
          <code class="log-detail__grid-value log-detail__grid-value--mono">{{
            subscriptionData.target.method.toUpperCase()
          }}</code>

          <span class="log-detail__grid-label">{{ t('logs.targetUrl') }}</span>
          <code class="log-detail__grid-value log-detail__grid-value--mono">{{
            subscriptionData.target.url
          }}</code>
        </template>
      </div>

      <!-- Payload -->
      <h4 class="log-detail__subsection-title">{{ t('events.payload') }}</h4>
      <Hook0Code :code="eventData.payload_decoded" language="json" :editable="false" />
    </Hook0Section>

    <!-- Response: not sent yet -->
    <Hook0Section v-if="!isSent" :title="t('logs.response')" :separator="true">
      <p class="log-detail__no-response">{{ t('responses.notSentYet') }}</p>
    </Hook0Section>

    <!-- Response: sent but no response (timeout) -->
    <Hook0Section v-else-if="!attempt.response_id" :title="t('logs.response')" :separator="true">
      <p class="log-detail__no-response">{{ t('responses.noResponse') }}</p>
    </Hook0Section>

    <!-- Response: loading -->
    <template v-else-if="responseLoading || (!responseData && !responseError)">
      <Hook0Section>
        <Hook0Skeleton size="text" />
        <Hook0Skeleton size="text-truncated" />
      </Hook0Section>
      <Hook0Section>
        <Hook0Skeleton size="block" />
      </Hook0Section>
    </template>

    <!-- Response: error -->
    <Hook0Section v-else-if="responseError">
      <Hook0ErrorCard :error="responseError" @retry="void responseRefetch()" />
    </Hook0Section>

    <!-- Response: loaded -->
    <template v-else-if="responseData">
      <Hook0Section :title="t('logs.response')" :separator="true">
        <div class="log-detail__grid">
          <!-- Summary -->
          <span class="log-detail__grid-label">{{ t('responses.httpStatusCode') }}</span>
          <span
            v-if="responseData.http_code != null"
            class="log-detail__status-badge"
            :class="statusCodeClass(responseData.http_code)"
            role="status"
            :aria-label="`HTTP ${responseData.http_code}`"
          >
            {{ responseData.http_code }}
          </span>
          <span v-else class="log-detail__grid-value">—</span>

          <span class="log-detail__grid-label">{{ t('responses.id') }}</span>
          <code class="log-detail__grid-value log-detail__grid-value--mono">{{
            responseData.response_id
          }}</code>

          <template v-if="responseData.elapsed_time_ms != null">
            <span class="log-detail__grid-label">{{ t('responses.elapsedTime') }}</span>
            <span class="log-detail__grid-value">{{
              t('responses.elapsedTimeMs', { ms: responseData.elapsed_time_ms })
            }}</span>
          </template>

          <template v-if="responseData.response_error_name">
            <span class="log-detail__grid-label">{{ t('responses.error') }}</span>
            <code
              class="log-detail__grid-value log-detail__grid-value--mono log-detail__grid-value--error"
              >{{ responseData.response_error_name }}</code
            >
          </template>

          <!-- Headers -->
          <h4 class="log-detail__subsection-title log-detail__grid-full">
            {{ t('responses.headers') }}
          </h4>
          <template v-if="filteredHeaders">
            <template v-for="(val, key) in filteredHeaders" :key="String(key)">
              <code class="log-detail__grid-label log-detail__grid-label--mono">{{ key }}</code>
              <code class="log-detail__grid-value log-detail__grid-value--mono">{{ val }}</code>
            </template>
          </template>
          <p v-else class="log-detail__grid-full log-detail__no-response">
            {{ t('responses.noHeaders') }}
          </p>
        </div>
        <p v-if="filteredHeaders && hasHiddenHeaders" class="log-detail__sensitive-note">
          {{ t('responses.sensitiveHidden') }}
        </p>

        <!-- Body -->
        <h4 class="log-detail__subsection-title">{{ t('responses.body') }}</h4>
        <div
          v-if="responseData.body_formatted"
          :class="{ 'log-detail__body--error': isErrorResponse }"
        >
          <Hook0Code :code="responseData.body_formatted" language="json" :editable="false" />
        </div>
        <p v-else class="log-detail__no-response">{{ t('responses.noBody') }}</p>
      </Hook0Section>
    </template>
  </template>
</template>

<style scoped>
:deep(.hook0-card-content-line) {
  padding: 0.25rem 0;
}

.log-detail__skeleton-meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.log-detail__skeleton-meta-row {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
}

.log-detail__subsection-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 1rem 0 0.5rem;
}

.log-detail__subsection-title:first-child {
  margin-top: 0;
}

.log-detail__meta-link {
  font-size: 0.8125rem;
}

.log-detail__meta-link--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.log-detail__labels {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.log-detail__label-badge {
  display: inline-flex;
  padding: 0.125rem 0.5rem;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--color-primary);
  background-color: var(--color-primary-light);
  border-radius: var(--radius-full);
}

.log-detail__grid {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0.375rem 0.75rem;
  align-items: baseline;
}

.log-detail__grid-full {
  grid-column: 1 / -1;
}

.log-detail__grid-label {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.log-detail__grid-label--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
}

.log-detail__grid-value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  word-break: break-all;
}

.log-detail__grid-value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.log-detail__grid-value--error {
  color: var(--color-error);
}

.log-detail__status-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 600;
  font-family: var(--font-mono);
  width: fit-content;
}

.log-detail__status-badge--success {
  background-color: var(--color-success-light);
  color: var(--color-success);
}

.log-detail__status-badge--warning {
  background-color: var(--color-warning-light);
  color: var(--color-warning);
}

.log-detail__status-badge--error {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.log-detail__status-badge--unknown {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-tertiary);
}

.log-detail__sensitive-note {
  margin: 0.5rem 0 0;
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-tertiary);
}

.log-detail__no-response {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
}

.log-detail__body--error {
  border-left: 3px solid var(--color-error);
  padding-left: 0.75rem;
}
</style>
