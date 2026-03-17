<script setup lang="ts">
import { Loader2 } from 'lucide-vue-next';

type Props = {
  size?: number;
}

withDefaults(defineProps<Props>(), {
  size: 20,
});
</script>

<template>
  <Loader2 class="hook0-spinner" :size="size" aria-hidden="true" />
</template>

<style scoped>
.hook0-spinner {
  animation: spin 0.8s linear infinite;
  color: currentColor;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
