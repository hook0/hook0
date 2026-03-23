<script setup lang="ts">
/**
 * Hook0CardDivider - A horizontal divider for use inside Hook0Card
 *
 * Can optionally display centered text (useful for "or" dividers in forms)
 */

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-divider" :class="{ 'hook0-card-divider--with-text': $slots.default }">
    <span v-if="$slots.default" class="hook0-card-divider__text">
      <slot />
    </span>
  </div>
</template>

<style scoped>
.hook0-card-divider {
  height: 1px;
  margin: 1rem 0;
  background-color: var(--color-border);
}

.hook0-card-divider--with-text {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: auto;
  background-color: transparent;
}

.hook0-card-divider--with-text::before,
.hook0-card-divider--with-text::after {
  content: '';
  flex: 1;
  height: 1px;
  background-color: var(--color-border);
}

.hook0-card-divider__text {
  padding: 0 1rem;
  font-size: 0.875rem;
  color: var(--color-text-tertiary);
  white-space: nowrap;
}
</style>
