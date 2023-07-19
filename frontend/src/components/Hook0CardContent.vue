<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="card-content">
    <slot></slot>
  </div>
</template>

<style lang="scss" scoped>
.card-content {
  @apply border-t border-gray-200 px-4 py-5 sm:p-0;
}
</style>
