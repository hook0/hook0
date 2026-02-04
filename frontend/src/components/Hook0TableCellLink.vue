<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router';

import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  value: string;
  icon?: string;
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
      <Hook0Icon class="mr-1" :name="props.icon" />
    </template>
    <template #default>
      {{ props.value }}
    </template>
  </Hook0Button>
</template>
