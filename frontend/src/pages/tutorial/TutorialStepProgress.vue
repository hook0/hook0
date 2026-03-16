<script setup lang="ts">
import { type Component } from 'vue';
import { Check } from 'lucide-vue-next';

type ProgressStep = {
  icon: Component;
  label: string;
};

type Props = {
  steps: ProgressStep[];
  current: number;
};

const props = defineProps<Props>();

function stepState(index: number): 'completed' | 'current' | 'future' {
  if (index < props.current) return 'completed';
  if (index === props.current) return 'current';
  return 'future';
}
</script>

<template>
  <nav class="step-progress" aria-label="Tutorial progress">
    <ol class="step-progress__list">
      <li
        v-for="(step, index) in steps"
        :key="step.label"
        class="step-progress__item"
      >
        <div
          v-if="index > 0"
          class="step-progress__connector"
          :class="{
            'step-progress__connector--completed': index <= current,
          }"
        />

        <div
          class="step-progress__circle"
          :class="[`step-progress__circle--${stepState(index)}`]"
          :aria-current="index === current ? 'step' : undefined"
          :aria-label="step.label"
        >
          <Check
            v-if="stepState(index) === 'completed'"
            :size="16"
            aria-hidden="true"
          />
          <component
            :is="step.icon"
            v-else
            :size="16"
            aria-hidden="true"
          />
        </div>

        <span
          class="step-progress__label"
          :class="[`step-progress__label--${stepState(index)}`]"
        >
          {{ step.label }}
        </span>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.step-progress {
  width: 100%;
  padding: 0.5rem 0;
}

.step-progress__list {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  list-style: none;
  margin: 0;
  padding: 0;
  gap: 0;
}

.step-progress__item {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
  flex: 1;
  min-width: 0;
}

.step-progress__connector {
  position: absolute;
  top: 1.125rem;
  right: 50%;
  width: 100%;
  height: 2px;
  background-color: var(--color-border);
  z-index: 0;
  transform: translateX(-50%);
}

.step-progress__connector--completed {
  background-color: var(--color-success);
}

.step-progress__circle {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: var(--radius-full);
  border: 2px solid transparent;
  cursor: default;
  flex-shrink: 0;
  transition: background-color 0.15s ease, color 0.15s ease, box-shadow 0.15s ease;
}

.step-progress__circle--completed {
  background-color: var(--color-success);
  color: var(--color-bg-primary);
}

.step-progress__circle--current {
  background-color: var(--color-primary);
  color: var(--color-bg-primary);
  box-shadow: 0 0 0 4px var(--color-primary-light);
}

.step-progress__circle--future {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-muted);
}

.step-progress__label {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
  padding: 0 0.25rem;
}

.step-progress__label--completed {
  color: var(--color-success);
}

.step-progress__label--current {
  color: var(--color-primary);
  font-weight: 600;
}

.step-progress__label--future {
  color: var(--color-text-muted);
}

@media (max-width: 640px) {
  .step-progress__circle {
    width: 1.75rem;
    height: 1.75rem;
  }

  .step-progress__connector {
    top: 0.875rem;
  }

  .step-progress__label {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .step-progress__circle {
    transition: none;
  }
}
</style>
