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

<style lang="scss" scoped>
.default {
  @apply text-gray-300 text-white flex items-center px-2 py-2 text-sm font-medium rounded-md cursor-pointer;
}

.active {
  @apply bg-gray-900;
}

.inactive {
  @apply hover:text-white hover:bg-gray-700;
}
</style>
