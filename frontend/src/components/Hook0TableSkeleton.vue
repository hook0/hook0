<script setup lang="ts">
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

interface Props {
  rows?: number;
  columns?: number;
}

withDefaults(defineProps<Props>(), {
  rows: 3,
  columns: 3,
});
</script>

<template>
  <div class="hook0-table-skeleton" aria-hidden="true">
    <!-- Header row -->
    <div class="hook0-table-skeleton-row hook0-table-skeleton-header">
      <Hook0Skeleton v-for="col in columns" :key="`header-${col}`" size="text-sm" />
    </div>
    <!-- Data rows -->
    <div v-for="row in rows" :key="`row-${row}`" class="hook0-table-skeleton-row">
      <Hook0Skeleton v-for="col in columns" :key="`cell-${row}-${col}`" size="text-sm" />
    </div>
  </div>
</template>

<style scoped>
.hook0-table-skeleton {
  display: flex;
  flex-direction: column;
  gap: 0;
  width: 100%;
}

.hook0-table-skeleton-row {
  display: grid;
  grid-template-columns: repeat(v-bind(columns), 1fr);
  gap: 1rem;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-table-skeleton-header {
  padding-bottom: 0.75rem;
}

.hook0-table-skeleton-row:last-child {
  border-bottom: none;
}
</style>
