<script setup lang="ts">
import { RouteLocationRaw, useRouter } from 'vue-router';
import { ref, computed, onMounted, onUpdated, useSlots, useAttrs } from 'vue';

import Hook0Spinner from '@/components/Hook0Spinner.vue';
import Hook0Tooltip from '@/components/Hook0Tooltip.vue';

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'link' | 'icon';
type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';

type Props = {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean | Promise<unknown>;
  to?: RouteLocationRaw;
  href?: string;
  disabled?: boolean;
  submit?: boolean;
  tooltip?: string;
  fullWidth?: boolean;
};

const router = useRouter();
const props = withDefaults(defineProps<Props>(), {
  variant: 'secondary',
  size: 'md',
  loading: false,
  to: undefined,
  href: undefined,
  disabled: false,
  submit: false,
  tooltip: undefined,
  fullWidth: false,
});
const emit = defineEmits<{
  click: [e: MouseEvent];
}>();
defineSlots<{
  default(): unknown;
  left(): unknown;
  right(): unknown;
}>();

const attrs = useAttrs();

const resolvedHref = computed(() => {
  if (props.href) {
    return props.href;
  }

  if (!props.to) {
    return undefined;
  }

  const { href } = router.resolve(props.to);
  return href;
});

/**
 * Determine whether to render as <button> or <a>.
 *
 * Render as <a> only when the button has navigation behavior (href or to).
 * Otherwise always render as <button> for correct semantics, accessibility,
 * and keyboard interaction.
 */
const isLink = computed(() => {
  return !!props.href || !!props.to;
});

/**
 * Compute the button type attribute.
 * - If submit prop is true, use "submit"
 * - If type is explicitly passed in $attrs, use that
 * - Otherwise default to "button"
 */
type ButtonType = 'submit' | 'button' | 'reset';

const buttonType = computed((): ButtonType => {
  if (props.submit) return 'submit';
  if (
    attrs.type &&
    typeof attrs.type === 'string' &&
    ['submit', 'button', 'reset'].includes(attrs.type)
  ) {
    return attrs.type as ButtonType;
  }
  return 'button';
});

/**
 * Filter out 'type' from $attrs when rendering as <button>,
 * since we handle it explicitly via :type="buttonType".
 */
const filteredAttrs = computed(() => {
  const result: Record<string, unknown> = {};
  for (const key of Object.keys(attrs)) {
    if (key !== 'type') {
      result[key] = attrs[key];
    }
  }
  return result;
});

const loading = computed(() => props.loading ?? false);
const loadingStatus = ref(false);

function forwardPromiseState() {
  if (!(loading.value instanceof Promise)) {
    loadingStatus.value = loading.value;
    return;
  }

  loadingStatus.value = true;
  void loading.value.finally(() => {
    if (!(loading.value instanceof Promise)) {
      return;
    }
    loadingStatus.value = false;
  });
}

function onClick(e: MouseEvent) {
  if (loadingStatus.value || props.disabled) {
    e.preventDefault();
    return;
  }

  // For links, handle navigation
  if (isLink.value) {
    if (e.metaKey && resolvedHref.value) {
      e.preventDefault();
      window.open(resolvedHref.value);
      return;
    }

    if (props.to) {
      e.preventDefault();
      void router.push(props.to);
      return;
    }

    // External href - let the browser handle it
    return;
  }

  // For buttons, emit click
  emit('click', e);
}

function hasSlot(name: string): boolean {
  return !!useSlots()[name];
}

onMounted(() => {
  forwardPromiseState();
});

onUpdated(() => {
  forwardPromiseState();
});

const sizeClasses: Record<ButtonSize, string> = {
  xs: 'px-2 py-0.5 text-xs',
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-4 py-2 text-sm',
  lg: 'px-5 py-2.5 text-base',
};

const spinnerSize: Record<ButtonSize, number> = {
  xs: 12,
  sm: 14,
  md: 16,
  lg: 18,
};
</script>

<template>
  <!-- Render as <a> when the button has navigation behavior (href or to) -->
  <a
    v-if="isLink"
    class="hook0-button"
    :class="[
      variant,
      variant !== 'icon' && variant !== 'link' ? sizeClasses[size] : '',
      { loading: loadingStatus, 'full-width': fullWidth },
    ]"
    :aria-disabled="loadingStatus || disabled || undefined"
    :aria-busy="loadingStatus || undefined"
    :href="resolvedHref"
    :title="tooltip"
    :tabindex="loadingStatus || disabled ? -1 : undefined"
    v-bind="$attrs"
    @click="onClick($event)"
  >
    <span v-if="hasSlot('left') && !loadingStatus" class="hook0-button-left">
      <slot name="left" />
    </span>
    <Hook0Spinner v-if="loadingStatus" :size="spinnerSize[size]" />
    <span v-else class="hook0-button-center">
      <slot />
    </span>
    <span v-if="hasSlot('right') && !loadingStatus" class="hook0-button-right">
      <slot name="right" />
    </span>
  </a>
  <!-- Disabled button with tooltip: wrap in Hook0Tooltip so hover works on disabled element -->
  <Hook0Tooltip v-else-if="disabled && tooltip" :content="tooltip">
    <button
      :type="buttonType"
      class="hook0-button"
      :class="[
        variant,
        variant !== 'icon' && variant !== 'link' ? sizeClasses[size] : '',
        { loading: loadingStatus, 'full-width': fullWidth },
      ]"
      :disabled="loadingStatus || disabled"
      :aria-disabled="loadingStatus || disabled || undefined"
      :aria-busy="loadingStatus || undefined"
      v-bind="filteredAttrs"
      @click="onClick($event)"
    >
      <span v-if="hasSlot('left') && !loadingStatus" class="hook0-button-left">
        <slot name="left" />
      </span>
      <Hook0Spinner v-if="loadingStatus" :size="spinnerSize[size]" />
      <span v-else class="hook0-button-center">
        <slot />
      </span>
      <span v-if="hasSlot('right') && !loadingStatus" class="hook0-button-right">
        <slot name="right" />
      </span>
    </button>
  </Hook0Tooltip>
  <!-- Render as <button> for all other cases (actions, submit, etc.) -->
  <button
    v-else
    :type="buttonType"
    class="hook0-button"
    :class="[
      variant,
      variant !== 'icon' && variant !== 'link' ? sizeClasses[size] : '',
      { loading: loadingStatus, 'full-width': fullWidth },
    ]"
    :disabled="loadingStatus || disabled"
    :aria-disabled="loadingStatus || disabled || undefined"
    :aria-busy="loadingStatus || undefined"
    :title="tooltip"
    v-bind="filteredAttrs"
    @click="onClick($event)"
  >
    <span v-if="hasSlot('left') && !loadingStatus" class="hook0-button-left">
      <slot name="left" />
    </span>
    <Hook0Spinner v-if="loadingStatus" :size="spinnerSize[size]" />
    <span v-else class="hook0-button-center">
      <slot />
    </span>
    <span v-if="hasSlot('right') && !loadingStatus" class="hook0-button-right">
      <slot name="right" />
    </span>
  </button>
</template>

<style scoped>
.hook0-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-wrap: nowrap;
  gap: 0.5rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease,
    box-shadow 0.15s ease,
    transform 50ms ease;
  user-select: none;
  text-decoration: none;
  white-space: nowrap;
}

.hook0-button:focus:not(:focus-visible) {
  outline: none;
}

.hook0-button:focus-visible {
  box-shadow:
    0 0 0 2px var(--color-bg-primary),
    0 0 0 4px var(--color-primary);
}

.hook0-button:active:not([disabled]):not([aria-disabled='true']) {
  transform: scale(0.98);
}

.hook0-button[disabled],
.hook0-button[aria-disabled='true'] {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Variants */
.hook0-button.primary {
  background-color: var(--color-primary);
  color: white;
  border: 1px solid var(--color-primary);
  box-shadow: var(--shadow-sm);
}

.hook0-button.primary:hover:not([disabled]) {
  background-color: var(--color-primary-hover);
  border-color: var(--color-primary-hover);
}

.hook0-button.secondary {
  background-color: var(--color-bg-primary);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}

.hook0-button.secondary:hover:not([disabled]) {
  background-color: var(--color-bg-secondary);
}

.hook0-button.danger {
  background-color: var(--color-danger);
  color: white;
  border: 1px solid var(--color-danger);
  box-shadow: var(--shadow-sm);
}

.hook0-button.danger:hover:not([disabled]) {
  background-color: var(--color-danger-hover);
  border-color: var(--color-danger-hover);
}

.hook0-button.ghost {
  background-color: transparent;
  color: var(--color-text-secondary);
  border: 1px solid transparent;
}

.hook0-button.ghost:hover:not([disabled]) {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-button.link {
  background-color: transparent;
  color: var(--color-primary);
  border: 1px solid transparent;
  padding-left: 0;
  padding-right: 0;
  font-size: inherit;
}

.hook0-button.link:hover:not([disabled]) {
  color: var(--color-primary-hover);
  text-decoration: underline;
}

.hook0-button.icon {
  background-color: transparent;
  color: var(--color-text-tertiary);
  border: none;
  padding: 0;
  opacity: 0.6;
  border-radius: 0;
}

.hook0-button.icon:hover:not([disabled]) {
  opacity: 1;
}

.hook0-button.icon:active:not([disabled]) {
  transform: none;
}

/* Legacy class support for backwards compatibility */
.hook0-button.white {
  background-color: var(--color-bg-primary);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}

.hook0-button.white:hover:not([disabled]) {
  background-color: var(--color-bg-secondary);
}

.hook0-button-left,
.hook0-button-right {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
}

.hook0-button :deep(svg) {
  flex-shrink: 0;
}

.hook0-button-center {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
  white-space: nowrap;
}

/* Full width variant */
.hook0-button.full-width {
  width: 100%;
}
</style>
