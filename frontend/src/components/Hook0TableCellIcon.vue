<script setup lang="ts">
import { ICellRendererParams } from 'ag-grid-community';

import Hook0Icon from '@/components/Hook0Icon.vue';

defineOptions({
  inheritAttrs: false,
});

interface ExtraParams {
  /**
   * Icon to add to the icon
   */
  icon: string;

  /**
   * Tooltip to add to the icon
   */
  title: string;
}

type Hook0TableCellDateParameter = ICellRendererParams & ExtraParams;

interface Props {
  params: Hook0TableCellDateParameter;
}

defineProps<Props>();
</script>

<template>
  <Hook0Icon
    :title="
      typeof params.colDef?.cellRendererParams.title === 'function'
        ? params.colDef.cellRendererParams.title(params.data)
        : params.colDef?.cellRendererParams.title
    "
    :name="
      typeof params.colDef?.cellRendererParams.icon === 'function'
        ? params.colDef.cellRendererParams.icon(params.data)
        : params.colDef?.cellRendererParams.icon
    "
  ></Hook0Icon>
</template>
