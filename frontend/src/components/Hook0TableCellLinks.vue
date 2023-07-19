<script setup lang="ts">
import { ICellRendererParams } from '@ag-grid-community/core';

import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';

defineOptions({
  inheritAttrs: false,
});

type Hook0TableCellDateParameter = ICellRendererParams;

interface Props {
  params: { parameters: Hook0TableCellDateParameter };
}

defineProps<Props>();
</script>

<template>
  <div class="grid grid-flow-col auto-cols-max gap-1">
    <Hook0TableCellLink
      v-for="(param, index) in params.parameters"
      :key="index"
      :params="{ ...param }"
    >
    </Hook0TableCellLink>
  </div>
</template>
