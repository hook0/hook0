<script setup lang="ts">
import type { Component } from 'vue';
import type { RouteLocationRaw } from 'vue-router';

import Hook0Button from '@/components/Hook0Button.vue';

defineOptions({
  inheritAttrs: false,
});

type Props = {
  value: string;
  icon?: Component;
  onClick?: () => void;
  href?: string;
  to?: RouteLocationRaw;
  disabled?: boolean;
  dataTest?: string;
}

const props = defineProps<Props>();

function handleClick(event: Event) {
  event.stopImmediatePropagation();
  event.preventDefault();

  if (props.onClick) {
    props.onClick();
  }
}
</script>

<template>
  <Hook0Button
    v-bind="{
      href: props.href,
      to: props.to,
      onClick: props.onClick ? handleClick : undefined,
      class: $attrs.class,
      disabled: props.disabled,
    }"
    :data-test="props.dataTest"
    style="width: fit-content"
  >
    <template v-if="props.icon" #left>
      <component :is="props.icon" :size="14" aria-hidden="true" class="mr-1" />
    </template>
    <template #default>
      {{ props.value }}
    </template>
  </Hook0Button>
</template>
