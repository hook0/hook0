<script setup lang="ts">
import { ref, onMounted, useSlots } from 'vue';

import Hook0Text from '@/components/Hook0Text.vue';

interface Props {
  autofocus?: boolean;
  label?: string;
  error?: string;
}

const props = withDefaults(defineProps<Props>(), {
  autofocus: false,
  label: undefined,
  error: undefined,
});

defineOptions({
  inheritAttrs: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

defineSlots<{
  helpText(): unknown;
}>();

const textarea = ref<null | HTMLTextAreaElement>(null);

function hasSlot(name: string): boolean {
  return !!useSlots()[name];
}

onMounted(() => {
  if (props.autofocus) {
    textarea.value?.focus();
  }
});
</script>

<template>
  <div :class="$attrs.class">
    <label v-if="label" class="hook0-textarea-label">{{ label }}</label>
    <textarea
      ref="textarea"
      v-bind="$attrs"
      class="hook0-textarea"
      :class="{ 'hook0-textarea-error': error }"
      :value="($attrs as Record<string, unknown>).modelValue as string"
      :aria-invalid="!!error"
      :aria-describedby="error ? `${String($attrs.id || '')}-error` : undefined"
      @input="(e: Event) => emit('update:modelValue', (e.target as HTMLTextAreaElement)?.value)"
    />
    <p
      v-if="error"
      :id="`${String($attrs.id || '')}-error`"
      class="hook0-textarea-error-text"
      role="alert"
    >
      {{ error }}
    </p>
    <div v-if="hasSlot('helpText')">
      <Hook0Text class="helpText">
        <slot name="helpText" />
      </Hook0Text>
    </div>
  </div>
</template>

<style scoped>
.hook0-textarea {
  display: block;
  width: 100%;
  min-height: 5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
  resize: vertical;
  font-family: inherit;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.hook0-textarea::placeholder {
  color: var(--color-text-muted);
}

.hook0-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow:
    0 0 0 1px var(--color-primary),
    var(--shadow-sm);
}

.hook0-textarea-error {
  border-color: var(--color-danger);
}

.hook0-textarea-error:focus {
  border-color: var(--color-danger);
  box-shadow: 0 0 0 1px var(--color-danger);
}

.hook0-textarea-error-text {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-danger);
}

.hook0-textarea-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 0.375rem;
}
</style>
