<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
});

type Props = {
  value: string;
}

defineProps<Props>();
</script>

<template>
  <code class="table-cell-code__value">{{ value }}</code>
</template>

<style scoped>
.table-cell-code__value {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.5;
  width: fit-content;
  white-space: nowrap;
  user-select: text;
}
</style>
