<script setup lang="ts">
import { computed, onMounted, onUpdated, useId } from 'vue';

import {
  firstValue,
  Hook0SelectGroupedOption,
  Hook0SelectSingleOption,
  isValidOption,
} from '@/components/Hook0Select';

const model = defineModel<null | string>();
defineOptions({
  inheritAttrs: false,
});

type Props = {
  options: Array<Hook0SelectSingleOption | Hook0SelectGroupedOption>;
  label?: string;
  error?: string;
}

const props = withDefaults(defineProps<Props>(), {
  label: undefined,
  error: undefined,
});

const selectId = `hook0-select-${useId()}`;

const simpleOptions = computed(() =>
  (props.options as Hook0SelectSingleOption[]).filter(isSimpleOption)
);
const groupedOptions = computed(() =>
  (props.options as Hook0SelectGroupedOption[]).filter(isGroupedOptions)
);

function isSimpleOption(option: Hook0SelectSingleOption) {
  return (
    Object.prototype.hasOwnProperty.call(option, 'value') &&
    Object.prototype.hasOwnProperty.call(option, 'label')
  );
}

function isGroupedOptions(option: Hook0SelectGroupedOption) {
  return Array.isArray(option.options) && option.options.every(isSimpleOption);
}

function initValue() {
  if (!isValidOption(props.options, model.value)) {
    model.value = firstValue(props.options);
  }
}

onMounted(() => {
  initValue();
});
onUpdated(() => {
  initValue();
});
</script>

<template>
  <div>
    <label v-if="label" :for="selectId" class="hook0-select-label">{{ label }}</label>
    <select
      :id="selectId"
      v-bind="$attrs"
      v-model="model"
      class="hook0-select"
      :class="{ 'hook0-select-error': error }"
      :aria-invalid="!!error"
      :aria-describedby="error ? `${selectId}-error` : undefined"
    >
      <optgroup v-for="group in groupedOptions" :key="group.label" :label="group.label">
        <option v-for="option in group.options" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </optgroup>

      <option v-for="option in simpleOptions" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
    <p v-if="error" :id="`${selectId}-error`" class="hook0-select-error-text" role="alert">
      {{ error }}
    </p>
  </div>
</template>

<style scoped>
.hook0-select {
  display: block;
  width: 100%;
  padding: 0.5rem 2.5rem 0.5rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-primary);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
  appearance: none;
  background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
  background-position: right 0.5rem center;
  background-repeat: no-repeat;
  background-size: 1.5em 1.5em;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.hook0-select:focus:not(:focus-visible) {
  outline: none;
}

.hook0-select:focus {
  border-color: var(--color-primary);
  box-shadow:
    0 0 0 1px var(--color-primary),
    var(--shadow-sm);
}

.hook0-select.width-small {
  width: 8rem;
}

.hook0-select-error {
  border-color: var(--color-danger);
}

.hook0-select-error-text {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-danger);
}

.hook0-select-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 0.375rem;
}
</style>
