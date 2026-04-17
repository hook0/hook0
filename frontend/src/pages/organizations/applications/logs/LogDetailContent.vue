<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';
import { toast } from 'vue-sonner';
import { handleMutationError } from '@/utils/handleMutationError';
import {
  RefreshCw,
  ArrowRight,
  Clock,
  Copy,
  CheckCircle2,
  XCircle,
  CircleDashed,
  Loader,
} from 'lucide-vue-next';

import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Section from '@/components/Hook0Section.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Uuid from '@/components/Hook0Uuid.vue';
import LogLifecycle from './LogLifecycle.vue';

import { useEventDetail } from '@/pages/organizations/applications/events/useEventQueries';
import { useSubscriptionDetail } from '@/pages/organizations/applications/subscriptions/useSubscriptionQueries';
import { useResponseDetail } from './useResponseQueries';
import { useRetryDelivery } from './useLogQueries';
import { filterSensitiveHeaders } from './responseHeaders';
import { routes } from '@/routes';

import type { RequestAttempt } from './LogService';
import { RequestAttemptStatusType } from './LogService';
import { getStatusConfig } from './logStatusConfig';

type Props = {
  attempt: RequestAttempt;
  applicationId: string;
};

const props = defineProps<Props>();
const { t } = useI18n();
const route = useRoute();

const statusConfig = computed(() => getStatusConfig(props.attempt.status.type));

const statusLabel = computed(() => {
  const httpCode = props.attempt.http_response_status;
  if (httpCode != null) return `${httpCode}`;
  if (props.attempt.status.type === RequestAttemptStatusType.Failed && !props.attempt.response_id) {
    return t('logs.statusTimeout');
  }
  return t(statusConfig.value.labelKey);
});

const statusDescription = computed(() => {
  const type = props.attempt.status.type;
  if (type === RequestAttemptStatusType.Successful) return t('logs.statusSuccessfulDesc');
  if (type === RequestAttemptStatusType.Failed) return t('logs.statusFailedDesc');
  if (type === RequestAttemptStatusType.InProgress) return t('logs.statusInProgressDesc');
  if (type === RequestAttemptStatusType.Pending) return t('logs.statusPendingDesc');
  if (type === RequestAttemptStatusType.Waiting) return t('logs.statusWaitingDesc');
  return '';
});

const statusIcon = computed(() => {
  const type = props.attempt.status.type;
  if (type === RequestAttemptStatusType.Successful) return CheckCircle2;
  if (type === RequestAttemptStatusType.Failed) return XCircle;
  if (type === RequestAttemptStatusType.InProgress) return Loader;
  if (type === RequestAttemptStatusType.Waiting) return Clock;
  return CircleDashed;
});

// We show request/response details only for states where the HTTP call
// was actually made.
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

const retryMutation = useRetryDelivery();

function handleRetry() {
  retryMutation.mutate(props.attempt.request_attempt_id, {
    onSuccess: () => {
      toast.success(t('logs.retryQueued'));
    },
    onError: (err) => {
      handleMutationError(err);
    },
  });
}

const isManualRetry = computed(() => props.attempt.attempt_trigger === 'manual_retry');

const canRetry = computed(() => {
  const status = props.attempt.status.type;
  return (
    status === RequestAttemptStatusType.Failed || status === RequestAttemptStatusType.Successful
  );
});

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

const responseStatusClass = computed(() => {
  const code = responseData.value?.http_code ?? props.attempt.http_response_status;
  if (code == null) return 'detail-response__code--unknown';
  if (code >= 200 && code < 300) return 'detail-response__code--success';
  if (code >= 300 && code < 400) return 'detail-response__code--warning';
  return 'detail-response__code--error';
});

const isErrorResponse = computed(() => {
  const code = responseData.value?.http_code ?? props.attempt.http_response_status;
  return code != null && code >= 400;
});

function copyUrl() {
  if (subscriptionData.value?.target?.url) {
    void navigator.clipboard.writeText(subscriptionData.value.target.url);
  }
}
</script>

<template>
  <Hook0Section v-if="eventError && !eventLoading">
    <Hook0ErrorCard :error="eventError" @retry="void eventRefetch()" />
  </Hook0Section>

  <template v-else-if="eventLoading || !eventData">
    <!-- Skeleton: status header -->
    <div class="detail-header detail-header--muted">
      <Hook0Skeleton size="text" style="width: 4rem; height: 1.5rem" />
      <Hook0Skeleton size="text" style="width: 12rem" />
    </div>
    <!-- Skeleton: metadata -->
    <Hook0Section>
      <div class="detail-meta">
        <div v-for="i in 3" :key="i" class="detail-meta__row">
          <Hook0Skeleton size="text" style="width: 5rem" />
          <Hook0Skeleton size="text-truncated" style="width: 10rem" />
        </div>
      </div>
    </Hook0Section>
    <!-- Skeleton: lifecycle -->
    <Hook0Section>
      <Hook0Skeleton size="text" style="width: 5rem; margin-bottom: 0.75rem" />
      <div v-for="i in 5" :key="i" class="detail-skeleton-lifecycle-row">
        <Hook0Skeleton size="text" style="width: 1rem; height: 1rem; border-radius: 50%" />
        <div style="flex: 1">
          <Hook0Skeleton size="text" style="width: 6rem" />
          <Hook0Skeleton size="text" style="width: 12rem; margin-top: 0.25rem" />
        </div>
      </div>
    </Hook0Section>
  </template>

  <!-- Event loaded -->
  <template v-else>
    <!-- STATUS HEADER -->
    <div class="detail-header" :class="`detail-header--${statusConfig.variant}`">
      <component :is="statusIcon" :size="16" aria-hidden="true" class="detail-header__icon" />
      <span class="detail-header__label">{{ statusLabel }}</span>
      <span v-if="statusDescription" class="detail-header__sep">&mdash;</span>
      <span v-if="statusDescription" class="detail-header__description">{{
        statusDescription
      }}</span>
    </div>

    <!-- METADATA -->
    <Hook0Section>
      <dl class="detail-meta">
        <div class="detail-meta__row">
          <dt class="detail-meta__label">{{ t('logs.event') }}</dt>
          <dd class="detail-meta__value">
            <Hook0Button
              variant="link"
              size="sm"
              :to="{
                name: routes.EventsDetail,
                params: {
                  organization_id: route.params.organization_id,
                  application_id: route.params.application_id,
                  event_id: eventData.event_id,
                },
              }"
              class="detail-meta__link detail-meta__link--mono"
            >
              {{ eventData.event_type_name }}
            </Hook0Button>
          </dd>
        </div>
        <div class="detail-meta__row">
          <dt class="detail-meta__label">{{ t('logs.subscription') }}</dt>
          <dd class="detail-meta__value">
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
              class="detail-meta__link"
            >
              <template v-if="attempt.subscription.description">{{
                attempt.subscription.description
              }}</template>
              <Hook0Uuid v-else :value="attempt.subscription.subscription_id" />
            </Hook0Button>
          </dd>
        </div>
        <div v-if="isManualRetry" class="detail-meta__row">
          <dt class="detail-meta__label">{{ t('logs.trigger') }}</dt>
          <dd class="detail-meta__value">{{ t('logs.manualRetryLabel') }}</dd>
        </div>
      </dl>
      <Hook0Button
        v-if="canRetry"
        variant="secondary"
        size="xs"
        :disabled="retryMutation.isPending.value"
        :aria-label="t('logs.retryDelivery')"
        @click="handleRetry"
      >
        <RefreshCw :size="14" aria-hidden="true" />
        {{ t('logs.retryDelivery') }}
      </Hook0Button>
    </Hook0Section>

    <!-- LIFECYCLE -->
    <Hook0Section :title="t('logs.lifecycle.title')">
      <LogLifecycle :event="eventData" :attempt="attempt" />
    </Hook0Section>

    <!-- LABELS -->
    <Hook0Section
      v-if="eventData.labels && Object.keys(eventData.labels as Record<string, string>).length > 0"
      :title="t('events.labels')"
    >
      <div class="detail-badges">
        <Hook0Badge
          v-for="(val, key) in eventData.labels as Record<string, string>"
          :key="String(key)"
          variant="primary"
          size="sm"
        >
          {{ key }}={{ val }}
        </Hook0Badge>
      </div>
    </Hook0Section>

    <!-- REQUEST -->
    <Hook0Section :title="t('logs.request')" :separator="true">
      <template v-if="subscriptionData?.target">
        <div class="detail-request-bar">
          <span class="detail-request-bar__method">{{
            subscriptionData.target.method.toUpperCase()
          }}</span>
          <code class="detail-request-bar__url">{{ subscriptionData.target.url }}</code>
          <button
            class="detail-request-bar__copy"
            type="button"
            aria-label="Copy URL"
            @click="copyUrl"
          >
            <Copy :size="14" aria-hidden="true" />
          </button>
        </div>
      </template>

      <h4 class="detail-subsection-title">{{ t('events.payload') }}</h4>
      <Hook0Code :code="eventData.payload_decoded" language="json" :editable="false" />
    </Hook0Section>

    <!-- RESPONSE: not sent yet -->
    <Hook0Section v-if="!isSent" :title="t('logs.response')" :separator="true">
      <div class="detail-empty-response">
        <Clock :size="16" aria-hidden="true" class="detail-empty-response__icon" />
        <p class="detail-empty-response__text">{{ t('responses.notSentYet') }}</p>
      </div>
    </Hook0Section>

    <!-- RESPONSE: sent but no response (timeout) -->
    <Hook0Section v-else-if="!attempt.response_id" :title="t('logs.response')" :separator="true">
      <div class="detail-empty-response">
        <XCircle
          :size="16"
          aria-hidden="true"
          class="detail-empty-response__icon detail-empty-response__icon--error"
        />
        <p class="detail-empty-response__text">{{ t('responses.noResponse') }}</p>
      </div>
    </Hook0Section>

    <!-- RESPONSE: loading -->
    <template v-else-if="responseLoading || (!responseData && !responseError)">
      <Hook0Section>
        <Hook0Skeleton size="text" />
        <Hook0Skeleton size="text-truncated" />
      </Hook0Section>
      <Hook0Section>
        <Hook0Skeleton size="block" />
      </Hook0Section>
    </template>

    <!-- RESPONSE: error -->
    <Hook0Section v-else-if="responseError">
      <Hook0ErrorCard :error="responseError" @retry="void responseRefetch()" />
    </Hook0Section>

    <!-- RESPONSE: loaded -->
    <template v-else-if="responseData">
      <Hook0Section :title="t('logs.response')" :separator="true">
        <!-- Status + elapsed time summary -->
        <div class="detail-response-summary">
          <span
            v-if="responseData.http_code != null"
            class="detail-response__code"
            :class="responseStatusClass"
            role="status"
            :aria-label="`HTTP ${responseData.http_code}`"
          >
            {{ responseData.http_code }}
          </span>
          <span v-else class="detail-response__code detail-response__code--unknown">--</span>

          <template v-if="responseData.elapsed_time_ms != null">
            <ArrowRight :size="14" aria-hidden="true" class="detail-response-summary__arrow" />
            <span class="detail-response-summary__latency">
              {{ t('responses.elapsedTimeMs', { ms: responseData.elapsed_time_ms }) }}
            </span>
          </template>
        </div>

        <!-- Error name -->
        <div v-if="responseData.response_error_name" class="detail-response-error">
          <code class="detail-response-error__code">{{ responseData.response_error_name }}</code>
        </div>

        <!-- Headers -->
        <h4 class="detail-subsection-title">{{ t('responses.headers') }}</h4>
        <template v-if="filteredHeaders && Object.keys(filteredHeaders).length > 0">
          <div class="detail-headers">
            <div
              v-for="(val, key) in filteredHeaders"
              :key="String(key)"
              class="detail-headers__row"
            >
              <code class="detail-headers__key">{{ key }}</code>
              <code class="detail-headers__value">{{ val }}</code>
            </div>
          </div>
        </template>
        <p v-else class="detail-muted-text">{{ t('responses.noHeaders') }}</p>
        <p v-if="filteredHeaders && hasHiddenHeaders" class="detail-sensitive-note">
          {{ t('responses.sensitiveHidden') }}
        </p>

        <!-- Body -->
        <h4 class="detail-subsection-title">{{ t('responses.body') }}</h4>
        <div
          v-if="responseData.body_formatted"
          :class="{ 'detail-response-body--error': isErrorResponse }"
        >
          <Hook0Code :code="responseData.body_formatted" language="json" :editable="false" />
        </div>
        <p v-else class="detail-muted-text">{{ t('responses.noBody') }}</p>
      </Hook0Section>
    </template>
  </template>
</template>

<style scoped>
/* ================================================
   STATUS HEADER
   ================================================ */
.detail-header {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md);
  margin-bottom: 1rem;
}

.detail-header--success {
  background-color: color-mix(in srgb, var(--color-success) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-success) 20%, transparent);
}

.detail-header--error {
  background-color: color-mix(in srgb, var(--color-error) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-error) 20%, transparent);
}

.detail-header--warning {
  background-color: color-mix(in srgb, var(--color-warning) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-warning) 20%, transparent);
}

.detail-header--info {
  background-color: color-mix(in srgb, var(--color-info) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-info) 20%, transparent);
}

.detail-header--muted {
  background-color: var(--color-bg-tertiary);
  border: 1px solid var(--color-border);
}

.detail-header__icon {
  flex-shrink: 0;
}

.detail-header--success .detail-header__icon {
  color: var(--color-success);
}

.detail-header--error .detail-header__icon {
  color: var(--color-error);
}

.detail-header--warning .detail-header__icon {
  color: var(--color-warning);
}

.detail-header--info .detail-header__icon {
  color: var(--color-info);
}

.detail-header--muted .detail-header__icon {
  color: var(--color-text-tertiary);
}

.detail-header__label {
  font-size: 0.875rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.detail-header__sep {
  color: var(--color-text-tertiary);
  font-size: 0.75rem;
}

.detail-header--success .detail-header__label {
  color: var(--color-success);
}

.detail-header--error .detail-header__label {
  color: var(--color-error);
}

.detail-header--warning .detail-header__label {
  color: var(--color-warning);
}

.detail-header--info .detail-header__label {
  color: var(--color-info);
}

.detail-header--muted .detail-header__label {
  color: var(--color-text-tertiary);
}

.detail-header__description {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

/* ================================================
   METADATA
   ================================================ */
.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin: 0;
}

.detail-meta__row {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
}

.detail-meta__label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-tertiary);
  min-width: 5.5rem;
  flex-shrink: 0;
}

.detail-meta__value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  min-width: 0;
}

.detail-meta__link {
  font-size: 0.8125rem;
}

.detail-meta__link--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.detail-meta + .hook0-button {
  margin-top: 0.75rem;
}

/* ================================================
   LABELS / BADGES
   ================================================ */
.detail-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

/* ================================================
   REQUEST BAR
   ================================================ */
.detail-request-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background-color: var(--color-bg-tertiary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  margin-bottom: 1rem;
  overflow: hidden;
}

.detail-request-bar__method {
  font-size: 0.6875rem;
  font-weight: 700;
  font-family: var(--font-mono);
  letter-spacing: 0.03em;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-sm);
  background-color: var(--color-primary);
  color: var(--color-primary-text, #fff);
  flex-shrink: 0;
}

.detail-request-bar__url {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.detail-request-bar__copy {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem;
  border: none;
  background: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
}

.detail-request-bar__copy:hover {
  color: var(--color-text-primary);
  background-color: color-mix(in srgb, var(--color-text-primary) 8%, transparent);
}

/* ================================================
   SUBSECTION TITLES
   ================================================ */
.detail-subsection-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 1rem 0 0.5rem;
}

.detail-subsection-title:first-child {
  margin-top: 0;
}

/* ================================================
   RESPONSE SUMMARY
   ================================================ */
.detail-response-summary {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.detail-response__code {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.75rem;
  border-radius: var(--radius-full);
  font-size: 0.875rem;
  font-weight: 700;
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums;
}

.detail-response__code--success {
  background-color: color-mix(in srgb, var(--color-success) 12%, transparent);
  color: var(--color-success);
}

.detail-response__code--warning {
  background-color: color-mix(in srgb, var(--color-warning) 12%, transparent);
  color: var(--color-warning);
}

.detail-response__code--error {
  background-color: color-mix(in srgb, var(--color-error) 12%, transparent);
  color: var(--color-error);
}

.detail-response__code--unknown {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-tertiary);
}

.detail-response-summary__arrow {
  color: var(--color-text-tertiary);
}

.detail-response-summary__latency {
  font-size: 0.8125rem;
  font-family: var(--font-mono);
  color: var(--color-text-secondary);
  font-variant-numeric: tabular-nums;
}

/* ================================================
   RESPONSE ERROR
   ================================================ */
.detail-response-error {
  margin-bottom: 0.75rem;
  padding: 0.375rem 0.75rem;
  background-color: color-mix(in srgb, var(--color-error) 6%, transparent);
  border-radius: var(--radius-md);
  border-left: 3px solid var(--color-error);
}

.detail-response-error__code {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-error);
  word-break: break-all;
}

/* ================================================
   RESPONSE HEADERS
   ================================================ */
.detail-headers {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.5rem 0.75rem;
  background-color: var(--color-bg-tertiary);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
}

.detail-headers__row {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
  padding: 0.125rem 0;
}

.detail-headers__key {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  flex-shrink: 0;
  word-break: break-all;
}

.detail-headers__value {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
  word-break: break-all;
  min-width: 0;
}

/* ================================================
   EMPTY RESPONSE STATE
   ================================================ */
.detail-empty-response {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background-color: var(--color-bg-tertiary);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
}

.detail-empty-response__icon {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.detail-empty-response__icon--error {
  color: var(--color-error);
}

.detail-empty-response__text {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
}

/* ================================================
   RESPONSE BODY
   ================================================ */
.detail-response-body--error {
  border-left: 3px solid var(--color-error);
  padding-left: 0.75rem;
}

/* ================================================
   SHARED
   ================================================ */
.detail-muted-text {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
}

.detail-sensitive-note {
  margin: 0.5rem 0 0;
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-tertiary);
}

/* Skeleton helpers */
.detail-skeleton-lifecycle-row {
  display: flex;
  gap: 0.625rem;
  margin-bottom: 0.75rem;
}
</style>
