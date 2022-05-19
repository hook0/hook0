<template>
  <span
    v-bind="omit({ ...$props, ...$attrs })"
    :class="['text'].concat($attrs.class || [])">
    <slot></slot>
  </span>
</template>

<script lang="ts">
import {Vue} from "vue-class-component";
import {omit} from "ramda";

export default class Hook0Text extends Vue {
  omit(props: Record<string, any>) {
    return omit(['class'], props);
  }
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

  &.code {
    @apply font-mono font-medium text-sm whitespace-nowrap select-text cursor-text;
  }

  &.label {
    @apply text-sm font-medium text-gray-500;
  }

  &.helpText {
    @apply mt-2 text-sm text-gray-500;

  }
}
</style>
