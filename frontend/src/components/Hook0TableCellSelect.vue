<script setup lang="ts">
import { ICellRendererParams } from 'ag-grid-community';

import Hook0Select from '@/components/Hook0Select.vue';

interface ExtraParams<T> {
  /**
   * Options for the select
   */
  options: (row: T) => Array<{ value: string; label: string }>;

  /**
   * Current selected value
   */
  value: (row: T) => string;

  /**
   * Change handler
   */
  onChange?: (value: string, row: T) => void;

  /**
   * If true, select will be disabled
   */
  disabled?: boolean;
}

type Hook0TableCellSelectParameter<T> = ICellRendererParams & ExtraParams<T>;

interface Props {
  params: Hook0TableCellSelectParameter<object[]>;
}

const props = defineProps<Props>();

function onChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;

  if (!props.params.onChange) {
    return;
  }

  props.params.onChange(value, props.params.data as object[]);
}
</script>

<template>
  <Hook0Select
    v-if="
      params.colDef?.cellRendererParams &&
      params.colDef.cellRendererParams.disabled &&
      !params.colDef.cellRendererParams.disabled(params.data)
    "
    v-bind="{
      options:
        typeof params.colDef?.cellRendererParams?.options === 'function'
          ? params.colDef?.cellRendererParams.options(params.data)
          : (params.colDef?.cellRendererParams?.options ?? []),
      modelValue:
        params.colDef?.cellRendererParams?.value &&
        typeof params.colDef?.cellRendererParams?.value === 'function'
          ? params.colDef?.cellRendererParams?.value(params.data)
          : undefined,
      onChange: onChange,
      class: $attrs.class,
    }"
  />
  <div v-else class="border-gray-300 border rounded-md pl-3 pr-10 text-gray-400">
    {{
      params.colDef?.cellRendererParams?.value &&
      typeof params.colDef?.cellRendererParams?.value === 'function'
        ? params.colDef?.cellRendererParams?.value(params.data).charAt(0).toUpperCase() +
          params.colDef?.cellRendererParams?.value(params.data).slice(1)
        : undefined
    }}
  </div>
</template>
