<script setup lang="ts">
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { Step } from '@/pages/tutorial/TutorialService';
import Hook0Button from './Hook0Button.vue';

interface Props {
  steps: Step[];
}

const props = defineProps<Props>();

const isNextStep = (index: number) => {
  if (index === 0) return !props.steps[0].isActive;
  for (let i = 0; i < index; i++) {
    if (!props.steps[i].isActive) return false;
  }
  return !props.steps[index].isActive;
};

const isLastStep = (index: number) => {
  return index === props.steps.length - 1;
};
</script>

<template>
  <nav aria-label="Progress">
    <ol role="list" class="overflow-hidden">
      <li v-for="(step, index) in props.steps" :key="step.title" class="relative pb-10">
        <div
          class="absolute left-4 top-4 -ml-px mt-0.5 h-full w-0.5"
          :class="[isLastStep(index) ? '' : step.isActive ? 'bg-indigo-600' : 'bg-gray-300']"
        ></div>
        <component :is="isNextStep(index) && step.route ? Hook0Button : 'div'" :to="step.route">
          <div class="relative flex items-start">
            <span class="flex h-9 items-center">
              <span
                class="relative z-10 flex size-8 items-center justify-center rounded-full"
                :class="[
                  step.isActive ? 'bg-indigo-600' : 'border-2',
                  step.isActive
                    ? ''
                    : isNextStep(index)
                      ? 'border-indigo-600 bg-white'
                      : 'border-gray-300 bg-white',
                ]"
              >
                <span v-if="step.isActive">
                  <FontAwesomeIcon :icon="['fas', 'check']" color="white" size="lg" />
                </span>
                <span
                  v-else
                  class="size-2.5 rounded-full"
                  :class="isNextStep(index) ? 'bg-indigo-600' : 'bg-transparent'"
                >
                </span>
              </span>
            </span>
            <span class="ml-4 flex min-w-0 flex-col">
              <span
                class="text-sm font-medium"
                :class="
                  step.isActive
                    ? 'text-gray-800'
                    : isNextStep(index)
                      ? 'text-indigo-600 font-semibold'
                      : 'text-gray-500'
                "
              >
                {{ step.title }}
              </span>
              <span class="text-sm text-gray-500">{{ step.details }}</span>
            </span>
          </div>
        </component>
      </li>
    </ol>
  </nav>
</template>
