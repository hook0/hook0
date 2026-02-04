<script setup lang="ts">
import { formatISO9075, formatDistance, parseISO } from 'date-fns';

import Hook0Text from '@/components/Hook0Text.vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  value: string | null;
  defaultText?: string;
}

const props = defineProps<Props>();

function formatDate(value: string | null): string {
  if (value === null || value === '') {
    return '';
  } else {
    return formatISO9075(parseISO(value));
  }
}

function formatHumanReadableDate(value: string | null): string {
  if (value === null || value === '') {
    return '';
  } else {
    return formatDistance(parseISO(value), new Date(), { addSuffix: true });
  }
}
</script>

<template>
  <abbr v-if="props.value" :title="formatDate(props.value)">
    <Hook0Text>
      {{ formatHumanReadableDate(props.value) }}
    </Hook0Text>
  </abbr>
  <Hook0Text v-else>
    {{ props.defaultText ?? '' }}
  </Hook0Text>
</template>
