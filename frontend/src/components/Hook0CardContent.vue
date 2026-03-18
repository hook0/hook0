<script setup lang="ts">
type Props = {
  removeTopBorder?: boolean;
  compact?: boolean;
};

withDefaults(defineProps<Props>(), {
  removeTopBorder: false,
  compact: false,
});

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div
    class="hook0-card-content"
    :class="{
      'hook0-card-content--removeTopBorder': removeTopBorder,
      'hook0-card-content--compact': compact,
    }"
  >
    <slot />
  </div>
</template>

<style scoped>
.hook0-card-content {
  padding: 1.25rem 1.5rem;
  border-top: 1px solid var(--color-border);
}

.hook0-card-content:first-child {
  border-top: none;
}

.hook0-card-content--removeTopBorder {
  border-top: none;
  padding-top: 0;
}

.hook0-card-content--compact {
  padding: 0.375rem;
}
</style>
