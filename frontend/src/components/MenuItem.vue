<script setup lang="ts">
import Hook0Button from '@/components/Hook0Button.vue';

interface Props {
  name: string;
  active: boolean;
}

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
  color: #ffffff;
  display: flex;
  align-items: center;
  padding: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.375rem;
  cursor: pointer;
}

.active {
  background-color: #111827;
}

.inactive:hover {
  color: #ffffff;
  background-color: #374151;
}
</style>
