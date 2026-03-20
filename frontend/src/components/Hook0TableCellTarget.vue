<script setup lang="ts">
import { computed } from 'vue';
import Hook0TruncatedText from './Hook0TruncatedText.vue';

type Props = {
  method: string;
  url: string;
};

const props = defineProps<Props>();

const fullValue = computed(() => `${props.method} ${props.url}`);

const parsed = computed(() => {
  try {
    const u = new URL(props.url);
    const segments = u.pathname.split('/').filter(Boolean);
    const domain = u.host;
    if (segments.length <= 1) {
      return { domain, midPath: '', lastChunk: u.pathname };
    }
    const midPath = '/' + segments.slice(0, -1).join('/');
    const lastChunk = '/' + (segments.at(-1) ?? '');
    return { domain, midPath, lastChunk };
  } catch {
    return { domain: props.url, midPath: '', lastChunk: '' };
  }
});
</script>

<template>
  <Hook0TruncatedText :value="fullValue" :display="fullValue">
    <span class="cell-target">
      <span class="cell-target__method">{{ method }}</span>
      <span class="cell-target__domain">{{ parsed.domain }}</span>
      <span v-if="parsed.midPath" class="cell-target__mid">{{ parsed.midPath }}</span>
      <span class="cell-target__last">{{ parsed.lastChunk }}</span>
    </span>
  </Hook0TruncatedText>
</template>

<style scoped>
.cell-target {
  display: inline-flex;
  align-items: baseline;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.4;
  max-width: 15rem;
  overflow: hidden;
}

.cell-target__method {
  color: var(--color-text-primary);
  font-weight: 700;
  flex-shrink: 0;
  margin-right: 0.375rem;
}

.cell-target__domain {
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
  min-width: 3rem;
}

.cell-target__mid {
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 100;
  min-width: 1.5rem;
}

.cell-target__last {
  color: var(--color-text-secondary);
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
