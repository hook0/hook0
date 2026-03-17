<script setup lang="ts">
/**
 * Hook0IconBadge - A wrapper component for icons with semantic background colors
 *
 * Used to display icons with colored backgrounds indicating status or category.
 * Commonly used in card headers and section titles.
 */

type Variant = 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'neutral';
type Size = 'sm' | 'md' | 'lg';

type Props = {
  variant?: Variant;
  size?: Size;
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
});

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div
    class="hook0-icon-badge"
    :class="[`hook0-icon-badge--${variant}`, `hook0-icon-badge--${size}`]"
  >
    <slot />
  </div>
</template>

<style scoped>
.hook0-icon-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg);
  flex-shrink: 0;
}

/* Sizes */
.hook0-icon-badge--sm {
  width: 1.5rem;
  height: 1.5rem;
}

.hook0-icon-badge--md {
  width: 2rem;
  height: 2rem;
}

.hook0-icon-badge--lg {
  width: 2.5rem;
  height: 2.5rem;
}

/* Variants */
.hook0-icon-badge--primary,
.hook0-icon-badge--info {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.hook0-icon-badge--success {
  background-color: var(--color-success-light);
  color: var(--color-success);
}

.hook0-icon-badge--warning {
  background-color: var(--color-warning-light);
  color: var(--color-warning);
}

.hook0-icon-badge--danger {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.hook0-icon-badge--neutral {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}
</style>
