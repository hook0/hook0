<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <dl class="hook0-card-content-lines">
    <slot></slot>
  </dl>
</template>

<style scoped>
@media (min-width: 640px) {
  .hook0-card-content-lines :deep(> * + *) {
    border-top: 1px solid #e5e7eb;
  }
}
</style>
