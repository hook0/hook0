<script setup lang="ts">
// Generic range slider with label, formatted display value, and error state. Uses a CSS custom property (--progress) to paint the filled track portion.
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { parseDuration } from '@/utils/parseDuration';

const { t } = useI18n();

type Props = {
  modelValue: number;
  min: number;
  max: number;
  step?: number;
  label?: string;
  error?: string;
  formatValue?: (value: number) => string;
  editable?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  step: 1,
  label: undefined,
  error: undefined,
  formatValue: undefined,
  editable: false,
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

// Inline text editing — lets users type exact values like "1h30min" instead of dragging the slider
const isEditing = ref(false);
const editText = ref('');
const editError = ref(false);
const editErrorMessage = ref('');

function startEditing() {
  if (!props.editable) {
    return;
  }
  editText.value = displayValue.value;
  editError.value = false;
  editErrorMessage.value = '';
  isEditing.value = true;
}

function confirmEdit() {
  const parsed = parseDuration(editText.value);
  if (parsed === null) {
    editError.value = true;
    editErrorMessage.value = t('slider.formatHint');
    return;
  }
  if (parsed < props.min || parsed > props.max) {
    editError.value = true;
    editErrorMessage.value = t('slider.rangeError', { min: props.min, max: props.max });
    return;
  }
  emit('update:modelValue', parsed);
  isEditing.value = false;
  editError.value = false;
}

function cancelEdit() {
  isEditing.value = false;
  editError.value = false;
}
</script>

<template>
  <div class="hook0-slider">
    <div v-if="label" class="hook0-slider__header">
      <label class="hook0-slider__label">{{ label }}</label>
      <div v-if="isEditing" class="hook0-slider__edit-wrapper">
        <input
          v-model="editText"
          class="hook0-slider__edit-input"
          :class="{ 'hook0-slider__edit-input--error': editError }"
          autofocus
          @keydown.enter="confirmEdit"
          @keydown.escape="cancelEdit"
        />
        <p v-if="editError" class="hook0-slider__edit-error">{{ editErrorMessage }}</p>
      </div>
      <span
        v-else
        class="hook0-slider__value"
        :class="{ 'hook0-slider__value--editable': editable }"
        @click="startEditing"
      >
        {{ displayValue }}
      </span>
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

.hook0-slider__value--editable {
  cursor: text;
  border-bottom: 1px dashed var(--color-border);
}

.hook0-slider__value--editable:hover {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.hook0-slider__edit-wrapper {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.125rem;
}

.hook0-slider__edit-input {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-primary);
  font-variant-numeric: tabular-nums;
  background: none;
  border: none;
  border-bottom: 2px solid var(--color-primary);
  outline: none;
  width: 6rem;
  text-align: right;
  padding: 0;
}

.hook0-slider__edit-input--error {
  border-bottom-color: var(--color-error);
  color: var(--color-error);
}

.hook0-slider__edit-error {
  font-size: 0.6875rem;
  color: var(--color-error);
  margin: 0;
}
</style>
