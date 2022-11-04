<template>
  <div class="card-content">
    <slot></slot>
  </div>
</template>

<script lang="ts">
import { Vue } from 'vue-class-component';

export default class Hook0CardContent extends Vue {}
</script>

<style lang="scss" scoped>
.card-content {
  @apply border-t border-gray-200 px-4 py-5 sm:p-0;
}
</style>
