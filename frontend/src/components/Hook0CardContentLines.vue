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

<style lang="scss" scoped>
.hook0-card-content-lines {
  @apply sm:divide-y sm:divide-gray-200;
}
</style>
