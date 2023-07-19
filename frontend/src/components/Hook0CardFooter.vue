<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-footer">
    <slot></slot>
  </div>
</template>

<style lang="scss" scoped>
.hook0-card-footer {
  @apply mt-6 flex items-center justify-end space-x-4 px-4 py-5 sm:p-6;
}
</style>
