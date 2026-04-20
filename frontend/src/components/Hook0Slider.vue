<script setup lang="ts">
import { computed, ref, useId, watch } from 'vue';

type Props = {
  label: string;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  helpText?: string;
  error?: string;
};

const props = withDefaults(defineProps<Props>(), {
  step: 1,
  unit: undefined,
  helpText: undefined,
  error: undefined,
});

function clamp(v: number, min: number, max: number): number {
  if (Number.isNaN(v)) return min;
  return Math.min(Math.max(v, min), max);
}

const model = defineModel<number>({ required: true });
const id = `hook0-slider-${useId()}`;

/** Range input writes back through clamp on every change. */
const clampedModel = computed({
  get: () => model.value,
  set: (v) => {
    model.value = clamp(v, props.min, props.max);
  },
});

/**
 * Typing buffer for the number input. Kept separate from the model so partial
 * values like "1" (min=10) aren't clamped mid-typing; clamp fires on commit (blur/change).
 */
const buffer = ref<string>(String(model.value));
watch(model, (v) => {
  buffer.value = String(v);
});

function commitBuffer() {
  const parsed = Number(buffer.value);
  const next = clamp(parsed, props.min, props.max);
  model.value = next;
  buffer.value = String(next);
}
</script>

<template>
  <div class="slider-row">
    <label :for="id" class="slider-row__label">{{ label }}</label>
    <div class="slider-row__inputs">
      <input
        :id="id"
        v-model.number="clampedModel"
        type="range"
        :min="min"
        :max="max"
        :step="step"
        :aria-label="label"
        class="slider-row__range"
      />
      <div class="slider-row__number-wrap">
        <input
          v-model="buffer"
          type="number"
          :min="min"
          :max="max"
          :step="step"
          :aria-label="`${label} (number input)`"
          class="slider-row__number"
          @blur="commitBuffer"
          @change="commitBuffer"
        />
        <span v-if="unit" class="slider-row__unit">{{ unit }}</span>
      </div>
    </div>
    <p v-if="helpText" class="slider-row__help">{{ helpText }}</p>
    <p v-if="error" class="slider-row__error">{{ error }}</p>
  </div>
</template>

<style scoped>
.slider-row {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.slider-row__label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.slider-row__inputs {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.slider-row__range {
  flex: 1;
  min-width: 0;
  height: 2.5rem;
  accent-color: var(--color-primary);
  transition: opacity 0.15s ease;
}

.slider-row__range:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.slider-row__number-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-primary);
  transition: border-color 0.15s ease;
}

.slider-row__number-wrap:focus-within {
  border-color: var(--color-primary);
  outline: 2px solid var(--color-primary);
  outline-offset: 1px;
}

.slider-row__number {
  width: 4.5rem;
  padding: 0;
  border: none;
  background: transparent;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  text-align: right;
}

.slider-row__number:focus {
  outline: none;
}

.slider-row__unit {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.slider-row__help {
  margin: 0;
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.slider-row__error {
  margin: 0;
  font-size: 0.75rem;
  color: var(--color-error);
}

@media (prefers-reduced-motion: reduce) {
  .slider-row__range,
  .slider-row__number-wrap {
    transition: none;
  }
}
</style>
