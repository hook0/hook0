<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <ul class="list" v-bind="{ ...$attrs }">
    <slot></slot>
  </ul>
</template>

<style lang="scss" scoped>
.list {
  @apply border border-gray-200 rounded-md divide-y divide-gray-200;
}
</style>
