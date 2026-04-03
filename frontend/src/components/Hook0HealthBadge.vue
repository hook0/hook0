<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Hook0Badge from './Hook0Badge.vue';
import Hook0Tooltip from './Hook0Tooltip.vue';

const props = defineProps<{
  failurePercent: number | null;
}>();

const { t } = useI18n();

// Thresholds must match health_monitor defaults in api/src/main.rs
const WARNING_THRESHOLD = 80;
const CRITICAL_THRESHOLD = 95;

const variant = computed(() => {
  if (props.failurePercent === null) return 'default';
  if (props.failurePercent >= CRITICAL_THRESHOLD) return 'danger';
  if (props.failurePercent >= WARNING_THRESHOLD) return 'warning';
  return 'success';
});

const status = computed(() => {
  if (props.failurePercent === null) return 'noData';
  if (props.failurePercent >= CRITICAL_THRESHOLD) return 'critical';
  if (props.failurePercent >= WARNING_THRESHOLD) return 'warning';
  return 'healthy';
});

const label = computed(() => t(`health.${status.value}`));

const tooltipContent = computed(() => {
  if (props.failurePercent === null) return t('health.awaitingData');
  return Math.round(props.failurePercent) + '% ' + t('health.failureRate');
});
</script>

<template>
  <Hook0Tooltip :content="tooltipContent" position="top">
    <Hook0Badge :variant="variant" size="sm">
      {{ label }}
    </Hook0Badge>
  </Hook0Tooltip>
</template>
