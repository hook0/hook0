<script setup lang="ts">
import type { Component } from 'vue';
import type { RouteLocationRaw } from 'vue-router';

import Hook0Button from '@/components/Hook0Button.vue';

defineOptions({
  inheritAttrs: false,
});

// Mirrors Hook0Button variant type
type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'link' | 'icon';

type Props = {
  value: string;
  icon?: Component;
  onClick?: () => void;
  href?: string;
  to?: RouteLocationRaw;
  disabled?: boolean;
  dataTest?: string;
  variant?: ButtonVariant;
};

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
  <!-- Navigation link: plain text link, underline on hover -->
  <RouterLink
    v-if="props.to"
    :to="props.to"
    class="table-cell-nav-link"
    :class="[props.variant === 'danger' && 'table-cell-nav-link--danger', $attrs.class]"
    :data-test="props.dataTest"
    @click.stop
  >
    {{ props.value }}
  </RouterLink>
  <a
    v-else-if="props.href"
    :href="props.href"
    class="table-cell-nav-link"
    :class="[props.variant === 'danger' && 'table-cell-nav-link--danger', $attrs.class]"
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
    :variant="props.variant ?? 'secondary'"
    :data-test="props.dataTest"
    class="table-cell-action-btn"
  >
    <template v-if="props.icon" #left>
      <component :is="props.icon" :size="14" aria-hidden="true" class="table-cell-action-btn__icon" />
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

.table-cell-nav-link--danger {
  color: var(--color-error);
}

.table-cell-nav-link--danger:hover {
  color: var(--color-error);
}

.table-cell-nav-link:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.table-cell-action-btn {
  width: fit-content;
}

.table-cell-action-btn__icon {
  margin-right: 0.25rem;
}
</style>
