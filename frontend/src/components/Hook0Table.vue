<script setup lang="ts">
import { AgGridVue } from 'ag-grid-vue3';
import {
  ClientSideRowModelModule,
  AgGridEvent,
  ColDef,
  Module,
  ModuleRegistry,
  ColumnAutoSizeModule,
  ValidationModule,
  themeAlpine,
  GetRowIdParams,
} from 'ag-grid-community';
import { computed } from 'vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  columnDefs: ColDef[];
  rowData: unknown[];
  context: object;
  rowIdField?: string;
}

const props = defineProps<Props>();

// Create getRowId callback if rowIdField is provided
const getRowId = props.rowIdField
  ? (params: GetRowIdParams) => {
      const row = params.data as Record<string, unknown>;
      return String(row[props.rowIdField!] ?? '');
    }
  : undefined;
const domLayout = 'autoHeight';
const modules = [ClientSideRowModelModule];
ModuleRegistry.registerModules([ValidationModule, ColumnAutoSizeModule]);

const defaultColDef = {
  resizable: false,
};
const gridOptions = {
  enableCellTextSelection: true,
};

type Context = Record<string, unknown>;
const contextModule = computed<null | (Module & { context: Context })>(() => {
  return props.context
    ? ({
        moduleName: 'context',
        version: ClientSideRowModelModule.version,
        context: props.context as Context,
      } as unknown as Module & { context: Context })
    : null;
});

function onFirstDataRendered(params: AgGridEvent<unknown>) {
  params.api.sizeColumnsToFit();
}
</script>

<template>
  <AgGridVue
    class="ag-theme-alpine"
    v-bind="{ ...$props, ...$attrs }"
    :dom-layout="domLayout"
    :modules="modules.concat(contextModule ? [contextModule] : [])"
    :default-col-def="defaultColDef"
    :suppress-row-hover-highlight="true"
    :suppress-horizontal-scroll="true"
    :suppress-cell-focus="true"
    :grid-options="gridOptions"
    :theme="themeAlpine"
    :get-row-id="getRowId"
    @first-data-rendered="onFirstDataRendered"
  >
  </AgGridVue>
</template>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss">
.ag-theme-alpine .ag-root-wrapper {
  border: 0;
}

.ag-center-cols-clipper {
  min-height: unset !important;
}

.ag-theme-alpine .ag-header {
  @apply bg-gray-50;
}

.ag-theme-alpine .ag-header-row {
  @apply text-left text-xs font-medium text-gray-500 tracking-wider;
}

.ag-theme-alpine .ag-ltr .ag-has-focus .ag-cell-focus {
  border: 0px;
}

.ag-theme-alpine .ag-ltr .ag-has-focus .ag-cell-focus:not(.ag-cell-range-selected) {
  border-color: transparent;
}

.ag-theme-alpine .ag-cell,
.ag-theme-alpine .ag-full-width-row .ag-cell-wrapper.ag-row-group {
  line-height: 40px;
}
</style>
