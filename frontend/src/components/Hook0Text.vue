<template>
  <span v-bind="{ ...$props, ...$attrs }" class="text">
    <slot></slot>
  </span>
</template>

<script lang="ts">
export default {
  name: 'hook0-text',
  data() {
    return {};
  },
};
</script>

<style lang="scss" scoped>
.text {
  @apply font-normal;

  &.medium {
    @apply font-medium;
  }
}
</style>
