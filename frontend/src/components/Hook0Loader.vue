<script setup lang="ts">
import Hook0Spinner from '@/components/Hook0Spinner.vue';
</script>

<template>
  <div class="hook0-loader">
    <Hook0Spinner :size="32" />
  </div>
</template>

<style scoped>
.hook0-loader {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 2rem;
  color: var(--color-text-muted);
}
</style>
