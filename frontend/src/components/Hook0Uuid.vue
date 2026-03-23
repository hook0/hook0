<script setup lang="ts">
import { computed } from 'vue';
import Hook0TooltipFullCopy from './Hook0TooltipFullCopy.vue';

type Props = {
  value: string;
  linked?: boolean;
  truncated?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  linked: false,
  truncated: false,
});

/**
 * @description Truncates a string in the middle, keeping start and end visible.
 * @example middleTruncate("abcdefghij", 7) => "abc…hij"
 */
function middleTruncate(value: string, maxChars: number): string {
  if (value.length <= maxChars) return value;
  const half = Math.floor((maxChars - 1) / 2);
  return `${value.slice(0, half)}\u2026${value.slice(-(maxChars - 1 - half))}`;
}

const display = computed(() => (props.truncated ? middleTruncate(props.value, 20) : props.value));
</script>

<template>
  <Hook0TooltipFullCopy v-if="truncated" :value="value" :linked="linked">
    <span class="hook0-uuid__select-all">{{ display }}</span>
  </Hook0TooltipFullCopy>
  <span
    v-else
    class="hook0-uuid hook0-uuid__select-all"
    :class="{ 'hook0-uuid--linked': linked }"
    >{{ display }}</span
  >
</template>

<style scoped>
.hook0-uuid__select-all {
  user-select: all;
}

.hook0-uuid {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.4;
  color: var(--color-text-primary);
}

.hook0-uuid--linked {
  cursor: pointer;
}

.hook0-uuid--linked:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>
