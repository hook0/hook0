<script setup lang="ts">
import { useIsFetching } from '@tanstack/vue-query';

const isFetching = useIsFetching();
</script>

<template>
  <div v-if="isFetching.value > 0" class="refetch-bar" />
</template>

<style scoped>
.refetch-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  z-index: 100;
  background: var(--color-primary);
  animation: refetch-progress 2s ease-in-out infinite;
}

@keyframes refetch-progress {
  0% {
    transform: translateX(-100%);
  }
  50% {
    transform: translateX(0%);
  }
  100% {
    transform: translateX(100%);
  }
}

@media (prefers-reduced-motion: reduce) {
  .refetch-bar {
    animation: none;
  }
}
</style>
