<template>
  <span v-bind="{ ...$props, ...$attrs }" class="text">
    <slot></slot>
  </span>
</template>

<script lang="ts">
import {Vue} from "vue-class-component";

export default class Hook0Text extends Vue {
}
</script>

<style lang="scss" scoped>
.text {
  @apply font-normal;

  &.medium {
    @apply font-medium;
  }

  &.bold {
    @apply font-bold;
  }

  &.def {
    @apply text-sm font-light text-gray-400;
    font-size: 0.65rem;
    line-height: 0.7rem;
  }
}
</style>
