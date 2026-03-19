<script setup lang="ts">
import { formatISO9075, formatDistance, parseISO } from 'date-fns';

defineOptions({
  inheritAttrs: false,
});

type Props = {
  value: string | null;
  defaultText?: string;
};

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
  <abbr v-if="props.value" class="table-cell-date__value" :title="formatDate(props.value)">
    {{ formatHumanReadableDate(props.value) }}
  </abbr>
  <span v-else class="table-cell-date__value">
    {{ props.defaultText ?? '' }}
  </span>
</template>

<style scoped>
.table-cell-date__value {
  color: var(--color-text-secondary);
  font-weight: 400;
  font-size: 0.875rem;
  line-height: 1.5;
  text-decoration: none;
}
</style>
