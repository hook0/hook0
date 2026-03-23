<script setup lang="ts">
/**
 * Hook0Form - A form wrapper that integrates with VeeValidate
 *
 * Provides consistent form styling, validation handling, and loading states.
 * Use with VeeValidate's useForm() composable and Zod schemas via toTypedSchema().
 */
import { computed } from 'vue';

type Props = {
  disabled?: boolean;
  loading?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  loading: false,
});

const emit = defineEmits<{
  submit: [event: Event];
}>();

defineSlots<{
  default(): unknown;
}>();

const isInteractive = computed(() => !props.disabled && !props.loading);

function handleSubmit(event: Event) {
  event.preventDefault();
  if (isInteractive.value) {
    emit('submit', event);
  }
}
</script>

<template>
  <form
    class="hook0-form"
    :class="{
      'hook0-form--disabled': disabled,
      'hook0-form--loading': loading,
    }"
    :aria-busy="loading"
    @submit="handleSubmit"
  >
    <slot />
  </form>
</template>

<style scoped>
.hook0-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.hook0-form--disabled {
  pointer-events: none;
}

.hook0-form--disabled {
  opacity: 0.6;
}

.hook0-form--loading {
  opacity: 0.8;
}
</style>
