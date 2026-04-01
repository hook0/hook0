<script setup lang="ts">
import { computed } from 'vue';
import { parseISO } from 'date-fns';
import Hook0TooltipFullCopy from './Hook0TooltipFullCopy.vue';

type Props = {
  value: string | null;
  defaultText?: string;
};

const props = withDefaults(defineProps<Props>(), {
  defaultText: '—',
});

const dateFmt = new Intl.DateTimeFormat(undefined, {
  day: 'numeric',
  month: 'short',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
});

const formatted = computed(() => {
  if (!props.value) return null;
  try {
    return dateFmt.format(parseISO(props.value));
  } catch {
    return props.value;
  }
});

const isoValue = computed(() => {
  if (!props.value) return '';
  try {
    return parseISO(props.value).toISOString();
  } catch {
    return props.value;
  }
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
