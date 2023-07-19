<script setup lang="ts">
import { omit } from 'ramda';
import { computed, onMounted, onUpdated, ref } from 'vue';

import { Hook0SelectGroupedOption, Hook0SelectSingleOption } from '@/components/Hook0Select';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  options: Array<Hook0SelectSingleOption | Hook0SelectGroupedOption>;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);
const simpleOptions = computed(() =>
  (props.options as Hook0SelectSingleOption[]).filter(isSimpleOption)
);
const groupedOptions = computed(() =>
  (props.options as Hook0SelectGroupedOption[]).filter(isGroupedOptions)
);

const select = ref<null | HTMLSelectElement>(null);

function omitOptions($props: Record<string, unknown>) {
  return omit(['options'], $props);
}

function sendEvent() {
  emit('update:modelValue', select.value?.value);
}

function isSimpleOption(option: Hook0SelectSingleOption) {
  return option.hasOwnProperty('value') && option.hasOwnProperty('label');
}

function isGroupedOptions(option: Hook0SelectGroupedOption) {
  return Array.isArray(option.options) && option.options.every(isSimpleOption);
}

onMounted(() => {
  sendEvent();
});

onUpdated(() => {
  sendEvent();
});
</script>

<template>
  <select
    v-bind="{ ...omitOptions($props), ...$attrs }"
    ref="select"
    class="hook0-select"
    @input="sendEvent()"
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
