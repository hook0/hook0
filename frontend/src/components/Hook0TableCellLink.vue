<script setup lang="ts">
import type { Component } from 'vue';
import { computed } from 'vue';
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
};

const props = defineProps<Props>();

const isNavigation = computed(() => !!props.to || !!props.href);

function handleClick(event: Event) {
  event.stopImmediatePropagation();
  event.preventDefault();

  if (props.onClick) {
    props.onClick();
  }
}
</script>

<template>
  <!-- Navigation link: plain text link, underline on hover -->
  <RouterLink
    v-if="isNavigation && props.to"
    :to="props.to"
    class="table-cell-nav-link"
    :class="$attrs.class"
    :data-test="props.dataTest"
    @click.stop
  >
    {{ props.value }}
  </RouterLink>
  <a
    v-else-if="isNavigation && props.href"
    :href="props.href"
    class="table-cell-nav-link"
    :class="$attrs.class"
    :data-test="props.dataTest"
    @click.stop
  >
    {{ props.value }}
  </a>

  <!-- Action button: keep existing Hook0Button behavior -->
  <Hook0Button
    v-else
    v-bind="{
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

<style scoped>
.table-cell-nav-link {
  color: var(--color-text-primary);
  font-weight: 400;
  font-size: 0.875rem;
  text-decoration: none;
  cursor: pointer;
}

.table-cell-nav-link:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}

.table-cell-nav-link:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}
</style>
