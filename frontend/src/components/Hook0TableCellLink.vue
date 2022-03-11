<template>
  <hook0-button
    :to="params.colDef.cellRendererParams && params.colDef.cellRendererParams.href ? params.colDef.cellRendererParams.href(params.data) : undefined"
  >
    {{ params.value }}
  </hook0-button>
</template>


<script lang="ts">
import {Vue, Options} from 'vue-class-component';
import {ICellRendererParams} from "@ag-grid-community/core";

@Options({
  name: 'hook0-table-cell-link',
  inheritAttrs: false,
  props: {},
  components: {}
})
export default class Hook0Table extends Vue {
  private params!: ICellRendererParams;
};
</script>
<style>
</style>

