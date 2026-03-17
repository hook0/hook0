<script setup lang="ts">
import { computed } from 'vue';

type Props = {
  percentage: number;
}

const props = defineProps<Props>();

const clampedPercentage = computed(() => (props.percentage >= 100 ? 100 : props.percentage));

const barVariant = computed(() => {
  if (props.percentage <= 0) return 'empty';
  if (props.percentage >= 100) return 'critical';
  if (props.percentage >= 80) return 'warning';
  return 'normal';
});

const barStyle = computed(() => ({
  width: `${clampedPercentage.value}%`,
}));
</script>

<template>
  <div
    class="hook0-progress-bar"
    role="progressbar"
    :aria-valuenow="clampedPercentage"
    aria-valuemin="0"
    aria-valuemax="100"
  >
    <div
      class="hook0-progress-bar__fill"
      :class="[
        `hook0-progress-bar__fill--${barVariant}`,
        { 'hook0-progress-bar__fill--low': percentage < 3 },
      ]"
      :style="barStyle"
    >
      {{ clampedPercentage }}%
    </div>
  </div>
</template>

<style scoped>
.hook0-progress-bar {
  width: 100%;
  background-color: var(--color-bg-tertiary);
  border-radius: var(--radius-lg);
}

.hook0-progress-bar__fill {
  font-size: 0.875rem;
  text-align: center;
  padding: 0.25rem;
  line-height: 1;
  border-radius: var(--radius-lg);
  font-weight: 800;
  color: var(--color-primary-text);
}

.hook0-progress-bar__fill--low {
  min-width: 2.5rem;
}

.hook0-progress-bar__fill--empty {
  background-color: transparent;
  color: var(--color-text-secondary);
}

.hook0-progress-bar__fill--normal {
  background-color: var(--color-primary);
}

.hook0-progress-bar__fill--warning {
  background-color: var(--color-warning);
}

.hook0-progress-bar__fill--critical {
  background-color: var(--color-error);
}
</style>
