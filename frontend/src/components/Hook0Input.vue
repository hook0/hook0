<script setup lang="ts">
import { omit } from 'ramda';
import { onMounted, onUpdated, ref, useAttrs, useSlots } from 'vue';

import Hook0Text from '@/components/Hook0Text.vue';

interface Props {
  autofocus?: boolean;
}
const props = defineProps<Props>();

defineOptions({
  inheritAttrs: false,
});
const emit = defineEmits(['update:modelValue']);
defineSlots<{
  helpText(): unknown;
}>();

const ipt = ref<null | HTMLInputElement>(null);

function hasSlot(name: string): boolean {
  return !!useSlots()[name];
}

function _internalState() {
  // checkbox needs special care
  if (
    useAttrs().type === 'checkbox' &&
    typeof useAttrs().value === 'boolean' &&
    ipt.value !== null
  ) {
    ipt.value.checked = useAttrs().value as boolean;
  }
}

onMounted(() => {
  _internalState();

  if (props.autofocus ?? false) {
    console.log('focus');
    ipt.value?.focus();
  }
});

onUpdated(() => {
  _internalState();
});
</script>

<template>
  <div :class="$attrs.class">
    <input
      ref="ipt"
      v-bind="{ ...omit(['class', 'style'], $props), ...$attrs }"
      class="hook0-input"
      :value="$attrs.modelValue"
      @input="(e: Event) => emit('update:modelValue', (e.target as HTMLInputElement)?.value)"
    />

    <div v-if="hasSlot('helpText')">
      <Hook0Text class="helpText">
        <slot name="helpText"></slot>
      </Hook0Text>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.hook0-input {
  @apply block w-full shadow-sm focus:ring-indigo-500 focus:border-indigo-500 text-sm border-gray-300 rounded-md;
}

.hook0-input[type='checkbox'] {
  @apply focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded;
}
</style>
