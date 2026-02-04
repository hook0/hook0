<script setup lang="ts">
import Hook0Select from '@/components/Hook0Select.vue';

interface Props {
  options: Array<{ value: string; label: string }>;
  modelValue: string;
  disabled?: boolean;
  onChange?: (value: string) => void;
}

const props = defineProps<Props>();

function handleChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;

  if (props.onChange) {
    props.onChange(value);
  }
}
</script>

<template>
  <Hook0Select
    v-if="!props.disabled"
    v-bind="{
      options: props.options,
      modelValue: props.modelValue,
      onChange: handleChange,
      class: $attrs.class,
    }"
  />
  <div v-else class="border-gray-300 border rounded-md pl-3 pr-10 text-gray-400">
    {{ props.modelValue.charAt(0).toUpperCase() + props.modelValue.slice(1) }}
  </div>
</template>
