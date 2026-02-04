<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-content">
    <slot />
  </div>
</template>

<style scoped>
.hook0-card-content {
  padding: 1.25rem 1.5rem;
  border-top: 1px solid var(--color-border);
}

.hook0-card-content:first-child {
  border-top: none;
}
</style>
