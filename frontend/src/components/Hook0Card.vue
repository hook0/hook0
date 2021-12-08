<template>
  <div class="card">
    <slot></slot>
  </div>
</template>

<script lang="ts">
export default {
  name: 'hook0-card',
  data() {
    return {};
  },
};
</script>

<style lang="scss" scoped>
.card {
  @apply bg-white shadow overflow-hidden sm:rounded-lg;
}
</style>
