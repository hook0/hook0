<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { CheckCircle, AlertTriangle, XCircle, Minus } from 'lucide-vue-next';
import Hook0Badge from './Hook0Badge.vue';
import Hook0Tooltip from './Hook0Tooltip.vue';
import { WARNING_THRESHOLD, CRITICAL_THRESHOLD } from '@/constants/healthThresholds';

const props = defineProps<{
  failurePercent: number | null;
}>();

const { t } = useI18n();

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

// Each status maps to a distinct icon so users can distinguish states without relying on color alone (accessibility)
const statusIcon = computed(() => {
  switch (status.value) {
    case 'healthy':
      return CheckCircle;
    case 'warning':
      return AlertTriangle;
    case 'critical':
      return XCircle;
    default:
      return Minus;
  }
});

const label = computed(() => t(`health.${status.value}`));

const tooltipContent = computed(() => {
  if (props.failurePercent === null) return t('health.awaitingData');
  return t('health.failureRateTooltip', { percent: Math.round(props.failurePercent) });
});
</script>

<template>
  <Hook0Tooltip :content="tooltipContent" position="top">
    <Hook0Badge :variant="variant" size="sm">
      <component v-if="status !== 'noData'" :is="statusIcon" :size="12" aria-hidden="true" />
      {{ label }}
    </Hook0Badge>
  </Hook0Tooltip>
</template>
