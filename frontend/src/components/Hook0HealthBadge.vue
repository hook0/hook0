<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Hook0Badge from './Hook0Badge.vue';

const props = defineProps<{
  failurePercent: number | null | undefined;
}>();

const { t } = useI18n();

// Thresholds aligned with health monitor defaults:
// --health-monitor-warning-failure-percent=80 and --health-monitor-disable-failure-percent=95
const status = computed(() => {
  if (props.failurePercent == null) return 'noData';
  if (props.failurePercent >= 95) return 'disabled';
  if (props.failurePercent >= 80) return 'warning';
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

const label = computed(() => {
  switch (status.value) {
    case 'healthy':
      return t('health.healthy');
    case 'warning':
      return t('health.warning');
    case 'disabled':
      return t('health.disabled');
    default:
      return t('health.noData');
  }
});

const percentLabel = computed(() => {
  if (props.failurePercent == null) return '';
  return `${Math.round(props.failurePercent)}%`;
});
</script>

<template>
  <Hook0Badge :variant="variant" size="sm">
    {{ label }}
    <template v-if="percentLabel"> ({{ percentLabel }})</template>
  </Hook0Badge>
</template>
