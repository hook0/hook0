<script setup lang="ts">
/**
 * Hook0HelpText - Contextual help text for form fields and UI elements
 *
 * Provides semantic tones for different types of feedback.
 * Use below form fields, in card headers, or anywhere contextual help is needed.
 *
 * @example
 * <Hook0HelpText>This field is required</Hook0HelpText>
 * <Hook0HelpText tone="warning">This action cannot be undone</Hook0HelpText>
 * <Hook0HelpText tone="info">Learn more about webhooks</Hook0HelpText>
 */

type HelpTextTone = 'neutral' | 'warning' | 'info' | 'emphasis';

type Props = {
  /** Text content (alternative to slot) */
  text?: string;
  /** Semantic tone for the help text */
  tone?: HelpTextTone;
}

withDefaults(defineProps<Props>(), {
  text: undefined,
  tone: 'neutral',
});

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <p class="hook0-help-text" :class="[`hook0-help-text--${tone}`]">
    <slot>{{ text }}</slot>
  </p>
</template>

<style scoped>
.hook0-help-text {
  margin-top: 0.5rem;
  color: var(--color-text-secondary);
  font-weight: 400;
  font-size: 0.875rem;
  line-height: 1.5;
  display: block;
}

/* Tones */
.hook0-help-text--neutral {
  /* Uses secondary defaults */
}

.hook0-help-text--warning {
  color: var(--color-warning);
}

.hook0-help-text--info {
  color: var(--color-info);
}

.hook0-help-text--emphasis {
  font-style: italic;
  font-weight: 500;
}
</style>
