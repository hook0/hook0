<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="py-1">
    <slot></slot>
  </div>
</template>
