<script setup lang="ts">
type Props = {
  size?: number;
};

withDefaults(defineProps<Props>(), {
  size: 16,
});
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <path d="M24 24H0V0h24L12 12Z" />
  </svg>
</template>
