<script setup lang="ts">
import { ICellRendererParams } from '@ag-grid-community/core';
import { formatISO9075, formatDistance, parseISO } from 'date-fns';

import Hook0Text from '@/components/Hook0Text.vue';

defineOptions({
  inheritAttrs: false,
});

interface ExtraParams {
  /**
   * Text to display if date is null
   */
  defaultText?: string;
}

type Hook0TableCellDateParameter = ICellRendererParams & ExtraParams;

interface Props {
  params: Hook0TableCellDateParameter;
}

const props = defineProps<Props>();

function getValue(): string | null {
  if (
    typeof props.params.colDef?.cellRendererParams !== 'undefined' &&
    typeof props.params.colDef.cellRendererParams?.value !== 'undefined' // eslint-disable-line @typescript-eslint/no-unsafe-member-access
  ) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    if (typeof props.params.colDef.cellRendererParams.value === 'function') {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
      return props.params.colDef.cellRendererParams.value(props.params.data) ?? null;
    } else {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access
      return props.params.colDef.cellRendererParams.value ?? null;
    }
  } else {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return props.params?.value ?? null;
  }
}

function emptyValue(): string {
  return props.params.defaultText ?? '';
}

function formatDate(value: string | null): string {
  if (value === null || value === '') {
    return '';
  } else {
    return formatISO9075(parseISO(value));
  }
}

function formatHumanReadableDate(value: string | null): string {
  if (value === null || value === '') {
    return '';
  } else {
    return formatDistance(parseISO(value), new Date(), { addSuffix: true });
  }
}
</script>

<template>
  <abbr v-if="getValue()" :title="formatDate(getValue())">
    <Hook0Text>
      {{ formatHumanReadableDate(getValue()) }}
    </Hook0Text>
  </abbr>
  <Hook0Text v-else>
    {{ emptyValue() }}
  </Hook0Text>
</template>

<style>
.ag-theme-alpine .ag-cell {
  line-height: 40px;
}
</style>
