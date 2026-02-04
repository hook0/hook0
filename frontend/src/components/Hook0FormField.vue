<script setup lang="ts">
/**
 * Hook0FormField - A form field wrapper with label and error display
 *
 * Provides consistent layout for form fields with:
 * - Label with optional required indicator
 * - Help text slot
 * - Error message display
 * - Accessible aria attributes
 */

interface Props {
  label?: string;
  name: string;
  error?: string;
  required?: boolean;
  helpText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  label: undefined,
  error: undefined,
  required: false,
  helpText: undefined,
});

const fieldId = `field-${props.name}`;
const errorId = `${fieldId}-error`;
const helpId = `${fieldId}-help`;
</script>

<template>
  <div class="hook0-form-field" :class="{ 'hook0-form-field--error': error }">
    <label v-if="label" :for="fieldId" class="hook0-form-field__label">
      {{ label }}
      <span v-if="required" class="hook0-form-field__required" aria-label="required">*</span>
    </label>

    <div class="hook0-form-field__input">
      <slot :id="fieldId" :aria-describedby="error ? errorId : helpText ? helpId : undefined" />
    </div>

    <p v-if="helpText && !error" :id="helpId" class="hook0-form-field__help">
      {{ helpText }}
    </p>

    <p v-if="error" :id="errorId" class="hook0-form-field__error" role="alert">
      {{ error }}
    </p>
  </div>
</template>

<style scoped>
.hook0-form-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.hook0-form-field__label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.hook0-form-field__required {
  color: var(--color-danger);
  margin-left: 0.125rem;
}

.hook0-form-field__input {
  display: flex;
  flex-direction: column;
}

.hook0-form-field__input :deep(input),
.hook0-form-field__input :deep(select),
.hook0-form-field__input :deep(textarea) {
  width: 100%;
}

.hook0-form-field__help {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin: 0;
}

.hook0-form-field__error {
  font-size: 0.75rem;
  color: var(--color-danger);
  margin: 0;
}

.hook0-form-field--error :deep(input),
.hook0-form-field--error :deep(select),
.hook0-form-field--error :deep(textarea) {
  border-color: var(--color-danger);
}

.hook0-form-field--error :deep(input:focus),
.hook0-form-field--error :deep(select:focus),
.hook0-form-field--error :deep(textarea:focus) {
  border-color: var(--color-danger);
  box-shadow: 0 0 0 1px var(--color-danger);
}
</style>
