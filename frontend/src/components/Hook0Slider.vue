<script setup lang="ts">
// Generic range slider with label, formatted display value, and error state. Uses a CSS custom property (--progress) to paint the filled track portion.
import { computed } from 'vue';

type Props = {
  modelValue: number;
  min: number;
  max: number;
  step?: number;
  label?: string;
  error?: string;
  formatValue?: (value: number) => string;
};

const props = withDefaults(defineProps<Props>(), {
  step: 1,
  label: undefined,
  error: undefined,
  formatValue: undefined,
});

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

const displayValue = computed(() =>
  props.formatValue ? props.formatValue(props.modelValue) : String(props.modelValue)
);

const range = computed(() => props.max - props.min);
// Feeds the --progress CSS custom property — the linear-gradient uses it to paint filled vs unfilled track
const progress = computed(() =>
  // Guard: if min === max the track is degenerate — default to 0% to avoid NaN
  range.value === 0 ? 0 : ((props.modelValue - props.min) / range.value) * 100
);

function onInput(event: Event) {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', Number(target.value));
}
</script>

<template>
  <div class="hook0-slider">
    <div v-if="label" class="hook0-slider__header">
      <label class="hook0-slider__label">{{ label }}</label>
      <span class="hook0-slider__value">{{ displayValue }}</span>
    </div>
    <input
      type="range"
      class="hook0-slider__input"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      :style="{ '--progress': progress + '%' }"
      :aria-label="label"
      :aria-valuenow="modelValue"
      :aria-valuemin="min"
      :aria-valuemax="max"
      @input="onInput"
    />
    <p v-if="error" class="hook0-slider__error">{{ error }}</p>
  </div>
</template>

<style scoped>
.hook0-slider {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.hook0-slider__header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.hook0-slider__label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-slider__value {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-primary);
  font-variant-numeric: tabular-nums;
}

.hook0-slider__input {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 6px;
  border-radius: var(--radius-full);
  background: linear-gradient(
    to right,
    var(--color-primary) 0%,
    var(--color-primary) var(--progress),
    var(--color-border) var(--progress),
    var(--color-border) 100%
  );
  outline: none;
  cursor: pointer;
}

.hook0-slider__input::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--color-bg-primary);
  border: 2px solid var(--color-primary);
  cursor: pointer;
  transition: box-shadow 0.15s ease;
}

.hook0-slider__input::-webkit-slider-thumb:hover {
  box-shadow: 0 0 0 4px var(--color-primary-light);
}

.hook0-slider__input:focus-visible::-webkit-slider-thumb {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-slider__input::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--color-bg-primary);
  border: 2px solid var(--color-primary);
  cursor: pointer;
}

.hook0-slider__error {
  font-size: 0.8125rem;
  color: var(--color-error);
}
</style>
