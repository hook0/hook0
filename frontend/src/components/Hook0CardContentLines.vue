<template>
  <dl class="hook0-card-content-lines">
    <slot></slot>
  </dl>
</template>

<script lang="ts">
import {Vue} from "vue-class-component";

export default class Hook0CardContentLines extends Vue {
}

</script>

<style lang="scss" scoped>
.hook0-card-content-lines {
  @apply sm:divide-y sm:divide-gray-200;
}
</style>
