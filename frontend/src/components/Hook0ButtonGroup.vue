<script setup lang="ts">
import Hook0Stack from '@/components/Hook0Stack.vue';

type Direction = 'row' | 'column';
type Align = 'start' | 'center' | 'end' | 'stretch';
type Gap = 'sm' | 'md' | 'lg';

type Props = {
  direction?: Direction;
  align?: Align;
  gap?: Gap;
  fullWidth?: boolean;
  responsive?: boolean;
};

withDefaults(defineProps<Props>(), {
  direction: 'row',
  align: 'center',
  gap: 'md',
  fullWidth: false,
  responsive: false,
});

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <Hook0Stack
    :direction="direction"
    :align="align"
    :gap="gap"
    :responsive="responsive"
    class="hook0-button-group"
    :class="{ 'hook0-button-group--full-width': fullWidth }"
  >
    <slot />
  </Hook0Stack>
</template>

<style scoped>
.hook0-button-group--full-width {
  width: 100%;
}

.hook0-button-group--full-width :deep(> *) {
  flex: 1;
}
</style>
