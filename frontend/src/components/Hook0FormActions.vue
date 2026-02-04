<script setup lang="ts">
/**
 * Hook0FormActions - Form action buttons container
 *
 * Provides consistent layout for form submit/cancel buttons.
 * Supports left/right alignment and loading states.
 */

interface Props {
  align?: 'left' | 'right' | 'space-between';
}

withDefaults(defineProps<Props>(), {
  align: 'right',
});
</script>

<template>
  <div class="hook0-form-actions" :class="`hook0-form-actions--${align}`">
    <slot />
  </div>
</template>

<style scoped>
.hook0-form-actions {
  display: flex;
  gap: 0.75rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--color-border);
  margin-top: 0.5rem;
}

.hook0-form-actions--left {
  justify-content: flex-start;
}

.hook0-form-actions--right {
  justify-content: flex-end;
}

.hook0-form-actions--space-between {
  justify-content: space-between;
}
</style>
