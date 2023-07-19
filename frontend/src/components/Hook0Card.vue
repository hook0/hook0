<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="card">
    <slot></slot>
  </div>
</template>

<style lang="scss" scoped>
.card {
  @apply bg-white shadow overflow-hidden sm:rounded-lg mb-4;
}
</style>
