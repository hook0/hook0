<script setup lang="ts">
import Hook0Button from '@/components/Hook0Button.vue';

defineSlots<{
  default(): unknown;
}>();
const emit = defineEmits(['click']);
</script>

<template>
  <Hook0Button
    v-bind="{ ...$props, ...$attrs }"
    id="menu-item-0"
    class="link"
    :class="$attrs.class"
    role="menuitem"
    tabindex="-1"
    @click="emit('click', $event)"
  >
    <slot></slot>
  </Hook0Button>
</template>
