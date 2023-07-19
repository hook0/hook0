<script setup lang="ts">
import { ICellRendererParams } from '@ag-grid-community/core';

import Hook0Text from '@/components/Hook0Text.vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  params: ICellRendererParams;
}

defineProps<Props>();
</script>

<template>
  <Hook0Text class="code" style="width: fit-content"
    >{{
      params.colDef?.cellRendererParams && params.colDef.cellRendererParams.value
        ? typeof params.colDef.cellRendererParams.value === 'function'
          ? params.colDef.cellRendererParams.value(params.data)
          : params.colDef.cellRendererParams.value
        : params.value
    }}
  </Hook0Text>
</template>

<style>
.ag-theme-alpine .ag-cell {
  line-height: 40px;
}
</style>
