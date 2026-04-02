<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Hook0Badge from './Hook0Badge.vue';

const props = defineProps<{
  failurePercent: number | null;
}>();

const { t } = useI18n();

// Thresholds aligned with health monitor defaults:
// --health-monitor-warning-failure-percent=80 and --health-monitor-disable-failure-percent=95
// COUPLING: If backend thresholds are changed via CLI args, these must be updated to match.
// Server-side config: api/src/main.rs --health-monitor-warning-failure-percent and --health-monitor-disable-failure-percent
// TODO: Expose active thresholds via instance config API to eliminate this coupling.
const WARNING_THRESHOLD = 80;
const DISABLE_THRESHOLD = 95;

const status = computed(() => {
  if (props.failurePercent === null) {
    return 'noData';
  }
  if (props.failurePercent >= DISABLE_THRESHOLD) {
    return 'disabled';
  }
  if (props.failurePercent >= WARNING_THRESHOLD) {
    return 'warning';
  }
  return 'healthy';
});

const variant = computed(() => {
  switch (status.value) {
    case 'healthy':
      return 'success';
    case 'warning':
      return 'warning';
    case 'disabled':
      return 'danger';
    default:
      return 'default';
  }
});

const label = computed(() => t(`health.${status.value}`));

const percentLabel = computed(() => {
  if (props.failurePercent === null) {
    return '';
  }
  return `${Math.round(props.failurePercent)}%`;
});
</script>

<template>
  <Hook0Badge :variant="variant" size="sm">
    {{ label }}
    <template v-if="percentLabel"> ({{ percentLabel }})</template>
  </Hook0Badge>
</template>
