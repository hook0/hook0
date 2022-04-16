<template>
  <div class="hook0-card-footer">
    <slot></slot>
  </div>
</template>

<script lang="ts">
import {Vue} from "vue-class-component";

export default class Hook0CardFooter extends Vue {
}
</script>

<style lang="scss" scoped>
.hook0-card-footer {
  @apply mt-6 flex items-center justify-end space-x-4 px-4 py-5 sm:p-6;
}
</style>
