<script setup lang="ts">
import { Check } from 'lucide-vue-next';
import type { Step } from '@/pages/tutorial/TutorialService';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type Props = {
  steps: Step[];
};

const props = defineProps<Props>();

const isNextStep = (index: number) => {
  if (index === 0) return !props.steps[0].isCompleted;
  for (let i = 0; i < index; i++) {
    if (!props.steps[i].isCompleted) return false;
  }
  return !props.steps[index].isCompleted;
};

const isLastStep = (index: number) => {
  return index === props.steps.length - 1;
};
</script>

<template>
  <nav :aria-label="t('common.progress')">
    <ol role="list" class="widget-list">
      <li v-for="(step, index) in props.steps" :key="step.title" class="widget-step">
        <div
          class="widget-step__connector"
          :class="[
            isLastStep(index)
              ? ''
              : step.isCompleted
                ? 'widget-step__connector--completed'
                : 'widget-step__connector--pending',
          ]"
        ></div>
        <component
          :is="step.route ? 'router-link' : 'div'"
          :to="step.route ?? undefined"
          :class="['widget-step__link', { 'widget-step__link--active': !!step.route }]"
        >
          <div class="widget-step__row">
            <span class="widget-step__icon-wrapper">
              <span
                class="widget-step__icon"
                :class="[
                  step.isCompleted
                    ? 'widget-step__icon--completed'
                    : isNextStep(index)
                      ? 'widget-step__icon--next'
                      : 'widget-step__icon--future',
                ]"
              >
                <span v-if="step.isCompleted">
                  <Check :size="20" color="white" />
                </span>
                <span
                  v-else
                  class="widget-step__dot"
                  :class="isNextStep(index) ? 'widget-step__dot--active' : ''"
                >
                </span>
              </span>
            </span>
            <span class="widget-step__text">
              <span
                class="widget-step__title"
                :class="[
                  step.isCompleted
                    ? 'widget-step__title--completed'
                    : isNextStep(index)
                      ? 'widget-step__title--next'
                      : 'widget-step__title--future',
                ]"
              >
                {{ t(step.title) }}
              </span>
              <span class="widget-step__details">{{ t(step.details) }}</span>
            </span>
          </div>
        </component>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.widget-list {
  overflow: hidden;
  list-style: none;
  margin: 0;
  padding: 0;
}

.widget-step {
  position: relative;
  padding-bottom: 2.5rem;
}

.widget-step__connector {
  position: absolute;
  left: 1rem;
  top: 1rem;
  margin-left: -1px;
  margin-top: 0.125rem;
  height: 100%;
  width: 0.125rem;
}

.widget-step__connector--completed {
  background-color: var(--color-primary);
}

.widget-step__connector--pending {
  background-color: var(--color-border);
}

.widget-step__link {
  position: relative;
}

.widget-step__link--active {
  text-decoration: none;
  color: inherit;
  display: block;
}

.widget-step__row {
  position: relative;
  display: flex;
  align-items: flex-start;
}

.widget-step__icon-wrapper {
  display: flex;
  height: 2.25rem;
  align-items: center;
}

.widget-step__icon {
  position: relative;
  z-index: 1;
  display: flex;
  width: 2rem;
  height: 2rem;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-full);
}

.widget-step__icon--completed {
  background-color: var(--color-primary);
}

.widget-step__icon--next {
  border: 2px solid var(--color-primary);
  background-color: var(--color-bg-primary);
  box-shadow: 0 0 0 3px var(--color-primary-light);
}

.widget-step__icon--future {
  border: 2px solid var(--color-border);
  background-color: var(--color-bg-primary);
}

.widget-step__dot {
  width: 0.625rem;
  height: 0.625rem;
  border-radius: var(--radius-full);
  background-color: transparent;
}

.widget-step__dot--active {
  background-color: var(--color-primary);
}

.widget-step__text {
  margin-left: 1rem;
  display: flex;
  min-width: 0;
  flex-direction: column;
}

.widget-step__title {
  font-size: 0.875rem;
  font-weight: 500;
}

.widget-step__title--completed {
  color: var(--color-text-primary);
}

.widget-step__title--next {
  color: var(--color-primary);
  font-weight: 600;
}

.widget-step__title--future {
  color: var(--color-text-tertiary);
}

.widget-step__details {
  font-size: 0.875rem;
  color: var(--color-text-tertiary);
}
</style>
