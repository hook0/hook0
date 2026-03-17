<script setup lang="ts">
import { computed, onMounted, ref, useAttrs, useSlots } from 'vue';

import Hook0HelpText from '@/components/Hook0HelpText.vue';

type ResizeOption = 'none' | 'vertical' | 'horizontal' | 'both';

type Props = {
  autofocus?: boolean;
  label?: string;
  error?: string;
  helpText?: string;
  disabled?: boolean;
  readonly?: boolean;
  required?: boolean;
  rows?: number;
  maxlength?: number;
  resize?: ResizeOption;
}

const props = withDefaults(defineProps<Props>(), {
  autofocus: false,
  label: undefined,
  error: undefined,
  helpText: undefined,
  disabled: false,
  readonly: false,
  required: false,
  rows: 4,
  maxlength: undefined,
  resize: 'vertical',
});

defineOptions({
  inheritAttrs: false,
});

const model = defineModel<string | null>();

defineSlots<{
  helpText(): unknown;
}>();

const attrs = useAttrs();
const textareaAttrs = computed(() => {
  const { class: _cls, style: _sty, ...rest } = attrs;
  return rest;
});

const textarea = ref<null | HTMLTextAreaElement>(null);

function hasSlot(name: string): boolean {
  return !!useSlots()[name];
}

onMounted(() => {
  if (props.autofocus) {
    textarea.value?.focus();
  }
});

const modelStr = computed({
  get: () => (model.value == null ? '' : String(model.value)),
  set: (v: string) => {
    model.value = v;
  },
});

const helpTextId = computed(() => {
  const id = attrs.id as string | undefined;
  return id ? `${id}-help` : undefined;
});

const errorId = computed(() => {
  const id = attrs.id as string | undefined;
  return id ? `${id}-error` : undefined;
});

const ariaDescribedBy = computed(() => {
  if (props.error && errorId.value) {
    return errorId.value;
  }
  if ((props.helpText || hasSlot('helpText')) && helpTextId.value) {
    return helpTextId.value;
  }
  return undefined;
});
</script>

<template>
  <div :class="$attrs.class">
    <label v-if="label" class="hook0-textarea__label">{{ label }}</label>
    <textarea
      ref="textarea"
      v-bind="textareaAttrs"
      v-model="modelStr"
      class="hook0-textarea"
      :class="[
        {
          'hook0-textarea--error': error,
          'hook0-textarea--disabled': disabled,
        },
        `hook0-textarea--resize-${resize}`,
      ]"
      :disabled="disabled"
      :readonly="readonly"
      :required="required"
      :rows="rows"
      :maxlength="maxlength"
      :aria-invalid="!!error"
      :aria-describedby="ariaDescribedBy"
    />
    <p v-if="error" :id="errorId" class="hook0-textarea__error-text" role="alert">
      {{ error }}
    </p>
    <div v-if="helpText || hasSlot('helpText')" :id="helpTextId">
      <Hook0HelpText v-if="hasSlot('helpText')">
        <slot name="helpText" />
      </Hook0HelpText>
      <Hook0HelpText v-else-if="helpText" :text="helpText" />
    </div>
  </div>
</template>

<style scoped>
.hook0-textarea {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
  font-family: inherit;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.hook0-textarea::placeholder {
  color: var(--color-text-muted, #9ca3af) !important;
  opacity: 1;
}

.hook0-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow:
    0 0 0 1px var(--color-primary),
    var(--shadow-sm);
}

.hook0-textarea--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Resize variants */
.hook0-textarea--resize-none {
  resize: none;
}

.hook0-textarea--resize-vertical {
  resize: vertical;
}

.hook0-textarea--resize-horizontal {
  resize: horizontal;
}

.hook0-textarea--resize-both {
  resize: both;
}

/* Error state */
.hook0-textarea--error {
  border-color: var(--color-error);
}

.hook0-textarea--error:focus {
  border-color: var(--color-error);
  box-shadow: 0 0 0 1px var(--color-error);
}

.hook0-textarea__error-text {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-error);
}

.hook0-textarea__label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 0.375rem;
}
</style>
