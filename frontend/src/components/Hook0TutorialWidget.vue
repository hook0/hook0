<script setup lang="ts">
import { computed } from 'vue';
import { ArrowRight } from 'lucide-vue-next';
import type { Step } from '@/pages/tutorial/TutorialService';
import Hook0Button from './Hook0Button.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type Props = {
  steps: Step[];
};

const props = defineProps<Props>();

const completedCount = computed(() => props.steps.filter((s) => s.isCompleted).length);

const progressPercent = computed(
  () => (completedCount.value / props.steps.length) * 100
);

const nextStep = computed(() => {
  const idx = props.steps.findIndex((s) => !s.isCompleted);
  return idx >= 0 ? props.steps[idx] : null;
});

const remainingSteps = computed(() => {
  const nextIdx = props.steps.findIndex((s) => !s.isCompleted);
  return nextIdx >= 0 ? props.steps.slice(nextIdx + 1).filter((s) => !s.isCompleted) : [];
});
</script>

<template>
  <nav class="tutorial-widget" aria-label="Onboarding progress">
    <!-- Progress header -->
    <div class="tutorial-widget__status">
      <span class="tutorial-widget__counter">
        {{ completedCount }} {{ t('tutorial.widget.of') }} {{ steps.length }}
        {{ t('tutorial.widget.completed') }}
      </span>
      <div class="tutorial-widget__pills">
        <span
          v-for="(step, index) in steps"
          :key="index"
          class="tutorial-widget__pill"
          :class="{
            'tutorial-widget__pill--completed': step.isCompleted,
            'tutorial-widget__pill--next': !step.isCompleted && nextStep === step,
          }"
        />
      </div>
    </div>

    <!-- Progress bar -->
    <div class="tutorial-widget__track">
      <div class="tutorial-widget__fill" :style="{ width: progressPercent + '%' }" />
    </div>

    <!-- Next action -->
    <Hook0Button
      v-if="nextStep && nextStep.route"
      variant="ghost"
      :to="nextStep.route"
      class="tutorial-widget__action"
    >
      <div class="tutorial-widget__action-icon">
        <component :is="nextStep.icon" v-if="nextStep.icon" :size="16" aria-hidden="true" />
      </div>
      <div class="tutorial-widget__action-content">
        <span class="tutorial-widget__action-title">{{ t(nextStep.title) }}</span>
        <span class="tutorial-widget__action-desc">{{ t(nextStep.details) }}</span>
      </div>
      <ArrowRight :size="14" aria-hidden="true" class="tutorial-widget__action-arrow" />
    </Hook0Button>

    <!-- Remaining steps -->
    <div v-if="remainingSteps.length > 0" class="tutorial-widget__remaining">
      <div
        v-for="step in remainingSteps"
        :key="step.title"
        class="tutorial-widget__remaining-item"
      >
        <span class="tutorial-widget__remaining-dot" />
        {{ t('tutorial.widget.then') }}: {{ t(step.title) }}
      </div>
    </div>
  </nav>
</template>

<style scoped>
.tutorial-widget {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.tutorial-widget__status {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.tutorial-widget__counter {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  font-weight: 500;
}

.tutorial-widget__pills {
  display: flex;
  gap: 0.375rem;
}

.tutorial-widget__pill {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: var(--radius-full);
  background-color: var(--color-border);
}

.tutorial-widget__pill--completed {
  background-color: var(--color-primary);
}

.tutorial-widget__pill--next {
  background-color: var(--color-primary);
  opacity: 0.4;
}

.tutorial-widget__track {
  height: 4px;
  background-color: var(--color-border);
  border-radius: 2px;
  overflow: hidden;
}

.tutorial-widget__fill {
  height: 100%;
  background-color: var(--color-primary);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.tutorial-widget__action {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  text-decoration: none;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
}

.tutorial-widget__action:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.tutorial-widget__action-icon {
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-md);
  background-color: var(--color-primary-light);
  color: var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.tutorial-widget__action-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  text-align: left;
}

.tutorial-widget__action-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.tutorial-widget__action-desc {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tutorial-widget__action-arrow {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.tutorial-widget__remaining {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.tutorial-widget__remaining-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  padding: 0.125rem 0;
}

.tutorial-widget__remaining-dot {
  width: 4px;
  height: 4px;
  border-radius: var(--radius-full);
  background-color: var(--color-border);
  flex-shrink: 0;
}

@media (prefers-reduced-motion: reduce) {
  .tutorial-widget__fill {
    transition: none;
  }
  .tutorial-widget__action {
    transition: none;
  }
}
</style>
