<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
});

interface Props {
  label?: string;
  error?: string;
}

defineProps<Props>();

const model = defineModel<boolean>({ default: false });
</script>

<template>
  <div class="hook0-switch-wrapper" :class="$attrs.class">
    <label class="hook0-switch-label">
      <button
        v-bind="$attrs"
        type="button"
        role="switch"
        :aria-checked="model"
        :aria-invalid="!!error"
        class="hook0-switch"
        :class="{ 'hook0-switch-on': model, 'hook0-switch-error': error }"
        @click="model = !model"
      >
        <span class="hook0-switch-thumb" :class="{ 'hook0-switch-thumb-on': model }" />
      </button>
      <span v-if="label" class="hook0-switch-text">{{ label }}</span>
      <slot />
    </label>
    <p v-if="error" class="hook0-switch-error-text" role="alert">{{ error }}</p>
  </div>
</template>

<style scoped>
.hook0-switch-wrapper {
  display: flex;
  flex-direction: column;
}

.hook0-switch-label {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.hook0-switch {
  position: relative;
  width: 2.25rem;
  height: 1.25rem;
  padding: 0;
  border: none;
  border-radius: 9999px;
  background-color: var(--color-border);
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color 0.2s ease;
}

.hook0-switch:focus {
  outline: none;
  box-shadow: 0 0 0 2px var(--color-primary-light);
}

.hook0-switch-on {
  background-color: var(--color-primary);
}

.hook0-switch-error {
  box-shadow: 0 0 0 2px var(--color-danger);
}

.hook0-switch-thumb {
  position: absolute;
  top: 0.125rem;
  left: 0.125rem;
  width: 1rem;
  height: 1rem;
  background-color: white;
  border-radius: 9999px;
  box-shadow: var(--shadow-sm);
  transition: transform 0.2s ease;
}

.hook0-switch-thumb-on {
  transform: translateX(1rem);
}

.hook0-switch-text {
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.hook0-switch-error-text {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-danger);
}
</style>
