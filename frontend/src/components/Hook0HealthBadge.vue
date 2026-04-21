<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { CheckCircle, AlertTriangle, XCircle } from 'lucide-vue-next';
import Hook0Badge, { type BadgeVariant } from './Hook0Badge.vue';
import Hook0Tooltip from './Hook0Tooltip.vue';
import { useHealthThresholds } from '@/composables/useHealthThresholds';

const props = defineProps<{
  failurePercent: number | null;
}>();

const { t } = useI18n();
const { warning: warningThreshold, critical: criticalThreshold } = useHealthThresholds();

type HealthLevel = 'healthy' | 'warning' | 'critical' | 'noData';

const level = computed<HealthLevel>(() => {
  if (props.failurePercent === null) return 'noData';
  if (props.failurePercent >= criticalThreshold.value) return 'critical';
  if (props.failurePercent >= warningThreshold.value) return 'warning';
  return 'healthy';
});

const variant = computed<BadgeVariant>(() => {
  switch (level.value) {
    case 'healthy':
      return 'success';
    case 'warning':
      return 'warning';
    case 'critical':
      return 'danger';
    case 'noData':
    default:
      return 'default';
  }
});

const iconComponent = computed(() => {
  switch (level.value) {
    case 'healthy':
      return CheckCircle;
    case 'warning':
      return AlertTriangle;
    case 'critical':
      return XCircle;
    case 'noData':
    default:
      return null;
  }
});

const label = computed(() => t(`health.${level.value}`));

const tooltipContent = computed(() => {
  if (props.failurePercent === null) {
    return t('health.awaitingData');
  }
  // Show "<1%" when rate is non-zero but rounds to 0, else 1 decimal
  const rounded =
    props.failurePercent > 0 && props.failurePercent < 0.1 ? '<1' : props.failurePercent.toFixed(1);
  return t('health.failureRateTooltip', { percent: rounded });
});
</script>

<template>
  <Hook0Tooltip :content="tooltipContent" position="top">
    <Hook0Badge :variant="variant" size="sm">
      <span v-if="level !== 'noData'" class="health-badge__icon">
        <component :is="iconComponent" :size="12" aria-hidden="true" />
      </span>
      {{ label }}
    </Hook0Badge>
  </Hook0Tooltip>
</template>

<style scoped>
.health-badge__icon {
  display: inline-flex;
  margin-right: 0.25rem;
  vertical-align: middle;
}
</style>
