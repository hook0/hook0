<script setup lang="ts">
import { omit } from 'ramda';

function omitClass(props: Record<string, unknown>) {
  return omit(['class'], props);
}
</script>

<template>
  <span
    v-bind="omitClass({ ...$props, ...$attrs })"
    :class="['text'].concat($attrs.class as string[] ?? [])"
  >
    <slot></slot>
  </span>
</template>

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

  &.darkmode {
    @apply text-gray-300;
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
