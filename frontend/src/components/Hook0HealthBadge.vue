<script setup lang="ts">
// Subscription health status badge — maps a failure_percent (0-100, null if no data) to colored badges: healthy, warning, disabled, noData.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Hook0Badge from './Hook0Badge.vue';

const props = defineProps<{
  failurePercent: number | null;
}>();

const { t } = useI18n();

// These thresholds must match health_monitor_warning_failure_percent (default 80) and health_monitor_disable_failure_percent (default 95) in api/src/main.rs — if the server values change, this badge shows incorrect status labels.
const WARNING_THRESHOLD = 80;
const DISABLE_THRESHOLD = 95;

// Evaluated top-down from worst to best: null → disabled → warning → healthy. Order matters — reversing would misclassify.
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
