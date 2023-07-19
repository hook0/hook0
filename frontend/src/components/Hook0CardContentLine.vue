<script setup lang="ts">
import { computed } from 'vue';
import Hook0Text from '@/components/Hook0Text.vue';

interface Props {
  type?: 'split' | 'full-width' | 'split-content-component' | 'stacked' | 'columns';
}
const props = defineProps<Props>();
const type = computed(() => props.type ?? 'split');

defineSlots<{
  label(): unknown;
  content(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-content-line" :class="type">
    <dt>
      <Hook0Text class="label">
        <slot name="label"></slot>
      </Hook0Text>
    </dt>
    <dd class="hook0-card-content-line-content">
      <slot name="content"></slot>
    </dd>
  </div>
</template>

<style lang="scss" scoped>
.hook0-card-content-line {
  @apply py-4 sm:py-5 sm:px-6;

  &.stacked {
    @apply grid-rows-2;
  }

  &.columns .hook0-card-content-line-content {
    @apply grid grid-flow-col auto-cols-auto gap-x-7;
  }

  &.split,
  &.split-content-component {
    @apply sm:grid sm:grid-cols-3 sm:gap-4;
  }

  &.full-width {
    .hook0-card-content-line-content {
      @apply py-4;
    }
  }

  &.split-content-component {
    .hook0-card-content-line-content {
      @apply pt-0 mt-0;
    }
  }
}

.hook0-card-content-line-content {
  @apply mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2;
}
</style>
