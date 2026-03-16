<script setup lang="ts">
/**
 * Hook0TableSkeleton - Loading skeleton that mimics a table layout
 *
 * Renders a fake table with shimmer rows to indicate data is loading.
 * Header cells use wider blocks; body cells use varying widths for realism.
 */
import { useI18n } from 'vue-i18n';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

interface Props {
  /** Number of columns to display */
  columns?: number;
  /** Number of body rows to display */
  rows?: number;
}

withDefaults(defineProps<Props>(), {
  columns: 4,
  rows: 5,
});

const { t } = useI18n();

/**
 * Returns a deterministic width percentage for a body cell skeleton,
 * producing visual variation across columns and rows.
 */
function cellWidth(rowIndex: number, colIndex: number): string {
  const widths = ['45%', '60%', '75%', '55%', '65%', '50%', '70%', '40%'];
  const index = (rowIndex * 3 + colIndex * 7) % widths.length;
  return widths[index];
}
</script>

<template>
  <div
    class="hook0-table-skeleton"
    role="status"
    aria-busy="true"
    :aria-label="t('common.loadingTable')"
  >
    <table class="hook0-table-skeleton__table">
      <thead class="hook0-table-skeleton__head">
        <tr>
          <th v-for="col in columns" :key="`header-${col}`" class="hook0-table-skeleton__th">
            <Hook0Skeleton size="text-sm" />
          </th>
        </tr>
      </thead>
      <tbody class="hook0-table-skeleton__body">
        <tr v-for="row in rows" :key="`row-${row}`" class="hook0-table-skeleton__tr">
          <td v-for="col in columns" :key="`cell-${row}-${col}`" class="hook0-table-skeleton__td">
            <div :style="{ width: cellWidth(row, col) }">
              <Hook0Skeleton size="text-sm" />
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.hook0-table-skeleton {
  overflow-x: auto;
  border-radius: var(--radius-md);
}

.hook0-table-skeleton__table {
  width: 100%;
  border-collapse: collapse;
}

.hook0-table-skeleton__head {
  background-color: var(--color-bg-secondary);
}

.hook0-table-skeleton__th {
  padding: 0.75rem 1rem;
  text-align: left;
  border-bottom: 1px solid var(--color-border);
}

.hook0-table-skeleton__body {
  background-color: var(--color-bg-primary);
}

.hook0-table-skeleton__tr {
  transition: background-color 0.1s ease;
}

.hook0-table-skeleton__td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border);
  vertical-align: middle;
}

.hook0-table-skeleton__tr:last-child .hook0-table-skeleton__td {
  border-bottom: none;
}
</style>
