<template>
  <hook0-text class="code" style="width: fit-content"
    >{{
      params.colDef.cellRendererParams && params.colDef.cellRendererParams.value
        ? typeof params.colDef.cellRendererParams.value === 'function'
          ? params.colDef.cellRendererParams.value(params.data)
          : params.colDef.cellRendererParams.value
        : params.value
    }}
  </hook0-text>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';
import { ICellRendererParams } from '@ag-grid-community/core';
import Hook0Text from '@/components/Hook0Text.vue';

@Options({
  name: 'hook0-table-cell-code',
  inheritAttrs: false,
  components: { Hook0Text },
})
export default class Hook0TableCellCode<T> extends Vue {
  private params!: ICellRendererParams;
}
</script>
<style>
.ag-theme-alpine .ag-cell {
  line-height: 40px;
}
</style>
