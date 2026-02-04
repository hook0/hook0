<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  text: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
}

withDefaults(defineProps<Props>(), {
  position: 'top',
});

const visible = ref(false);
</script>

<template>
  <span
    class="hook0-tooltip-wrapper"
    @mouseenter="visible = true"
    @mouseleave="visible = false"
    @focusin="visible = true"
    @focusout="visible = false"
  >
    <slot />
    <span
      v-if="visible"
      class="hook0-tooltip"
      :class="[`hook0-tooltip-${position}`]"
      role="tooltip"
    >
      {{ text }}
    </span>
  </span>
</template>

<style scoped>
.hook0-tooltip-wrapper {
  position: relative;
  display: inline-flex;
}

.hook0-tooltip {
  position: absolute;
  z-index: 50;
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-primary-text);
  background-color: var(--color-text-primary);
  border-radius: var(--radius-md);
  white-space: nowrap;
  pointer-events: none;
  box-shadow: var(--shadow-md);
}

.hook0-tooltip-top {
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
}

.hook0-tooltip-bottom {
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
}

.hook0-tooltip-left {
  right: calc(100% + 6px);
  top: 50%;
  transform: translateY(-50%);
}

.hook0-tooltip-right {
  left: calc(100% + 6px);
  top: 50%;
  transform: translateY(-50%);
}
</style>
