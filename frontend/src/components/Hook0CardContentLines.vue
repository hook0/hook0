<template>
  <dl class="hook0-card-content-lines">
    <slot></slot>
  </dl>
</template>

<script lang="ts">
export default {
  name: 'hook0-card-content-lines',
  data() {
    return {};
  },
};
</script>

<style lang="scss" scoped>
.hook0-card-content-lines {
  @apply sm:divide-y sm:divide-gray-200;
}
</style>
