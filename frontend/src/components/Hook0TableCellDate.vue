<template>
  <abbr v-if="getValue()" :title="formatDate(getValue())">
    <hook0-text>
      {{ formatHumanReadableDate(getValue()) }}
    </hook0-text>
  </abbr>
  <hook0-text v-else>
    {{ emptyValue() }}
  </hook0-text>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';
import { ICellRendererParams } from '@ag-grid-community/core';
import Hook0Text from '@/components/Hook0Text.vue';
import { formatISO9075, formatDistance, parseISO } from 'date-fns';

interface ExtraParams<T> {
  /**
   * Text to display if date is null
   */
  defaultText?: string;
}

type Hook0TableCellDateParameter<T> = ICellRendererParams & ExtraParams<T>;

@Options({
  name: 'hook0-table-cell-date',
  inheritAttrs: false,
  components: { Hook0Text },
})
export default class Hook0TableCellCode<T> extends Vue {
  private params!: Hook0TableCellDateParameter<T>;

  getValue(): string | null {
    if (
      typeof this.params.colDef?.cellRendererParams !== 'undefined' &&
      typeof this.params.colDef.cellRendererParams?.value !== 'undefined' // eslint-disable-line @typescript-eslint/no-unsafe-member-access
    ) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      if (typeof this.params.colDef.cellRendererParams.value === 'function') {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        return this.params.colDef.cellRendererParams.value(this.params.data) ?? null;
      } else {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access
        return this.params.colDef.cellRendererParams.value ?? null;
      }
    } else {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return this.params?.value ?? null;
    }
  }

  emptyValue(): string {
    return this.params.defaultText ?? '';
  }

  formatDate(value: string | null): string {
    if (value === null || value === '') {
      return '';
    } else {
      return formatISO9075(parseISO(value));
    }
  }

  formatHumanReadableDate(value: string | null): string {
    if (value === null || value === '') {
      return '';
    } else {
      return formatDistance(parseISO(value), new Date(), { addSuffix: true });
    }
  }
}
</script>
<style>
.ag-theme-alpine .ag-cell {
  line-height: 40px;
}
</style>
