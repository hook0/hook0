<script setup lang="ts">
import { useId } from 'vue';

interface Props {
  label: string;
  error: string;
  helpText: string;
  required: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  label: '',
  error: '',
  helpText: '',
  required: false,
});

defineSlots<{
  default(): unknown;
}>();

const fieldId = `hook0-form-field-${useId()}`;
const errorId = `${fieldId}-error`;
const helpId = `${fieldId}-help`;
</script>

<template>
  <div class="hook0-form-field">
    <label v-if="props.label" :for="fieldId" class="hook0-form-field__label">
      {{ props.label }}
      <span v-if="props.required" class="hook0-form-field__required" aria-hidden="true">*</span>
    </label>

    <div
      class="hook0-form-field__control"
      :aria-describedby="
        [props.error ? errorId : '', props.helpText ? helpId : ''].filter(Boolean).join(' ') ||
        undefined
      "
    >
      <slot />
    </div>

    <p v-if="props.error" :id="errorId" class="hook0-form-field__error" role="alert">
      {{ props.error }}
    </p>

    <p v-if="props.helpText" :id="helpId" class="hook0-form-field__help">
      {{ props.helpText }}
    </p>
  </div>
</template>

<style scoped>
.hook0-form-field {
  display: flex;
  flex-direction: column;
}

.hook0-form-field__label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 0.375rem;
}

.hook0-form-field__required {
  color: var(--color-error);
  margin-left: 0.125rem;
}

.hook0-form-field__control {
  position: relative;
}

.hook0-form-field__error {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--color-error);
}

.hook0-form-field__help {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--color-text-tertiary);
}
</style>
