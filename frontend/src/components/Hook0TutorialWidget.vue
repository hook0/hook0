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
    <ol role="list" class="overflow-hidden">
      <li v-for="(step, index) in props.steps" :key="step.title" class="relative pb-10">
        <div
          class="absolute left-4 top-4 -ml-px mt-0.5 h-full w-0.5"
          :class="[isLastStep(index) ? '' : step.isCompleted ? 'bg-indigo-600' : 'bg-gray-300']"
        ></div>
        <component
          :is="isNextStep(index) && step.route ? 'router-link' : 'div'"
          :to="isNextStep(index) && step.route ? step.route : undefined"
          class="relative"
          :style="isNextStep(index) ? 'text-decoration: none; color: inherit; display: block;' : ''"
        >
          <div class="relative flex items-start">
            <span class="flex h-9 items-center">
              <span
                class="relative z-10 flex size-8 items-center justify-center rounded-full"
                :class="[
                  step.isCompleted ? 'bg-indigo-600' : 'border-2',
                  step.isCompleted
                    ? ''
                    : isNextStep(index)
                      ? 'border-indigo-600 bg-white ring-3 ring-indigo-100'
                      : 'border-gray-300 bg-white',
                ]"
              >
                <span v-if="step.isCompleted">
                  <Check :size="20" color="white" />
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
                  step.isCompleted
                    ? 'text-gray-800'
                    : isNextStep(index)
                      ? 'text-indigo-600 font-semibold'
                      : 'text-gray-500'
                "
              >
                {{ t(step.title) }}
              </span>
              <span class="text-sm text-gray-500">{{ t(step.details) }}</span>
            </span>
          </div>
        </component>
      </li>
    </ol>
  </nav>
</template>
