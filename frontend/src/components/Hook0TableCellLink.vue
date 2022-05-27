<template>
  <hook0-button
    v-bind="{
    href: params.colDef.cellRendererParams && params.colDef.cellRendererParams.href ? params.colDef.cellRendererParams.href(params.data) : undefined,
    to: params.colDef.cellRendererParams && params.colDef.cellRendererParams.to ? params.colDef.cellRendererParams.to(params.data) : undefined,
    onClick: params.colDef.cellRendererParams && params.colDef.cellRendererParams.onClick ? onClick : undefined
     }"
    style="width: fit-content"
  >
    <template #left v-if="params.colDef.cellRendererParams && params.colDef.cellRendererParams.icon">
      <hook0-icon class="mr-1" :name="params.colDef.cellRendererParams.icon"></hook0-icon>
    </template>
    <template #default>
      {{
        params.colDef.cellRendererParams && params.colDef.cellRendererParams.value ? (typeof params.colDef.cellRendererParams.value === 'function' ? params.colDef.cellRendererParams.value(params.data) : params.colDef.cellRendererParams.value) : params.value
      }}
    </template>
  </hook0-button>
</template>


<script lang="ts">
import {Vue, Options} from 'vue-class-component';
import {ICellRendererParams} from "@ag-grid-community/core";
import Hook0Icon from "@/components/Hook0Icon.vue";
import {RouteLocation} from "vue-router";

interface ExtraParams<T> {

  /**
   * Raw value
   */
  value?: (row: T) => string | string;

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
  href: (row: T) => string

  /**
   * RouteLocation factory
   */
  to: (row: T) => RouteLocation
}


@Options({
  name: 'hook0-table-cell-link',
  inheritAttrs: false,
  props: {
    params: {
      type: Object,
      required: true,
    }
  },
  components: {Hook0Icon}
})
export default class Hook0TableCellLink<T> extends Vue {
  private params!: ICellRendererParams & ExtraParams<T>;

  onClick<T>(event: Event): any {
    event.stopImmediatePropagation();
    event.preventDefault();

    if (!this.params.onClick) {
      return;
    }

    // eslint-disable-next-line
    this.params.onClick.call(this.params.context, this.params.data);
  }
};
</script>
<style>
</style>

