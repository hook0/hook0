<script setup lang="ts">
import { computed } from 'vue';
import Hook0TooltipFullCopy from './Hook0TooltipFullCopy.vue';

type Props = {
  method: string;
  url: string;
};

const props = defineProps<Props>();

const fullValue = computed(() => `${props.method} ${props.url}`);

/**
 * @description Splits a URL into 3 segments for smart truncation via CSS flex-shrink:
 *   - domain: the host (e.g. "example.com") — shrinks moderately (flex-shrink: 1)
 *   - midPath: intermediate path segments (e.g. "/api/v1") — shrinks first (flex-shrink: 100)
 *   - lastChunk: final path segment (e.g. "/webhooks") — never shrinks (flex-shrink: 0)
 * @example "https://example.com/api/v1/webhooks" => { domain: "example.com", midPath: "/api/v1", lastChunk: "/webhooks" }
 */
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
  <Hook0TooltipFullCopy :value="fullValue" :mono="false">
    <span class="cell-target">
      <span class="cell-target__method">{{ method }}</span>
      <span class="cell-target__domain">{{ parsed.domain }}</span>
      <span v-if="parsed.midPath" class="cell-target__mid">{{ parsed.midPath }}</span>
      <span class="cell-target__last">{{ parsed.lastChunk }}</span>
    </span>
  </Hook0TooltipFullCopy>
</template>

<style scoped>
.cell-target {
  display: inline-flex;
  align-items: baseline;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.4;
  max-width: 18rem;
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
