<script setup lang="ts">
import Hook0Button from '@/components/Hook0Button.vue';

type Props = {
  name: string;
  active: boolean;
};

defineProps<Props>();
</script>

<template>
  <Hook0Button class="default" :class="{ active: active, inactive: !active }">
    <slot></slot>
    {{ name }}
  </Hook0Button>
</template>

<style scoped>
.default {
  color: var(--color-primary-text);
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.5rem;
  padding: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.375rem;
  cursor: pointer;
}

.default :deep(svg) {
  flex-shrink: 0;
}

.active {
  background-color: var(--color-text-primary);
}

.inactive:hover {
  color: var(--color-primary-text);
  background-color: var(--color-text-secondary);
}
</style>
