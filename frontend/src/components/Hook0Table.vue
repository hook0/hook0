<template>
  <ag-grid-vue
    class="ag-theme-alpine"
    v-bind="{ ...$props, ...$attrs }"
    :domLayout="domLayout"
    :modules="modules.concat(contextModule ? [contextModule] : [])"
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
import { AgGridEvent, ColDef, ICellRendererParams, Module, RowNode } from '@ag-grid-community/core';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0TableCellLinks from '@/components/Hook0TableCellLinks.vue';
import Hook0TableCellIcon from '@/components/Hook0TableCellIcon.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import { defineComponent } from 'vue';

type Context = Record<string, any>;

export default defineComponent({
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
  data() {
    return {
      domLayout: null as string | null,
      modules: [ClientSideRowModelModule],

      defaultColDef: {
        resizable: false,
      },
      gridOptions: {
        enableCellTextSelection: true,
      },
    };
  },
  created() {
    this.domLayout = 'autoHeight';
  },
  computed: {
    contextModule(): null | (Module & { context: Context }) {
      return this.context
        ? {
            moduleName: 'context',
            version: ClientSideRowModelModule.version,
            context: this.context as Context,
          }
        : null;
    },
  },
  components: {
    AgGridVue,
    // eslint-disable-next-line vue/no-unused-components
    Hook0TableCellLink,
    // eslint-disable-next-line vue/no-unused-components
    Hook0TableCellLinks,
    // eslint-disable-next-line vue/no-unused-components
    Hook0TableCellCode,
    // eslint-disable-next-line vue/no-unused-components
    Hook0TableCellIcon,
    // eslint-disable-next-line vue/no-unused-components
    Hook0TableCellDate,
  },
  methods: {
    onFirstDataRendered(params: AgGridEvent<any>) {
      params.api.sizeColumnsToFit();
    },
  },
});
</script>
<!-- Add "scoped" attribute to limit CSS to this component only -->
<style>
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
