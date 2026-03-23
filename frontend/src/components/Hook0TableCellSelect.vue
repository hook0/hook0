<script setup lang="ts">
import Hook0Select from '@/components/Hook0Select.vue';

type Props = {
  options: Array<{ value: string; label: string }>;
  modelValue: string;
  disabled?: boolean;
  onChange?: (value: string) => void;
};

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
  <div v-else class="hook0-table-cell-select--disabled">
    {{ props.modelValue.charAt(0).toUpperCase() + props.modelValue.slice(1) }}
  </div>
</template>

<style scoped>
.hook0-table-cell-select--disabled {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 0.5rem 2.5rem 0.5rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-muted);
}
</style>
