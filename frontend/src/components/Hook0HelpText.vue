<script setup lang="ts">
/**
 * Hook0HelpText - Contextual help text for form fields and UI elements
 *
 * Wraps Hook0Text with semantic tones for different types of feedback.
 * Use below form fields, in card headers, or anywhere contextual help is needed.
 *
 * @example
 * <Hook0HelpText>This field is required</Hook0HelpText>
 * <Hook0HelpText tone="warning">This action cannot be undone</Hook0HelpText>
 * <Hook0HelpText tone="info">Learn more about webhooks</Hook0HelpText>
 */
import Hook0Text from '@/components/Hook0Text.vue';

type HelpTextTone = 'neutral' | 'warning' | 'info' | 'emphasis';

interface Props {
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
  <Hook0Text
    variant="secondary"
    size="md"
    :weight="tone === 'emphasis' ? 'medium' : 'normal'"
    block
    class="hook0-help-text"
    :class="[`hook0-help-text--${tone}`]"
  >
    <slot>{{ text }}</slot>
  </Hook0Text>
</template>

<style scoped>
.hook0-help-text {
  margin-top: 0.5rem;
}

/* Tones */
.hook0-help-text--neutral {
  /* Uses Hook0Text secondary defaults */
}

.hook0-help-text--warning {
  color: var(--color-warning);
}

.hook0-help-text--info {
  color: var(--color-info);
}

.hook0-help-text--emphasis {
  font-style: italic;
}
</style>
