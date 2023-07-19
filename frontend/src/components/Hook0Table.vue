<script setup lang="ts">
import { AgGridVue } from '@ag-grid-community/vue3';
import { ClientSideRowModelModule } from '@ag-grid-community/client-side-row-model';
import { AgGridEvent, ColDef, Module } from '@ag-grid-community/core';
import { computed } from 'vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  columnDefs: ColDef[];
  rowData: unknown[];
  context: object;
}

const props = defineProps<Props>();
const domLayout = 'autoHeight';
const modules = [ClientSideRowModelModule];

const defaultColDef = {
  resizable: false,
};
const gridOptions = {
  enableCellTextSelection: true,
};

type Context = Record<string, unknown>;
const contextModule = computed<null | (Module & { context: Context })>(() => {
  return props.context
    ? {
        moduleName: 'context',
        version: ClientSideRowModelModule.version,
        context: props.context as Context,
      }
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
    @first-data-rendered="onFirstDataRendered"
  >
  </AgGridVue>
</template>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss">
@import '@ag-grid-community/styles/ag-grid.css';
@import '@ag-grid-community/styles/ag-theme-alpine.css';

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
