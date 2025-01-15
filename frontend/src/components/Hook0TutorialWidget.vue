<script setup lang="ts">
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { Step } from '@/pages/tutorial/TutorialService';

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
</script>

<template>
  <div class="px-4">
    <ol class="relative text-gray-500 border-s border-gray-200">
      <li
        v-for="(step, index) in props.steps"
        :key="step.title"
        class="mb-10 ms-8"
        :class="isNextStep(index) ? 'cursor-pointer' : ''"
        @click="isNextStep(index) && step.action"
      >
        <span
          :class="[
            'absolute flex items-center justify-center w-8 h-8 rounded-full -start-4 ring-4 ring-white',
            step.isActive ? 'bg-green-200' : isNextStep(index) ? 'bg-blue-200' : 'bg-gray-100',
          ]"
        >
          <FontAwesomeIcon v-if="step.icon && !step.isActive" :icon="['fas', step.icon]" />
          <FontAwesomeIcon
            v-else-if="step.isActive"
            :icon="['fas', 'check']"
            class="text-green-500"
          />
          <FontAwesomeIcon v-else :icon="['fas', 'circle']" size="xs" class="text-gray-400" />
        </span>
        <h3 class="font-medium leading-tight">
          {{ step.title }}
          <span
            v-if="isNextStep(index)"
            class="ml-2 inline-flex items-center rounded-md bg-indigo-200 px-2 py-1 text-xs font-medium text-gray-600 ring-1 ring-inset ring-gray-500/10"
            title="Plan: Developer"
            >Next step</span
          >
        </h3>
        <p class="text-sm">{{ step.details }}</p>
      </li>
    </ol>
  </div>
</template>
