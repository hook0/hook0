<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import { buildPlayInspectUrl, buildPlayViewUrl } from '@/utils/playEndpoint';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Spinner from '@/components/Hook0Spinner.vue';

type Props = {
  base: string;
  token: string;
};

const props = defineProps<Props>();

const { t } = useI18n();

// Fields mirror play/src/storage/mod.rs (StoredWebhook, JSON serialization).
// `body` is base64-encoded; `received_at` is an RFC 3339 timestamp.
type PlayWebhook = {
  id: string;
  method: string;
  path: string;
  headers: Record<string, string>;
  body: string;
  body_size: number;
  received_at: string;
  forwarded: boolean;
};

// Bounded polling: stop as soon as a webhook lands, or after ~60s.
const POLL_INTERVAL_MS = 2500;
const MAX_ATTEMPTS = 24;
const FETCH_TIMEOUT_MS = 10000;

const lastWebhook = ref<PlayWebhook | null>(null);
const errored = ref(false);

let attempts = 0;
let pollTimer: ReturnType<typeof setInterval> | undefined;

function stopPolling() {
  if (pollTimer !== undefined) {
    clearInterval(pollTimer);
    pollTimer = undefined;
  }
}

function poll() {
  attempts += 1;

  fetch(buildPlayInspectUrl(props.base, props.token), {
    signal: AbortSignal.timeout(FETCH_TIMEOUT_MS),
  })
    .then((response) => response.json())
    .then((data: { webhooks?: PlayWebhook[] }) => {
      errored.value = false;
      const webhooks = data.webhooks ?? [];

      if (webhooks.length > 0) {
        // Storage returns webhooks in map order, so pick the most recent one.
        lastWebhook.value = webhooks.reduce((latest, current) =>
          new Date(current.received_at).getTime() >= new Date(latest.received_at).getTime()
            ? current
            : latest
        );
        stopPolling();
        return;
      }

      if (attempts >= MAX_ATTEMPTS) {
        stopPolling();
      }
    })
    .catch(() => {
      errored.value = true;
      if (attempts >= MAX_ATTEMPTS) {
        stopPolling();
      }
    });
}

function restart() {
  stopPolling();
  attempts = 0;
  lastWebhook.value = null;
  errored.value = false;

  if (!props.token) {
    return;
  }

  poll();
  pollTimer = setInterval(poll, POLL_INTERVAL_MS);
}

watch(() => props.token, restart, { immediate: true });

onBeforeUnmount(stopPolling);

const viewUrl = computed(() => buildPlayViewUrl(props.base, props.token));

const receivedAt = computed(() => {
  if (lastWebhook.value === null) {
    return '';
  }
  return new Date(lastWebhook.value.received_at).toLocaleTimeString();
});

const formattedBody = computed(() => {
  if (lastWebhook.value === null || lastWebhook.value.body.length === 0) {
    return '';
  }

  let decoded: string;
  try {
    const binary = atob(lastWebhook.value.body);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    decoded = new TextDecoder().decode(bytes);
  } catch {
    return '';
  }

  try {
    return JSON.stringify(JSON.parse(decoded), null, 2);
  } catch {
    return decoded;
  }
});
</script>

<template>
  <div class="play-inspector" data-test="play-inspector">
    <!-- Received: the "aha" moment -->
    <div
      v-if="lastWebhook !== null"
      class="play-inspector__received"
      data-test="play-inspector-received"
    >
      <div class="play-inspector__header">
        <span class="play-inspector__title">{{ t('subscriptions.playReceived') }}</span>
        <span class="play-inspector__method">{{ lastWebhook.method }}</span>
        <span class="play-inspector__time">{{
          t('subscriptions.playReceivedAt', { time: receivedAt })
        }}</span>
      </div>
      <pre v-if="formattedBody" class="play-inspector__body" data-test="play-inspector-body">{{
        formattedBody
      }}</pre>
      <Hook0Button variant="link" :href="viewUrl" target="_blank">
        {{ t('subscriptions.playOpenInspector') }}
      </Hook0Button>
    </div>

    <!-- Error: couldn't reach the inbox -->
    <div v-else-if="errored" class="play-inspector__error" data-test="play-inspector-error">
      <span>{{ t('subscriptions.playError') }}</span>
      <Hook0Button variant="link" :href="viewUrl" target="_blank">
        {{ t('subscriptions.playOpenInspector') }}
      </Hook0Button>
    </div>

    <!-- Waiting for the first webhook -->
    <div v-else class="play-inspector__waiting" data-test="play-inspector-waiting">
      <div class="play-inspector__waiting-head">
        <Hook0Spinner :size="16" />
        <span class="play-inspector__title">{{ t('subscriptions.playWaiting') }}</span>
      </div>
      <span class="play-inspector__hint">{{ t('subscriptions.playWaitingHint') }}</span>
      <Hook0Button variant="link" :href="viewUrl" target="_blank">
        {{ t('subscriptions.playOpenInspector') }}
      </Hook0Button>
    </div>
  </div>
</template>

<style scoped>
.play-inspector {
  margin-top: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-secondary);
}

.play-inspector__waiting,
.play-inspector__received,
.play-inspector__error {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.play-inspector__waiting-head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-text-secondary);
}

.play-inspector__header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.play-inspector__title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.play-inspector__method {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.0625rem 0.375rem;
  border-radius: var(--radius-sm);
  background-color: var(--color-success-light);
  color: var(--color-success);
}

.play-inspector__time {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.play-inspector__hint {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.play-inspector__body {
  margin: 0;
  padding: 0.5rem;
  max-height: 12rem;
  overflow: auto;
  border-radius: var(--radius-sm);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  white-space: pre;
}

.play-inspector__error {
  color: var(--color-error);
  font-size: 0.8125rem;
}

.play-inspector :deep(.hook0-button) {
  align-self: flex-start;
  padding-left: 0;
  padding-right: 0;
}
</style>
