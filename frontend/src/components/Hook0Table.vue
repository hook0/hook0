<script setup lang="ts" generic="T">
import {
  useVueTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  FlexRender,
  type ColumnDef,
  type SortingState,
  type CellContext,
} from '@tanstack/vue-table';
import { ref, computed, watch, h, type Component } from 'vue';
import { ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-vue-next';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

// ---------------------------------------------------------------------------
// AG-Grid ColDef → TanStack ColumnDef adapter (temporary, removed in Phase 2)
// ---------------------------------------------------------------------------

interface AgGridColDef {
  field?: string;
  headerName?: string;
  cellRenderer?: Component;
  cellRendererParams?: Record<string, unknown>;
  valueGetter?: (params: { data: unknown }) => unknown;
  valueFormatter?: (params: { value: unknown; data: unknown }) => string;
  sortable?: boolean;
  width?: number;
  minWidth?: number;
  maxWidth?: number;
  suppressMovable?: boolean;
  suppressSizeToFit?: boolean;
  resizable?: boolean;
}

function resolveParams(
  params: Record<string, unknown>,
  row: unknown,
  cellValue: unknown
): Record<string, unknown> {
  const resolved: Record<string, unknown> = {};

  for (const [key, val] of Object.entries(params)) {
    if (key === 'onClick' && typeof val === 'function') {
      resolved[key] = () => (val as (r: unknown) => void)(row);
    } else if (key === 'onChange' && typeof val === 'function') {
      resolved[key] = (v: unknown) => (val as (v: unknown, r: unknown) => void)(v, row);
    } else if (typeof val === 'function') {
      resolved[key] = (val as (r: unknown) => unknown)(row);
    } else {
      resolved[key] = val;
    }
  }

  // Default: use the cell accessor value when no explicit 'value' was provided
  if (!('value' in resolved) && cellValue !== undefined && cellValue !== null) {
    resolved.value = String(cellValue);
  }

  return resolved;
}

function adaptAgGridColDefs(colDefs: AgGridColDef[]): ColumnDef<unknown, unknown>[] {
  return colDefs.map((col, index) => {
    const id = col.field ?? `col-${index}`;
    const header = col.headerName ?? '';
    const enableSorting = col.sortable ?? false;

    // Build accessor
    let accessorKey: string | undefined;
    let accessorFn: ((row: unknown) => unknown) | undefined;

    if (col.valueGetter) {
      const vg = col.valueGetter;
      accessorFn = (row: unknown) => vg({ data: row });
    } else if (col.field) {
      accessorKey = col.field;
    }

    // Build cell renderer
    let cell: ((info: CellContext<unknown, unknown>) => unknown) | undefined;

    if (col.cellRenderer) {
      const renderer = col.cellRenderer;
      const params = col.cellRendererParams ?? {};

      cell = (info: CellContext<unknown, unknown>) => {
        const row = info.row.original;
        const hasAccessor = accessorKey !== undefined || accessorFn !== undefined;
        const cellValue = hasAccessor ? info.getValue() : undefined;
        const resolvedProps = resolveParams(params, row, cellValue);

        // Hook0TableCellSelect expects modelValue, not value
        if (
          'options' in resolvedProps &&
          'value' in resolvedProps &&
          !('modelValue' in resolvedProps)
        ) {
          resolvedProps.modelValue = resolvedProps.value;
          delete resolvedProps.value;
        }

        return h(renderer, resolvedProps);
      };
    } else if (col.valueFormatter) {
      const vf = col.valueFormatter;
      cell = (info: CellContext<unknown, unknown>) => {
        return vf({ value: info.getValue(), data: info.row.original });
      };
    }

    // Assemble ColumnDef
    const result: Record<string, unknown> = { id, header, enableSorting };
    if (accessorKey !== undefined) result.accessorKey = accessorKey;
    if (accessorFn !== undefined) result.accessorFn = accessorFn;
    if (cell !== undefined) result.cell = cell;

    return result as unknown as ColumnDef<unknown, unknown>;
  });
}

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface Props {
  // New TanStack props
  columns?: ColumnDef<T, unknown>[];
  data?: T[];

  // Legacy AG-Grid props (backward compat, removed in Phase 2)
  columnDefs?: AgGridColDef[];
  rowData?: T[];
  context?: unknown;

  // Common
  rowIdField?: string;
  loading?: boolean;
  skeletonRows?: number;
  globalFilter?: string;
  clickableRows?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  columns: undefined,
  data: undefined,
  columnDefs: undefined,
  rowData: undefined,
  context: undefined,
  rowIdField: undefined,
  loading: false,
  skeletonRows: 5,
  globalFilter: '',
  clickableRows: false,
});

const emit = defineEmits<{
  'row-click': [row: T];
}>();

defineOptions({
  inheritAttrs: false,
});

function handleRowClick(row: T) {
  if (props.clickableRows) {
    emit('row-click', row);
  }
}

// ---------------------------------------------------------------------------
// Effective columns / data (prefer new props, fall back to legacy)
// ---------------------------------------------------------------------------

const effectiveColumns = computed<ColumnDef<T, unknown>[]>(() => {
  if (props.columns) return props.columns;
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  if (props.columnDefs)
    return adaptAgGridColDefs(props.columnDefs) as unknown as ColumnDef<T, unknown>[];
  return [];
});

const effectiveData = computed<T[]>(() => {
  if (props.data) return props.data;
  if (props.rowData) return props.rowData;
  return [];
});

// ---------------------------------------------------------------------------
// Table state
// ---------------------------------------------------------------------------

const sorting = ref<SortingState>([]);
const globalFilter = ref(props.globalFilter);

watch(
  () => props.globalFilter,
  (val) => {
    globalFilter.value = val;
  }
);

function getRowId(row: T): string {
  if (props.rowIdField) {
    return String((row as Record<string, unknown>)[props.rowIdField] ?? '');
  }
  return '';
}

const table = useVueTable({
  get data() {
    return effectiveData.value;
  },
  get columns() {
    return effectiveColumns.value;
  },
  state: {
    get sorting() {
      return sorting.value;
    },
    get globalFilter() {
      return globalFilter.value;
    },
  },
  onSortingChange: (updater) => {
    sorting.value = typeof updater === 'function' ? updater(sorting.value) : updater;
  },
  onGlobalFilterChange: (updater: string | ((old: string) => string)) => {
    globalFilter.value = typeof updater === 'function' ? updater(globalFilter.value) : updater;
  },
  getRowId: props.rowIdField ? getRowId : undefined,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  enableSortingRemoval: true,
});

const headerGroups = computed(() => table.getHeaderGroups());
const rows = computed(() => table.getRowModel().rows);
</script>

<template>
  <div class="hook0-table-wrapper" v-bind="$attrs">
    <table class="hook0-table">
      <thead class="hook0-table-head">
        <tr v-for="headerGroup in headerGroups" :key="headerGroup.id">
          <th
            v-for="header in headerGroup.headers"
            :key="header.id"
            class="hook0-table-th"
            :class="{ sortable: header.column.getCanSort() }"
            :aria-sort="
              header.column.getIsSorted() === 'asc'
                ? 'ascending'
                : header.column.getIsSorted() === 'desc'
                  ? 'descending'
                  : 'none'
            "
            @click="header.column.getToggleSortingHandler()?.($event)"
          >
            <div class="hook0-table-th-content">
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
              <span v-if="header.column.getCanSort()" class="hook0-table-sort-icon">
                <ArrowUp
                  v-if="header.column.getIsSorted() === 'asc'"
                  :size="14"
                  aria-hidden="true"
                />
                <ArrowDown
                  v-else-if="header.column.getIsSorted() === 'desc'"
                  :size="14"
                  aria-hidden="true"
                />
                <ArrowUpDown v-else :size="14" aria-hidden="true" class="hook0-table-sort-idle" />
              </span>
            </div>
          </th>
        </tr>
      </thead>

      <tbody class="hook0-table-body">
        <!-- Skeleton loading rows -->
        <template v-if="loading">
          <tr v-for="i in skeletonRows" :key="`skeleton-${i}`" class="hook0-table-tr">
            <td
              v-for="header in headerGroups[0]?.headers ?? []"
              :key="`skeleton-${i}-${header.id}`"
              class="hook0-table-td"
            >
              <Hook0Skeleton size="text-truncated" />
            </td>
          </tr>
        </template>

        <!-- Data rows -->
        <template v-else>
          <tr
            v-for="row in rows"
            :key="row.id"
            :row-id="row.id"
            class="hook0-table-tr"
            :class="{ 'hook0-table-tr--clickable': clickableRows }"
            @click="handleRowClick(row.original)"
          >
            <td v-for="cell in row.getVisibleCells()" :key="cell.id" class="hook0-table-td">
              <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
            </td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.hook0-table-wrapper {
  overflow-x: auto;
  border-radius: var(--radius-md);
}

.hook0-table {
  width: 100%;
  border-collapse: collapse;
}

.hook0-table-head {
  background-color: var(--color-bg-secondary);
}

.hook0-table-th {
  padding: 0.75rem 1rem;
  text-align: left;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  white-space: nowrap;
  border-bottom: 1px solid var(--color-border);
}

.hook0-table-th.sortable {
  cursor: pointer;
  user-select: none;
}

.hook0-table-th.sortable:hover {
  color: var(--color-text-primary);
}

.hook0-table-th-content {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.hook0-table-sort-icon {
  display: inline-flex;
  align-items: center;
}

.hook0-table-sort-idle {
  opacity: 0.3;
}

.hook0-table-body {
  background-color: var(--color-bg-primary);
}

.hook0-table-tr {
  transition: background-color 0.1s ease;
}

.hook0-table-tr:hover {
  background-color: var(--color-bg-secondary);
}

.hook0-table-tr--clickable {
  cursor: pointer;
}

.hook0-table-td {
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  color: var(--color-text-primary);
  border-bottom: 1px solid var(--color-border);
  vertical-align: middle;
}

.hook0-table-tr:last-child .hook0-table-td {
  border-bottom: none;
}

@media (max-width: 767px) {
  .hook0-table th:first-child,
  .hook0-table td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background-color: var(--color-bg-primary);
  }
}
</style>
