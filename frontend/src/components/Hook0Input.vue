<script setup lang="ts">
import { computed, onMounted, onUpdated, ref, useAttrs, useId, useSlots } from 'vue';
import { Eye, EyeOff } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

import Hook0HelpText from '@/components/Hook0HelpText.vue';

const { t } = useI18n();

interface Props {
  autofocus?: boolean;
  label?: string;
  error?: string;
  showPasswordToggle?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  autofocus: false,
  label: undefined,
  error: undefined,
  showPasswordToggle: false,
});

defineOptions({
  inheritAttrs: false,
});

const model = defineModel<string | boolean | Date | null>();

defineSlots<{
  helpText(): unknown;
}>();

const generatedId = `hook0-input-${useId()}`;

const attrs = useAttrs();
const inputId = computed(() => (attrs.id as string) || generatedId);
const inputAttrs = computed(() => {
  const { class: _cls, style: _sty, type: _type, ...rest } = attrs;
  return rest;
});

// Password visibility state
const isPasswordVisible = ref(false);

// Compute the actual input type
const computedType = computed<string | undefined>(() => {
  const attrType = attrs.type as string | undefined;
  if (attrType === 'password' && props.showPasswordToggle && isPasswordVisible.value) {
    return 'text';
  }
  return attrType;
});

// Check if this is a password field with toggle
const isPasswordWithToggle = computed(
  () => (attrs.type as string | undefined) === 'password' && props.showPasswordToggle
);

function togglePasswordVisibility() {
  isPasswordVisible.value = !isPasswordVisible.value;
}

const ipt = ref<null | HTMLInputElement>(null);

function hasSlot(name: string): boolean {
  return !!useSlots()[name];
}

function syncCheckbox() {
  if (attrs.type === 'checkbox' && typeof model.value === 'boolean' && ipt.value !== null) {
    ipt.value.checked = model.value;
  }
}

onMounted(() => {
  syncCheckbox();
  if (props.autofocus) {
    ipt.value?.focus();
  }
});

onUpdated(() => {
  syncCheckbox();
});

const modelStr = computed({
  get: () => (model.value == null ? '' : String(model.value)),
  set: (v: string) => {
    model.value = v;
  },
});
</script>

<template>
  <div :class="$attrs.class">
    <label v-if="label" :for="inputId" class="hook0-label">{{ label }}</label>
    <div
      class="hook0-input-wrapper"
      :class="{ 'hook0-input-wrapper--with-toggle': isPasswordWithToggle }"
    >
      <input
        :id="inputId"
        ref="ipt"
        v-bind="inputAttrs"
        v-model="modelStr"
        :type="computedType"
        class="hook0-input"
        :class="{
          'hook0-input-error': error,
          'hook0-input--with-toggle': isPasswordWithToggle,
        }"
        :aria-invalid="!!error"
        :aria-describedby="error ? `${inputId}-error` : undefined"
      />
      <button
        v-if="isPasswordWithToggle"
        type="button"
        class="hook0-input-toggle"
        :aria-label="isPasswordVisible ? t('common.hidePassword') : t('common.showPassword')"
        @click="togglePasswordVisibility"
      >
        <EyeOff v-if="isPasswordVisible" :size="20" aria-hidden="true" />
        <Eye v-else :size="20" aria-hidden="true" />
      </button>
    </div>
    <p
      v-if="error"
      :id="`${inputId}-error`"
      class="hook0-input-error-text"
      role="alert"
    >
      {{ error }}
    </p>
    <Hook0HelpText v-if="hasSlot('helpText')">
      <slot name="helpText" />
    </Hook0HelpText>
  </div>
</template>

<style scoped>
.hook0-input {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-primary);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.hook0-input::placeholder {
  color: var(--color-text-muted, #9ca3af) !important;
  opacity: 1;
}

.hook0-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow:
    0 0 0 1px var(--color-primary),
    var(--shadow-sm);
}

.hook0-input[type='checkbox'] {
  width: 1rem;
  height: 1rem;
  padding: 0;
  border-radius: var(--radius-sm);
  accent-color: var(--color-primary);
}

.hook0-input-error {
  border-color: var(--color-danger);
}

.hook0-input-error:focus {
  border-color: var(--color-danger);
  box-shadow: 0 0 0 1px var(--color-danger);
}

.hook0-input-error-text {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-danger);
}

.hook0-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 0.375rem;
}

/* Input wrapper for password toggle */
.hook0-input-wrapper {
  position: relative;
}

.hook0-input--with-toggle {
  padding-right: 2.75rem;
}

.hook0-input-toggle {
  position: absolute;
  right: 0.75rem;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: color 0.15s ease;
}

.hook0-input-toggle:hover {
  color: var(--color-text-primary);
}

.hook0-input-toggle:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 1px;
}
</style>
