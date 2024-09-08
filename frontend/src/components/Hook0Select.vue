<script setup lang="ts">
import { omit } from 'ramda';
import { computed, onMounted, onUpdated, ref } from 'vue';

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

interface Props {
  options: Array<Hook0SelectSingleOption | Hook0SelectGroupedOption>;
}

const props = defineProps<Props>();
const simpleOptions = computed(() =>
  (props.options as Hook0SelectSingleOption[]).filter(isSimpleOption)
);
const groupedOptions = computed(() =>
  (props.options as Hook0SelectGroupedOption[]).filter(isGroupedOptions)
);

const select = ref<null | HTMLSelectElement>(null);

function omitOptions($props: Partial<Record<string, unknown>>) {
  return omit(['options'], $props);
}

function isSimpleOption(option: Hook0SelectSingleOption) {
  return option.hasOwnProperty('value') && option.hasOwnProperty('label');
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
  <select
    v-bind="{ ...omitOptions($props), ...$attrs }"
    ref="select"
    v-model="model"
    class="hook0-select"
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
</template>

<style lang="scss" scoped>
.hook0-select {
  @apply block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md;

  &.width-small {
    @apply w-32;
  }
}
</style>
