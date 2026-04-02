<script setup lang="ts">
import type { Component } from 'vue';
import { Check } from 'lucide-vue-next';

type Props = {
  modelValue: boolean;
  label: string;
  description?: string;
  icon: Component;
  name: string;
  dataTest?: string;
  disabled?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  description: undefined,
  dataTest: undefined,
  disabled: false,
});
defineEmits<{
  'update:modelValue': [value: boolean];
}>();
</script>

<template>
  <label
    class="selectable-card"
    :class="{
      'selectable-card--selected': modelValue,
      'selectable-card--disabled': props.disabled,
    }"
    :data-test="dataTest"
    @click="!props.disabled && $emit('update:modelValue', true)"
  >
    <input
      type="radio"
      :name="name"
      :checked="modelValue"
      :disabled="props.disabled"
      :aria-label="label"
      class="selectable-card__radio"
    />
    <span class="selectable-card__icon" :class="{ 'selectable-card__icon--selected': modelValue }">
      <component :is="icon" :size="18" aria-hidden="true" />
    </span>
    <span class="selectable-card__text">
      <span class="selectable-card__label">{{ label }}</span>
      <span v-if="description" class="selectable-card__description">{{ description }}</span>
    </span>
    <span class="selectable-card__indicator">
      <Check v-if="modelValue" :size="16" aria-hidden="true" />
    </span>
  </label>
</template>

<style scoped>
.selectable-card {
  display: flex;
  align-items: center;
  padding: 1rem 1.25rem;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
  gap: 0.75rem;
}

.selectable-card:hover {
  border-color: var(--color-border-strong);
  background-color: var(--color-bg-secondary);
}

.selectable-card:focus-within {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.selectable-card--selected {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.selectable-card--selected:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.selectable-card--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.selectable-card--disabled:hover {
  border-color: var(--color-border);
  background-color: var(--color-bg-primary);
}

.selectable-card__radio {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.selectable-card__icon {
  flex-shrink: 0;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.selectable-card__icon--selected {
  background-color: var(--color-primary);
  color: var(--color-bg-primary);
}

.selectable-card__text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.selectable-card__label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.selectable-card__description {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.selectable-card__indicator {
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  color: var(--color-primary);
}

@media (prefers-reduced-motion: reduce) {
  .selectable-card {
    transition: none;
  }
}
</style>
