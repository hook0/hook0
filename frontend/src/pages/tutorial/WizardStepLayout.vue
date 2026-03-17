<script setup lang="ts">
import { useI18n } from 'vue-i18n';

import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

import { ArrowRight, X } from 'lucide-vue-next';

type Props = {
  stepNumber?: number;
  title: string;
  showSkip?: boolean;
  continueLabel?: string;
  continueDisabled?: boolean;
};

withDefaults(defineProps<Props>(), {
  stepNumber: undefined,
  showSkip: true,
  continueLabel: undefined,
  continueDisabled: true,
});

const emit = defineEmits<{
  skip: [];
  continue: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="wizard-modal__header">
    <Hook0Stack direction="row" align="center" gap="sm">
      <slot name="header-icon">
        <Hook0Badge v-if="stepNumber != null" display="step" variant="primary">{{
          stepNumber
        }}</Hook0Badge>
      </slot>
      <span id="wizard-step-title" class="wizard-modal__title">{{ title }}</span>
    </Hook0Stack>
    <button
      class="wizard-modal__close"
      type="button"
      :aria-label="t('tutorial.skip')"
      @click="emit('skip')"
    >
      <X :size="18" aria-hidden="true" />
    </button>
  </div>

  <div class="wizard-modal__content">
    <slot />
  </div>

  <div class="wizard-modal__footer">
    <slot name="footer">
      <Hook0Button v-if="showSkip" variant="secondary" type="button" @click="emit('skip')">
        <X :size="16" aria-hidden="true" />
        {{ t('tutorial.skip') }}
      </Hook0Button>
      <Hook0Button
        v-if="continueLabel && !continueDisabled"
        variant="primary"
        type="button"
        @click="emit('continue')"
      >
        {{ continueLabel }}
        <ArrowRight :size="16" aria-hidden="true" />
      </Hook0Button>
    </slot>
  </div>
</template>

<style scoped>
.wizard-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.wizard-modal__title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.wizard-modal__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  border: none;
  background: none;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
  flex-shrink: 0;
}

.wizard-modal__close:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.wizard-modal__close:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.wizard-modal__content {
  padding: 1.5rem;
  overflow-y: auto;
  overscroll-behavior: contain;
  flex: 1;
}

.wizard-modal__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

@media (prefers-reduced-motion: reduce) {
  .wizard-modal__close {
    transition: none;
  }
}
</style>
