<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
});

type Props = {
  label?: string;
  error?: string;
};

defineProps<Props>();

const model = defineModel<boolean>({ default: false });
</script>

<template>
  <div class="hook0-checkbox-wrapper" :class="$attrs.class">
    <label class="hook0-checkbox-label">
      <input
        v-bind="$attrs"
        v-model="model"
        type="checkbox"
        class="hook0-checkbox"
        :class="{ 'hook0-checkbox-error': error }"
        :aria-invalid="!!error"
      />
      <span v-if="label" class="hook0-checkbox-text">{{ label }}</span>
      <slot />
    </label>
    <p v-if="error" class="hook0-checkbox-error-text" role="alert">{{ error }}</p>
  </div>
</template>

<style scoped>
.hook0-checkbox-wrapper {
  display: flex;
  flex-direction: column;
}

.hook0-checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  user-select: none;
}

.hook0-checkbox {
  width: 1rem;
  height: 1rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  accent-color: var(--color-primary);
  cursor: pointer;
  flex-shrink: 0;
}

.hook0-checkbox:focus {
  outline: none;
  box-shadow: 0 0 0 2px var(--color-primary-light);
}

.hook0-checkbox-error {
  border-color: var(--color-danger);
}

.hook0-checkbox-text {
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.hook0-checkbox-error-text {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-danger);
}
</style>
