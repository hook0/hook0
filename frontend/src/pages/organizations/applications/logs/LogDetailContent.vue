<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useI18n } from 'vue-i18n';

import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';

import { useEventDetail } from '@/pages/organizations/applications/events/useEventQueries';
import { useResponseDetail } from './useResponseQueries';
import { filterSensitiveHeaders } from './responseHeaders';
import { statusCodeClass } from './responseStatus';

type Props = {
  eventId: string;
  applicationId: string;
  responseId: string | null;
  httpResponseStatus: number | null;
};

const props = defineProps<Props>();
const { t } = useI18n();

const eventIdRef = toRef(props, 'eventId');
const applicationIdRef = toRef(props, 'applicationId');
const responseIdRef = computed(() => props.responseId ?? '');

const { data: eventData, isLoading: eventLoading } = useEventDetail(eventIdRef, applicationIdRef);
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
  const code = responseData.value?.http_code ?? props.httpResponseStatus;
  return code != null && code >= 400;
});
</script>

<template>
  <!-- Loading -->
  <template v-if="eventLoading || !eventData">
    <div class="log-detail__section">
      <Hook0Skeleton size="text-truncated" />
      <Hook0Skeleton size="text" />
      <Hook0Skeleton size="text-truncated" />
    </div>
    <div class="log-detail__section">
      <Hook0Skeleton size="block" />
    </div>
  </template>

  <!-- Event loaded -->
  <template v-else>
    <!-- Metadata -->
    <div class="log-detail__section">
      <h3 class="log-detail__section-title">{{ t('events.metadata') }}</h3>
      <div class="log-detail__meta">
        <div class="log-detail__meta-row">
          <span class="log-detail__meta-label">{{ t('events.id') }}</span>
          <code class="log-detail__meta-value log-detail__meta-value--mono">{{
            eventData.event_id
          }}</code>
        </div>
        <div class="log-detail__meta-row">
          <span class="log-detail__meta-label">{{ t('events.type') }}</span>
          <code class="log-detail__meta-value log-detail__meta-value--mono">{{
            eventData.event_type_name
          }}</code>
        </div>
        <div class="log-detail__meta-row">
          <span class="log-detail__meta-label">{{ t('events.receivedAt') }}</span>
          <span class="log-detail__meta-value">{{ eventData.received_at }}</span>
        </div>
        <div class="log-detail__meta-row">
          <span class="log-detail__meta-label">{{ t('events.occurredAt') }}</span>
          <span class="log-detail__meta-value">{{ eventData.occurred_at }}</span>
        </div>
      </div>
    </div>

    <!-- Labels -->
    <div
      v-if="
        eventData.labels && Object.keys(eventData.labels as Record<string, string>).length > 0
      "
      class="log-detail__section"
    >
      <h3 class="log-detail__section-title">{{ t('events.labels') }}</h3>
      <div class="log-detail__labels">
        <span
          v-for="(val, key) in eventData.labels as Record<string, string>"
          :key="String(key)"
          class="log-detail__label-badge"
        >
          {{ key }}={{ val }}
        </span>
      </div>
    </div>

    <!-- Payload -->
    <div class="log-detail__section">
      <h3 class="log-detail__section-title">{{ t('events.payload') }}</h3>
      <Hook0Code :code="eventData.payload_decoded" language="json" :editable="false" />
    </div>

    <!-- Response: no responseId -->
    <div v-if="!responseId" class="log-detail__section">
      <h3 class="log-detail__section-title">{{ t('responses.detail') }}</h3>
      <p class="log-detail__no-response">{{ t('responses.noResponse') }}</p>
    </div>

    <!-- Response: loading -->
    <template v-else-if="responseLoading || (!responseData && !responseError)">
      <div class="log-detail__section">
        <Hook0Skeleton size="text" />
        <Hook0Skeleton size="text-truncated" />
      </div>
      <div class="log-detail__section">
        <Hook0Skeleton size="block" />
      </div>
    </template>

    <!-- Response: error -->
    <div v-else-if="responseError" class="log-detail__section">
      <Hook0ErrorCard :error="responseError" @retry="void responseRefetch()" />
    </div>

    <!-- Response: loaded -->
    <template v-else-if="responseData">
      <!-- Response Summary -->
      <div class="log-detail__section">
        <h3 class="log-detail__section-title">{{ t('responses.summary') }}</h3>
        <div class="log-detail__meta">
          <div class="log-detail__meta-row">
            <span class="log-detail__meta-label">{{ t('responses.id') }}</span>
            <code class="log-detail__meta-value log-detail__meta-value--mono">{{
              responseData.response_id
            }}</code>
          </div>
          <div class="log-detail__meta-row">
            <span class="log-detail__meta-label">{{ t('responses.httpStatusCode') }}</span>
            <span
              v-if="responseData.http_code != null"
              class="log-detail__status-badge"
              :class="statusCodeClass(responseData.http_code)"
            >
              {{ responseData.http_code }}
            </span>
            <span v-else class="log-detail__meta-value">—</span>
          </div>
          <div v-if="responseData.elapsed_time_ms != null" class="log-detail__meta-row">
            <span class="log-detail__meta-label">{{ t('responses.elapsedTime') }}</span>
            <span class="log-detail__meta-value">{{
              t('responses.elapsedTimeMs', { ms: responseData.elapsed_time_ms })
            }}</span>
          </div>
          <div v-if="responseData.response_error_name" class="log-detail__meta-row">
            <span class="log-detail__meta-label">{{ t('responses.error') }}</span>
            <code class="log-detail__meta-value log-detail__meta-value--mono log-detail__meta-value--error">{{
              responseData.response_error_name
            }}</code>
          </div>
        </div>
      </div>

      <!-- Response Headers -->
      <div class="log-detail__section">
        <h3 class="log-detail__section-title">{{ t('responses.headers') }}</h3>
        <template v-if="filteredHeaders">
          <div class="log-detail__headers">
            <div
              v-for="(val, key) in filteredHeaders"
              :key="String(key)"
              class="log-detail__header-row"
            >
              <code class="log-detail__header-key">{{ key }}</code>
              <code class="log-detail__header-value">{{ val }}</code>
            </div>
          </div>
          <p v-if="hasHiddenHeaders" class="log-detail__sensitive-note">
            {{ t('responses.sensitiveHidden') }}
          </p>
        </template>
        <p v-else class="log-detail__no-response">{{ t('responses.noHeaders') }}</p>
      </div>

      <!-- Response Body -->
      <div class="log-detail__section">
        <h3 class="log-detail__section-title">{{ t('responses.body') }}</h3>
        <div v-if="responseData.body_formatted" :class="{ 'log-detail__body--error': isErrorResponse }">
          <Hook0Code :code="responseData.body_formatted" language="json" :editable="false" />
        </div>
        <p v-else class="log-detail__no-response">{{ t('responses.noBody') }}</p>
      </div>
    </template>
  </template>
</template>

<style scoped>
.log-detail__section {
  margin-bottom: 1.25rem;
}

.log-detail__section:last-child {
  margin-bottom: 0;
}

.log-detail__section-title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-tertiary);
  margin: 0 0 0.75rem;
}

.log-detail__meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.log-detail__meta-row {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
}

.log-detail__meta-label {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  min-width: 6rem;
  flex-shrink: 0;
}

.log-detail__meta-value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  word-break: break-all;
}

.log-detail__meta-value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.log-detail__meta-value--error {
  color: var(--color-error);
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

.log-detail__status-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 600;
  font-family: var(--font-mono);
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
  color: var(--color-text-tertiary);
}

.log-detail__headers {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.log-detail__header-row {
  display: flex;
  gap: 0.75rem;
  align-items: baseline;
}

.log-detail__header-key {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  min-width: 8rem;
  flex-shrink: 0;
}

.log-detail__header-value {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-primary);
  word-break: break-all;
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
