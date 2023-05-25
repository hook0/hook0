<template>
  <div class="grid grid-flow-col auto-cols-max gap-1">
    <hook0-table-cell-link
      v-for="(param, index) in params.parameters"
      :key="index"
      :params="{ ...params, ...param }"
    >
    </hook0-table-cell-link>
  </div>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';

@Options({
  name: 'hook0-table-cell-links',
  inheritAttrs: false,
  props: {
    params: {
      type: Object, // Array
      required: true,
      validator: (v: { parameters: any[] }) => {
        return Array.isArray(v.parameters);
      },
    },
  },
  components: { Hook0TableCellLink },
})
export default class Hook0TableCellLinks<T> extends Vue {}
</script>
<style></style>
