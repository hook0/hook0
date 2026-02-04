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

<style scoped>
.list {
  border: 1px solid #e5e7eb;
  border-radius: 0.375rem;
}

.list :deep(> * + *) {
  border-top: 1px solid #e5e7eb;
}
</style>
