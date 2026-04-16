<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { CheckCircle, AlertTriangle, XCircle, Minus } from 'lucide-vue-next';
import Hook0Badge from './Hook0Badge.vue';
import Hook0Tooltip from './Hook0Tooltip.vue';
import { useHealthThresholds } from '@/composables/useHealthThresholds';

const props = defineProps<{
  failurePercent: number | null;
}>();

const { t } = useI18n();
const { warning: warningThreshold, critical: criticalThreshold } = useHealthThresholds();

type HealthLevel = 'healthy' | 'warning' | 'critical' | 'noData';
type BadgeVariant = 'default' | 'success' | 'warning' | 'danger';

const level = computed<HealthLevel>(() => {
  if (props.failurePercent === null) return 'noData';
  if (props.failurePercent >= criticalThreshold.value) return 'critical';
  if (props.failurePercent >= warningThreshold.value) return 'warning';
  return 'healthy';
});

const variant = computed<BadgeVariant>(() => {
  const map: Record<HealthLevel, BadgeVariant> = {
    healthy: 'success',
    warning: 'warning',
    critical: 'danger',
    noData: 'default',
  };
  return map[level.value];
});

const iconComponent = computed(() => {
  const map = { healthy: CheckCircle, warning: AlertTriangle, critical: XCircle, noData: Minus };
  return map[level.value];
});

const label = computed(() => t(`health.${level.value}`));

const tooltipContent = computed(() => {
  if (props.failurePercent === null) return t('health.awaitingData');
  return t('health.failureRateTooltip', { percent: Math.round(props.failurePercent) });
});
</script>

<template>
  <Hook0Tooltip :content="tooltipContent" position="top">
    <Hook0Badge :variant="variant" size="sm">
      <component
        v-if="level !== 'noData'"
        :is="iconComponent"
        :size="12"
        aria-hidden="true"
      />
      {{ label }}
    </Hook0Badge>
  </Hook0Tooltip>
</template>
