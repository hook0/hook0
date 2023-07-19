<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div v-bind="{ ...$attrs }" class="px-4 py-2 text-sm" role="none">
    <slot></slot>
  </div>
</template>
