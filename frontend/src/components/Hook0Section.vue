<script setup lang="ts">
/**
 * Lightweight section wrapper for detail panels.
 * Replaces the repeated `log-detail__section` + `log-detail__section-title` pattern.
 */

withDefaults(
  defineProps<{
    /** Section heading text. Omit for untitled sections. */
    title?: string;
    /** HTML tag for the title. */
    titleTag?: 'h3' | 'h4';
    /** Show a top border separator (used for response sections). */
    separator?: boolean;
  }>(),
  {
    title: undefined,
    titleTag: 'h3',
    separator: false,
  }
);
</script>

<template>
  <div class="hook0-section" :class="{ 'hook0-section--separator': separator }">
    <component :is="titleTag" v-if="title" class="hook0-section__title">
      {{ title }}
    </component>
    <slot />
  </div>
</template>

<style scoped>
.hook0-section {
  margin-bottom: 1.25rem;
}

.hook0-section:last-child {
  margin-bottom: 0;
}

.hook0-section--separator {
  margin-top: 1.5rem;
  padding-top: 2rem;
  border-top: 1px solid var(--color-border);
}

.hook0-section__title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-tertiary);
  margin: 0 0 0.75rem;
}
</style>
