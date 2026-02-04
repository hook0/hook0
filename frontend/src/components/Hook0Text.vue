<script setup lang="ts">
/**
 * Hook0Text - Design system text primitive
 *
 * A minimal, design-system-centered text component with semantic variants
 * based on visual hierarchy (not HTML elements).
 *
 * @example
 * <Hook0Text variant="primary">Main value</Hook0Text>
 * <Hook0Text variant="secondary" size="sm">Label text</Hook0Text>
 * <Hook0Text variant="muted">Annotation</Hook0Text>
 * <Hook0Text variant="mono">abc-123-xyz</Hook0Text>
 */

type TextVariant = 'primary' | 'secondary' | 'muted' | 'mono';
type TextSize = 'xs' | 'sm' | 'md' | 'lg';
type TextWeight = 'normal' | 'medium' | 'semibold' | 'bold';

interface Props {
  /** Visual hierarchy variant */
  variant?: TextVariant;
  /** Text size */
  size?: TextSize;
  /** Font weight override (optional, variants have sensible defaults) */
  weight?: TextWeight;
  /** Render as block element instead of inline */
  block?: boolean;
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  weight: undefined,
  block: false,
});

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <component
    :is="block ? 'div' : 'span'"
    class="hook0-text"
    :class="[
      `hook0-text--${variant}`,
      `hook0-text--size-${size}`,
      weight ? `hook0-text--weight-${weight}` : '',
    ]"
  >
    <slot />
  </component>
</template>

<style scoped>
.hook0-text {
  line-height: 1.5;
}

/* Variants - semantic visual hierarchy */
.hook0-text--primary {
  color: var(--color-text-primary);
  font-weight: 600;
}

.hook0-text--secondary {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.hook0-text--muted {
  color: var(--color-text-tertiary);
  font-weight: 400;
}

.hook0-text--mono {
  font-family: var(--font-mono);
  color: var(--color-text-primary);
  font-weight: 500;
  white-space: nowrap;
  user-select: text;
}

/* Sizes */
.hook0-text--size-xs {
  font-size: 0.6875rem;
}

.hook0-text--size-sm {
  font-size: 0.75rem;
}

.hook0-text--size-md {
  font-size: 0.875rem;
}

.hook0-text--size-lg {
  font-size: 1rem;
}

/* Weight overrides */
.hook0-text--weight-normal {
  font-weight: 400;
}

.hook0-text--weight-medium {
  font-weight: 500;
}

.hook0-text--weight-semibold {
  font-weight: 600;
}

.hook0-text--weight-bold {
  font-weight: 700;
}
</style>
