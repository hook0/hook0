<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';
import { isAxiosError } from '@/http';
import { toast } from 'vue-sonner';
import { RefreshCw } from 'lucide-vue-next';

import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Section from '@/components/Hook0Section.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Uuid from '@/components/Hook0Uuid.vue';
import LogLifecycle from './LogLifecycle.vue';

import { useEventDetail } from '@/pages/organizations/applications/events/useEventQueries';
import { useSubscriptionDetail } from '@/pages/organizations/applications/subscriptions/useSubscriptionQueries';
import { useResponseDetail } from './useResponseQueries';
import { useRetryDelivery } from './useLogQueries';
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

// Hook0 lifecycle states for an attempt:
//   Waiting    = scheduled for later (delay_until in the future, e.g. retry backoff)
//   Pending    = eligible but not yet picked up by a worker
//   InProgress = worker is executing the HTTP call right now
//   Successful = HTTP call got a 2xx response
//   Failed     = HTTP call got a non-2xx or timed out
//
// We show request/response details only for states where the HTTP call
// was actually made.  Without this guard, the UI shows empty response
// sections for queued attempts.
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

const retryMutation = useRetryDelivery(applicationIdRef);

function handleRetry() {
  retryMutation.mutate(props.attempt.request_attempt_id, {
    onSuccess: () => {
      toast.success(t('logs.retryQueued'));
    },
    onError: (err) => {
      if (isAxiosError(err)) {
        const status = err.response?.status;
        // 410 Gone: the event payload has been purged from storage,
        // so the backend cannot rebuild the outgoing HTTP request.
        if (status === 410) {
          toast.error(t('logs.payloadExpired'));
          return;
        }
        // 429 Too Many Requests: a retry for this event was already
        // triggered within the server-configured cooldown window.
        if (status === 429) {
          toast.error(t('logs.retryCooldown'));
          return;
        }
      }
      toast.error(t('common.unknownError'));
    },
  });
}

// When true, shows a "Manual Retry" badge in the metadata section so
// the user can distinguish user-triggered retries from automatic ones.
const isManualRetry = computed(() => props.attempt.attempt_trigger === 'manual_retry');

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
            <template v-if="attempt.subscription.description">{{
              attempt.subscription.description
            }}</template>
            <Hook0Uuid v-else :value="attempt.subscription.subscription_id" />
          </Hook0Button>
        </template>
      </Hook0CardContentLine>
      <Hook0CardContentLine v-if="isManualRetry" type="split">
        <template #label>{{ t('logs.trigger') }}</template>
        <template #content>
          <Hook0Badge variant="info" size="sm">{{ t('logs.manualRetryBadge') }}</Hook0Badge>
        </template>
      </Hook0CardContentLine>
    </Hook0Section>

    <!-- Retry action -->
    <Hook0Section>
      <Hook0Button
        variant="secondary"
        size="sm"
        :disabled="retryMutation.isPending.value"
        :aria-label="t('logs.retryDelivery')"
        @click="handleRetry"
      >
        <RefreshCw :size="14" aria-hidden="true" />
        {{ t('logs.retryDelivery') }}
      </Hook0Button>
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
      <div class="log-detail__badge-list">
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

    <!-- Request -->
    <Hook0Section :title="t('logs.request')" :separator="true">
      <template v-if="subscriptionData?.target">
        <Hook0CardContentLine type="split">
          <template #label>{{ t('logs.httpMethod') }}</template>
          <template #content>
            <code class="mono">{{ subscriptionData.target.method.toUpperCase() }}</code>
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine type="split">
          <template #label>{{ t('logs.targetUrl') }}</template>
          <template #content>
            <code class="mono">{{ subscriptionData.target.url }}</code>
          </template>
        </Hook0CardContentLine>
      </template>

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
        <!-- Summary -->
        <Hook0CardContentLine type="split">
          <template #label>{{ t('responses.httpStatusCode') }}</template>
          <template #content>
            <span
              v-if="responseData.http_code != null"
              class="log-detail__status-badge"
              :class="statusCodeClass(responseData.http_code)"
              role="status"
              :aria-label="`HTTP ${responseData.http_code}`"
            >
              {{ responseData.http_code }}
            </span>
            <span v-else>—</span>
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine type="split">
          <template #label>{{ t('responses.id') }}</template>
          <template #content>
            <code class="mono">{{ responseData.response_id }}</code>
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine v-if="responseData.elapsed_time_ms != null" type="split">
          <template #label>{{ t('responses.elapsedTime') }}</template>
          <template #content>{{
            t('responses.elapsedTimeMs', { ms: responseData.elapsed_time_ms })
          }}</template>
        </Hook0CardContentLine>
        <Hook0CardContentLine v-if="responseData.response_error_name" type="split">
          <template #label>{{ t('responses.error') }}</template>
          <template #content>
            <code class="mono mono--error">{{ responseData.response_error_name }}</code>
          </template>
        </Hook0CardContentLine>

        <!-- Headers -->
        <h4 class="log-detail__subsection-title">{{ t('responses.headers') }}</h4>
        <template v-if="filteredHeaders">
          <Hook0CardContentLine
            v-for="(val, key) in filteredHeaders"
            :key="String(key)"
            type="split"
          >
            <template #label>
              <code class="mono mono--bold">{{ key }}</code>
            </template>
            <template #content>
              <code class="mono">{{ val }}</code>
            </template>
          </Hook0CardContentLine>
        </template>
        <p v-else class="log-detail__no-response">{{ t('responses.noHeaders') }}</p>
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

.log-detail__badge-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  word-break: break-all;
}

.mono--bold {
  font-weight: 600;
}

.mono--error {
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
