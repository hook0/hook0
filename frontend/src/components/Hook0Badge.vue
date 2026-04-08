<script setup lang="ts">
import { computed } from 'vue';

type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info';
type BadgeSize = 'sm' | 'md';
type BadgeDisplay = 'badge' | 'trust' | 'step';

const statusVariants: ReadonlySet<string> = new Set(['success', 'warning', 'danger', 'info']);

type Props = {
  variant?: BadgeVariant;
  size?: BadgeSize;
  display?: BadgeDisplay;
  ariaLabel?: string;
};

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'md',
  display: 'badge',
  ariaLabel: undefined,
});

const isStatusBadge = computed(() => statusVariants.has(props.variant));

defineSlots<{
  default(): unknown;
  icon(): unknown;
}>();
</script>

<template>
  <!-- Step indicator display (numbered step badge) -->
  <span
    v-if="display === 'step'"
    class="hook0-badge-step"
    :class="[`hook0-badge-step--${variant}`]"
    :role="isStatusBadge ? 'status' : undefined"
    :aria-label="ariaLabel"
  >
    <slot />
  </span>

  <!-- Trust indicator display (icon + text, no background) -->
  <span
    v-else-if="display === 'trust'"
    class="hook0-badge-trust"
    :class="[`hook0-badge-trust--${variant}`]"
    :role="isStatusBadge ? 'status' : undefined"
    :aria-label="ariaLabel"
  >
    <span v-if="$slots.icon" class="hook0-badge-trust__icon">
      <slot name="icon" />
    </span>
    <slot />
  </span>

  <!-- Standard badge display -->
  <span
    v-else
    class="hook0-badge"
    :class="[variant, size]"
    :role="isStatusBadge ? 'status' : undefined"
    :aria-label="ariaLabel"
  >
    <slot />
  </span>
</template>

<style scoped>
/* Standard badge styles */
.hook0-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-weight: 500;
  border-radius: 9999px;
  white-space: nowrap;
}

.hook0-badge.sm {
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
}

.hook0-badge.md {
  padding: 0.25rem 0.625rem;
  font-size: 0.75rem;
  line-height: 1.25rem;
}

.hook0-badge.default {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.hook0-badge.primary {
  background-color: color-mix(in srgb, var(--color-primary) 15%, transparent);
  color: var(--color-primary);
}

.hook0-badge.success {
  background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
  color: var(--color-success);
}

.hook0-badge.warning {
  background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
  color: var(--color-warning);
}

.hook0-badge.danger {
  background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
  color: var(--color-danger);
}

.hook0-badge.info {
  background-color: color-mix(in srgb, var(--color-info) 15%, transparent);
  color: var(--color-info);
}

/* Trust indicator styles */
.hook0-badge-trust {
  display: inline-flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.hook0-badge-trust__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.hook0-badge-trust--default .hook0-badge-trust__icon {
  color: var(--color-text-tertiary);
}

.hook0-badge-trust--primary .hook0-badge-trust__icon {
  color: var(--color-primary);
}

.hook0-badge-trust--success .hook0-badge-trust__icon {
  color: var(--color-success);
}

.hook0-badge-trust--warning .hook0-badge-trust__icon {
  color: var(--color-warning);
}

.hook0-badge-trust--danger .hook0-badge-trust__icon {
  color: var(--color-danger);
}

.hook0-badge-trust--info .hook0-badge-trust__icon {
  color: var(--color-info);
}

/* Step indicator styles */
.hook0-badge-step {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--radius-md);
  font-size: 0.8125rem;
  font-weight: 700;
  flex-shrink: 0;
}

.hook0-badge-step--default {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.hook0-badge-step--primary {
  background-color: var(--color-primary);
  color: var(--color-bg-primary, #ffffff);
}

.hook0-badge-step--success {
  background-color: var(--color-success);
  color: var(--color-bg-primary, #ffffff);
}

.hook0-badge-step--warning {
  background-color: var(--color-warning);
  color: var(--color-bg-primary, #ffffff);
}

.hook0-badge-step--danger {
  background-color: var(--color-danger);
  color: var(--color-bg-primary, #ffffff);
}

.hook0-badge-step--info {
  background-color: var(--color-info);
  color: var(--color-bg-primary, #ffffff);
}
</style>
