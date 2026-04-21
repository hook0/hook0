<script setup lang="ts">
// Generic range slider with label, formatted display value, and error state. Uses a CSS custom property (--progress) to paint the filled track portion.
import { computed, nextTick, ref, useId } from 'vue';
import { useI18n } from 'vue-i18n';
import { Pencil } from 'lucide-vue-next';
import { parseDuration } from '@/utils/duration';

const { t } = useI18n();

type Props = {
  modelValue: number;
  min: number;
  max: number;
  step?: number;
  label?: string;
  hideLabel?: boolean;
  error?: string;
  formatValue?: (value: number) => string;
  editable?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  step: 1,
  hideLabel: false,
  formatValue: undefined,
  editable: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

// Stable id so <label for> links to <input id> — native label behaviour (click focuses slider, SR announces)
const inputId = useId();

const displayValue = computed(() =>
  props.formatValue ? props.formatValue(props.modelValue) : String(props.modelValue)
);

const editButtonLabel = computed(() =>
  props.label ? t('slider.editValueLabel', { label: props.label }) : t('slider.editValue')
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
const editInputRef = ref<HTMLInputElement | null>(null);

function startEditing() {
  if (!props.editable) {
    return;
  }
  editText.value = displayValue.value;
  editError.value = false;
  editErrorMessage.value = '';
  isEditing.value = true;
  // Wait for the input to mount, then select its content so the user can overwrite or refine
  void nextTick(() => {
    editInputRef.value?.focus();
    editInputRef.value?.select();
  });
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
    editErrorMessage.value = t('slider.rangeError', {
      min: props.formatValue?.(props.min) ?? props.min,
      max: props.formatValue?.(props.max) ?? props.max,
    });
    return;
  }
  emit('update:modelValue', parsed);
  isEditing.value = false;
  editError.value = false;
}

// On blur, close the edit regardless of validity — an invalid entry would otherwise strand
// the user in edit mode after clicking away. Valid values commit; invalid values revert silently.
function handleBlur() {
  const parsed = parseDuration(editText.value);
  if (parsed !== null && parsed >= props.min && parsed <= props.max) {
    emit('update:modelValue', parsed);
  }
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
    <div v-if="label && !hideLabel" class="hook0-slider__header">
      <label :for="inputId" class="hook0-slider__label">{{ label }}</label>
      <div v-if="isEditing" class="hook0-slider__edit-wrapper">
        <input
          ref="editInputRef"
          v-model="editText"
          class="hook0-slider__edit-input"
          :class="{ 'hook0-slider__edit-input--error': editError }"
          :aria-label="label"
          @keydown.enter="confirmEdit"
          @keydown.escape="cancelEdit"
          @blur="handleBlur"
        />
        <p v-if="editError" class="hook0-slider__edit-error">{{ editErrorMessage }}</p>
      </div>
      <button
        v-else-if="editable"
        type="button"
        class="hook0-slider__value hook0-slider__value--editable"
        :aria-label="editButtonLabel"
        @click="startEditing"
      >
        <span class="hook0-slider__value-text">{{ displayValue }}</span>
        <Pencil class="hook0-slider__edit-icon" :size="12" aria-hidden="true" />
      </button>
      <span v-else class="hook0-slider__value">
        <span class="hook0-slider__value-text">{{ displayValue }}</span>
      </span>
    </div>
    <input
      :id="inputId"
      type="range"
      class="hook0-slider__input"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      :style="{ '--progress': progress + '%' }"
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
  gap: 0.25rem;
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
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-primary);
  font-variant-numeric: tabular-nums;
}

.hook0-slider__edit-icon {
  color: var(--color-text-tertiary);
  transition: color 0.15s ease;
}

.hook0-slider__value--editable:hover .hook0-slider__edit-icon,
.hook0-slider__value--editable:focus-visible .hook0-slider__edit-icon {
  color: var(--color-primary);
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

/* Reset native <button> styles so the editable value chip looks like a text hint, not a button. */
.hook0-slider__value--editable {
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  cursor: text;
  border-bottom: 1px dashed var(--color-border);
  color: var(--color-primary);
}

.hook0-slider__value--editable:hover {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.hook0-slider__value--editable:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
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

.hook0-slider__edit-input:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-slider__edit-error {
  font-size: 0.6875rem;
  color: var(--color-error);
  margin: 0;
}
</style>
