<script setup lang="ts">
import { ICellRendererParams } from 'ag-grid-community';
import { RouteLocation } from 'vue-router';

import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';

defineOptions({
  inheritAttrs: false,
});

interface ExtraParams<T> {
  /**
   * Raw value
   */
  value?: (row: T) => string;

  /**
   * Icon to add to the button
   */
  icon?: string;

  /**
   * Click handler
   */
  onClick?: (row: T) => void;

  /**
   * RouteLocation factory
   */
  href: (row: T) => string;

  /**
   * RouteLocation factory
   */
  to: (row: T) => RouteLocation;

  /**
   * If true, link will be disabled
   */
  disabled?: boolean;

  /**
   * data-test attribute for E2E testing
   */
  dataTest?: string | ((row: T) => string);
}

type Hook0TableCellDateParameter<T> = ICellRendererParams & ExtraParams<T>;

interface Props {
  params: Hook0TableCellDateParameter<object[]>;
}

const props = defineProps<Props>();

function onClick(event: Event) {
  event.stopImmediatePropagation();
  event.preventDefault();

  if (!props.params.onClick) {
    return;
  }

  props.params.onClick(props.params.data as object[]);
}
</script>

<template>
  <Hook0Button
    v-bind="{
      href:
        params.colDef?.cellRendererParams && params.colDef.cellRendererParams.href
          ? params.colDef.cellRendererParams.href(params.data)
          : undefined,
      to:
        params.colDef?.cellRendererParams && params.colDef.cellRendererParams.to
          ? params.colDef.cellRendererParams.to(params.data)
          : undefined,
      onClick:
        params.colDef?.cellRendererParams && params.colDef.cellRendererParams.onClick
          ? onClick
          : undefined,
      class: $attrs.class,
      disabled:
        params.colDef?.cellRendererParams && params.colDef.cellRendererParams.disabled
          ? params.colDef.cellRendererParams.disabled(params.data)
          : undefined,
    }"
    :data-test="
      params.colDef?.cellRendererParams && params.colDef.cellRendererParams.dataTest
        ? typeof params.colDef.cellRendererParams.dataTest === 'function'
          ? params.colDef.cellRendererParams.dataTest(params.data)
          : params.colDef.cellRendererParams.dataTest
        : undefined
    "
    style="width: fit-content"
  >
    <template
      v-if="params.colDef?.cellRendererParams && params.colDef.cellRendererParams.icon"
      #left
    >
      <Hook0Icon
        class="mr-1"
        :name="
          typeof params.colDef.cellRendererParams.icon === 'function'
            ? params.colDef.cellRendererParams.icon(params.data)
            : params.colDef.cellRendererParams.icon
        "
      ></Hook0Icon>
    </template>
    <template #default>
      {{
        params.colDef?.cellRendererParams && params.colDef.cellRendererParams.value
          ? typeof params.colDef.cellRendererParams.value === 'function'
            ? params.colDef.cellRendererParams.value(params.data)
            : params.colDef.cellRendererParams.value
          : params.value
      }}
    </template>
  </Hook0Button>
</template>
