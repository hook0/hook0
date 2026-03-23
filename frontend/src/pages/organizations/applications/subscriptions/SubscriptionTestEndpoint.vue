<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Send } from 'lucide-vue-next';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';

type Props = {
  targetUrl: string;
};

const props = defineProps<Props>();

const { t } = useI18n();

type TestEndpointResult = {
  status: number;
  latencyMs: number;
  body: string;
  success: boolean;
};

const testEndpointLoading = ref(false);
const testEndpointResult = ref<TestEndpointResult | null>(null);
const testEndpointError = ref<string | null>(null);

/**
 * Check whether the target URL looks like a valid URL (has protocol + host).
 */
const isValidUrl = computed(() => {
  if (!props.targetUrl) return false;
  try {
    const url = new URL(props.targetUrl);
    return url.protocol === 'http:' || url.protocol === 'https:';
  } catch {
    return false;
  }
});

function testEndpoint() {
  if (!props.targetUrl) {
    testEndpointError.value = t('subscriptions.testUrlRequired');
    return;
  }

  testEndpointLoading.value = true;
  testEndpointResult.value = null;
  testEndpointError.value = null;

  const startTime = performance.now();

  fetch(props.targetUrl, {
    method: 'HEAD',
    mode: 'no-cors',
    signal: AbortSignal.timeout(10000),
  })
    .then((response) => {
      const latencyMs = Math.round(performance.now() - startTime);

      // In no-cors mode, response.type is 'opaque' and status is 0
      // This means the request reached the server but we can't read the response
      if (response.type === 'opaque') {
        testEndpointResult.value = {
          status: 0,
          latencyMs,
          body: '',
          success: true,
        };
        return;
      }

      void response.text().then((body) => {
        testEndpointResult.value = {
          status: response.status,
          latencyMs,
          body: body.slice(0, 2000),
          success: response.ok,
        };
      });
    })
    .catch((err: Error) => {
      const latencyMs = Math.round(performance.now() - startTime);
      testEndpointError.value = err.message;
      testEndpointResult.value = {
        status: 0,
        latencyMs,
        body: '',
        success: false,
      };
    })
    .finally(() => {
      testEndpointLoading.value = false;
    });
}

const testStatusVariant = computed(() => {
  if (!testEndpointResult.value) return '';
  const status = testEndpointResult.value.status;
  if (status === 0 && testEndpointResult.value.success) return 'opaque';
  if (status >= 200 && status < 300) return 'success';
  if (status >= 400 && status < 500) return 'warning';
  if (status >= 500) return 'error';
  return 'error';
});
</script>

<template>
  <div
    class="test-endpoint"
    :class="{ 'test-endpoint--has-result': testEndpointResult || testEndpointError }"
  >
    <Hook0Button
      variant="secondary"
      size="sm"
      type="button"
      :loading="testEndpointLoading"
      :disabled="!isValidUrl"
      data-test="subscription-test-endpoint-button"
      @click="testEndpoint()"
    >
      <template #left>
        <Send :size="14" aria-hidden="true" />
      </template>
      {{ t('subscriptions.testEndpoint') }}
    </Hook0Button>

    <!-- Test Result -->
    <div
      v-if="testEndpointResult"
      class="test-endpoint__result"
      :class="`test-endpoint__result--${testStatusVariant}`"
      data-test="subscription-test-endpoint-result"
    >
      <div class="test-endpoint__result-header">
        <span class="test-endpoint__result-title">{{ t('subscriptions.testResponse') }}</span>
        <span
          class="test-endpoint__status-badge"
          :class="`test-endpoint__status-badge--${testStatusVariant}`"
          data-test="subscription-test-endpoint-status"
        >
          {{
            testEndpointResult.status === 0
              ? t('subscriptions.testSuccess')
              : testEndpointResult.status
          }}
        </span>
      </div>

      <dl class="test-endpoint__details">
        <div class="test-endpoint__detail-row">
          <dt class="test-endpoint__detail-label">
            {{ t('subscriptions.testLatency') }}
          </dt>
          <dd
            class="test-endpoint__detail-value test-endpoint__detail-value--mono"
            data-test="subscription-test-endpoint-latency"
          >
            {{ testEndpointResult.latencyMs }}ms
          </dd>
        </div>
      </dl>

      <div v-if="testEndpointResult.body" class="test-endpoint__body">
        <span class="test-endpoint__detail-label">{{ t('subscriptions.testBodyPreview') }}</span>
        <Hook0Code :code="testEndpointResult.body" data-test="subscription-test-endpoint-body" />
      </div>

      <Hook0HelpText
        v-if="testEndpointResult.status === 0 && testEndpointResult.success"
        tone="info"
      >
        {{ t('subscriptions.testCorsWarning') }}
      </Hook0HelpText>
    </div>

    <!-- Test Error -->
    <div
      v-if="testEndpointError && (!testEndpointResult || !testEndpointResult.success)"
      class="test-endpoint__error"
      data-test="subscription-test-endpoint-error"
    >
      <span class="test-endpoint__error-title">{{ t('subscriptions.testFailed') }}</span>
      <span class="test-endpoint__error-message">{{ testEndpointError }}</span>
    </div>
  </div>
</template>

<style scoped>
.test-endpoint {
  display: contents;
}

.test-endpoint__result {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 0.75rem;
  background-color: var(--color-bg-secondary);
  flex-basis: 100%;
  min-width: 0;
  margin-top: 0.25rem;
}

.test-endpoint__result--success {
  border-color: var(--color-success);
  background-color: var(--color-success-light);
}

.test-endpoint__result--warning {
  border-color: var(--color-warning);
  background-color: var(--color-warning-light);
}

.test-endpoint__result--error {
  border-color: var(--color-error);
  background-color: var(--color-error-light);
}

.test-endpoint__result--opaque {
  border-color: var(--color-info);
  background-color: var(--color-info-light);
}

.test-endpoint__result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  flex-wrap: wrap;
}

.test-endpoint__result-title {
  font-weight: 600;
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.test-endpoint__status-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.test-endpoint__status-badge--success {
  background-color: var(--color-success);
  color: var(--color-on-dark);
}

.test-endpoint__status-badge--warning {
  background-color: var(--color-warning);
  color: var(--color-on-dark);
}

.test-endpoint__status-badge--error {
  background-color: var(--color-error);
  color: var(--color-on-dark);
}

.test-endpoint__status-badge--opaque {
  background-color: var(--color-info);
  color: var(--color-on-dark);
}

.test-endpoint__details {
  margin: 0;
}

.test-endpoint__detail-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.test-endpoint__detail-label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.test-endpoint__detail-value {
  font-size: 0.75rem;
  color: var(--color-text-primary);
}

.test-endpoint__detail-value--mono {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
}

.test-endpoint__body {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  margin-top: 0.5rem;
}

.test-endpoint__error {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  border: 1px solid var(--color-error);
  border-radius: var(--radius-md);
  padding: 0.75rem;
  background-color: var(--color-error-light);
  flex-basis: 100%;
  min-width: 0;
  margin-top: 0.25rem;
}

.test-endpoint__error-title {
  font-weight: 600;
  font-size: 0.8125rem;
  color: var(--color-error);
}

.test-endpoint__error-message {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  word-break: break-all;
}
</style>
