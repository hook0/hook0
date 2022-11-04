<template>
  <ul class="list" v-bind="{ ...$props, ...$attrs }">
    <slot></slot>
  </ul>
</template>

<script lang="ts">
import { Vue } from 'vue-class-component';

export default class Hook0List extends Vue {}
</script>

<style lang="scss" scoped>
.list {
  @apply border border-gray-200 rounded-md divide-y divide-gray-200;
}
</style>
