<template>
  <li class="list-item">
    <div class="list-item-left">
      <slot name="left"></slot>
    </div>
    <div class="list-item-right">
      <slot name="right"></slot>
    </div>
  </li>
</template>

<script lang="ts">
import { Vue } from 'vue-class-component';

export default class Hook0ListItem extends Vue {}
</script>

<style lang="scss" scoped>
.list-item {
  @apply pl-3 pr-4 py-3 flex items-center justify-between text-sm;
}

.list-item-left {
  @apply w-0 flex-1 flex items-center;
}

.list-item-right {
  @apply ml-4 flex-shrink-0;
}
</style>
