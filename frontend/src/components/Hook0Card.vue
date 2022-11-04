<template>
  <div class="card">
    <slot></slot>
  </div>
</template>

<script lang="ts">
import { Vue } from 'vue-class-component';

export default class Hook0Card extends Vue {}
</script>

<style lang="scss" scoped>
.card {
  @apply bg-white shadow overflow-hidden sm:rounded-lg mb-4;
}
</style>
