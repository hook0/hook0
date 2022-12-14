<template>
  <ag-grid-vue
    class="ag-theme-alpine"
    v-bind="{ ...$props, ...$attrs }"
    :domLayout="domLayout"
    :modules="modules.concat(context ? [{ moduleName: 'context', context: context }] : [])"
    :defaultColDef="defaultColDef"
    :suppressRowHoverHighlight="true"
    :suppressHorizontalScroll="true"
    :suppressCellFocus="true"
    :gridOptions="gridOptions"
    @first-data-rendered="onFirstDataRendered"
  >
  </ag-grid-vue>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';
import { AgGridVue } from '@ag-grid-community/vue3';
import { ClientSideRowModelModule } from '@ag-grid-community/client-side-row-model';
import { AgGridEvent, ColDef, ICellRendererParams, RowNode } from '@ag-grid-community/core';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0TableCellLinks from '@/components/Hook0TableCellLinks.vue';
import Hook0TableCellIcon from '@/components/Hook0TableCellIcon.vue';

@Options({
  name: 'hook0-table',
  inheritAttrs: false,
  props: {
    columnDefs: {
      type: Array,
      required: true,
    },
    rowData: {
      required: true,
    },

    /**
     * The Vue component context (this) that will be available to the callbacks as execution context
     */
    context: {
      type: Object,
      required: true,
    },
  },
  components: {
    AgGridVue,
    Hook0TableCellLink,
    Hook0TableCellLinks,
    Hook0TableCellCode,
    Hook0TableCellIcon,
  },
})
export default class Hook0Table extends Vue {
  private domLayout: string | null = null;
  private modules = [ClientSideRowModelModule];
  private defaultColDef = {
    resizable: false,
  };

  private gridOptions = {};

  created() {
    this.domLayout = 'autoHeight';
  }

  onFirstDataRendered(params: AgGridEvent<any>) {
    params.api.sizeColumnsToFit();
  }
}
</script>
<!-- Add "scoped" attribute to limit CSS to this component only -->
<style>
@import '@ag-grid-community/core/dist/styles/ag-grid.css';
@import '@ag-grid-community/core/dist/styles/ag-theme-alpine.css';

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
