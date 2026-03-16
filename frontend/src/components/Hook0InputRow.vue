<script setup lang="ts">
type Gap = 'sm' | 'md' | 'lg';

interface Props {
  gap?: Gap;
  columns?: number;
}

withDefaults(defineProps<Props>(), {
  gap: 'md',
  columns: 2,
});

defineSlots<{
  default(): unknown;
}>();

const gapMap: Record<Gap, string> = {
  sm: '0.5rem',
  md: '1rem',
  lg: '1.5rem',
};
</script>

<template>
  <div
    class="hook0-input-row"
    :style="{
      '--input-row-gap': gapMap[gap],
      '--input-row-columns': columns,
    }"
  >
    <slot />
  </div>
</template>

<style scoped>
.hook0-input-row {
  display: flex;
  gap: var(--input-row-gap);
}

.hook0-input-row > :deep(*) {
  flex: 1;
  min-width: 6rem;
}

.hook0-input-row > :deep(.hook0-input-row__separator) {
  flex: 0;
  min-width: auto;
}

@media (max-width: 640px) {
  .hook0-input-row {
    flex-direction: column;
  }
}
</style>
