<template>
  <hook0-icon
    :title="
      typeof params.colDef.cellRendererParams.title === 'function'
        ? params.colDef.cellRendererParams.title(params.data)
        : params.colDef.cellRendererParams.title
    "
    :name="
      typeof params.colDef.cellRendererParams.icon === 'function'
        ? params.colDef.cellRendererParams.icon(params.data)
        : params.colDef.cellRendererParams.icon
    "
  ></hook0-icon>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';
import Hook0Icon from '@/components/Hook0Icon.vue';
import { ICellRendererParams } from '@ag-grid-community/core';
import { RouteLocation } from 'vue-router';

interface ExtraParams<T> {
  /**
   * Icon to add to the icon
   */
  icon: string;

  /**
   * Tooltip to add to the icon
   */
  title: string;
}

type Hook0TableCellIconParameter<T> = ICellRendererParams & ExtraParams<T>;

@Options({
  name: 'hook0-table-cell-icon',
  inheritAttrs: false,
  props: {
    params: {
      type: Object,
      required: true,
    },
  },
  components: { Hook0Icon },
})
export default class Hook0TableCellIcon<T> extends Vue {
  private params!: Hook0TableCellIconParameter<T>;
}
</script>
<style></style>
