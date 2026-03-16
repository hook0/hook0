<script setup lang="ts">
import { formatISO9075, parseISO } from 'date-fns';
import { computed } from 'vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  value: string;
}
const props = defineProps<Props>();
const valueHumanized = computed(() => formatISO9075(parseISO(props.value)));
</script>

<template>
  <span class="date-time__value" :title="value">{{ valueHumanized }}</span>
</template>

<style scoped>
.date-time__value {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
