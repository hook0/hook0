<script setup lang="ts">
import { computed } from 'vue';
import Hook0TooltipFullCopy from './Hook0TooltipFullCopy.vue';
import { formatDate } from '@/utils/formatDate';

type Props = {
  value: string | null;
  defaultText?: string;
};

const props = withDefaults(defineProps<Props>(), {
  defaultText: '—',
});

const formatted = computed(() => {
  if (!props.value) return null;
  const result = formatDate(props.value);
  // formatDate returns "—" for invalid dates; treat that as unformatted
  return result === '\u2014' ? props.value : result;
});

const isoValue = computed(() => {
  if (!props.value) return '';
  const date = new Date(props.value);
  if (Number.isNaN(date.getTime())) return props.value;
  return date.toISOString();
});
</script>

<template>
  <span v-if="!formatted" class="hook0-date-formatted--empty">{{ defaultText }}</span>
  <Hook0TooltipFullCopy v-else :value="isoValue" :mono="false">
    <span class="hook0-date-formatted">{{ formatted }}</span>
  </Hook0TooltipFullCopy>
</template>

<style scoped>
.hook0-date-formatted {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  white-space: nowrap;
}

.hook0-date-formatted--empty {
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
}
</style>
