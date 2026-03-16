<script setup lang="ts">
import { computed } from 'vue';
import { Check } from 'lucide-vue-next';
import type { Step } from '@/pages/tutorial/TutorialService';
import Hook0Button from './Hook0Button.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type Props = {
  steps: Step[];
};

const props = defineProps<Props>();

const stepState = (index: number): 'completed' | 'next' | 'future' => {
  if (props.steps[index].isCompleted) return 'completed';
  if (index === 0) return 'next';
  const allPreviousCompleted = props.steps.slice(0, index).every((s) => s.isCompleted);
  return allPreviousCompleted ? 'next' : 'future';
};

const nextStepIndex = computed(() => props.steps.findIndex((s) => !s.isCompleted));
</script>

<template>
  <nav class="widget" aria-label="Onboarding progress">
    <ol class="widget__list">
      <li
        v-for="(step, index) in props.steps"
        :key="step.title"
        class="widget__item"
      >
        <!-- Connector line -->
        <div
          v-if="index < props.steps.length - 1"
          class="widget__connector"
          :class="{ 'widget__connector--completed': step.isCompleted }"
        />

        <!-- Circle indicator (z-index above connector) -->
        <span
          class="widget__circle"
          :class="`widget__circle--${stepState(index)}`"
        >
          <Check
            v-if="stepState(index) === 'completed'"
            :size="16"
            aria-hidden="true"
          />
          <span
            v-else-if="stepState(index) === 'next'"
            class="widget__dot"
          />
        </span>

        <!-- Content -->
        <component
          :is="stepState(index) === 'next' && step.route ? Hook0Button : 'div'"
          :to="step.route"
          class="widget__content"
        >
          <span
            class="widget__title"
            :class="`widget__title--${stepState(index)}`"
          >
            {{ t(step.title) }}
          </span>
          <span class="widget__details">{{ t(step.details) }}</span>
        </component>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.widget__list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.widget__item {
  position: relative;
  padding-left: 2.5rem;
  padding-bottom: 1.5rem;
}

.widget__item:last-child {
  padding-bottom: 0;
}

/* Connector line — sits behind the circle */
.widget__connector {
  position: absolute;
  left: 0.9375rem;
  top: 2rem;
  bottom: 0;
  width: 2px;
  background-color: var(--color-border);
}

.widget__connector--completed {
  background-color: var(--color-primary);
}

/* Circle — z-index above the connector */
.widget__circle {
  position: absolute;
  left: 0;
  top: 0;
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}

.widget__circle--completed {
  background-color: var(--color-primary);
  color: var(--color-bg-primary);
}

.widget__circle--next {
  border: 2.5px solid var(--color-primary);
  background-color: var(--color-bg-primary);
  box-shadow: 0 0 0 3px var(--color-primary-light);
}

.widget__circle--future {
  border: 2px solid var(--color-border);
  background-color: var(--color-bg-primary);
}

.widget__dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: var(--radius-full);
  background-color: var(--color-primary);
}

/* Content */
.widget__content {
  display: flex;
  flex-direction: column;
  padding-top: 0.25rem;
  text-decoration: none;
}

.widget__title {
  font-size: 0.875rem;
  font-weight: 500;
  line-height: 1.4;
}

.widget__title--completed {
  color: var(--color-text-secondary);
}

.widget__title--next {
  color: var(--color-primary);
  font-weight: 600;
}

.widget__title--next:hover {
  text-decoration: underline;
}

.widget__title--future {
  color: var(--color-text-muted);
}

.widget__details {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  margin-top: 0.125rem;
  line-height: 1.4;
}
</style>
