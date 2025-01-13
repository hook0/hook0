<script setup lang="ts">
import { defineProps } from 'vue';

interface ProgressBarItem {
  description: string;
}

interface ProgressBarProps {
  title?: string;
  actual: string;
  items: ProgressBarItem[];
}

const props = defineProps<ProgressBarProps>();

const progressBarPercentage = () => {
  return Number.isNaN(props.actual)
    ? 0
    : ((Number(props.actual) - 1) / (props.items.length - 1)) * 100;
};
</script>

<template>
  <div>
    <p v-if="props.title" class="text-sm font-medium text-gray-900">
      {{ props.title }}
    </p>
    <div class="mt-6 relative" aria-hidden="true">
      <div class="overflow-hidden rounded-full bg-gray-300 h-4 relative">
        <div
          class="h-4 rounded-full bg-indigo-500 transition-all duration-300"
          :style="{ width: progressBarPercentage() + '%' }"
        ></div>
        <template v-for="(_item, index) in props.items" :key="index">
          <div
            class="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 w-12 h-12 flex items-center justify-center rounded-full border-4 bg-white shadow-lg transition-all duration-300"
            :style="{
              left: `${
                index === 0
                  ? 1.5
                  : index === props.items.length - 1
                    ? 98.5
                    : (index / (props.items.length - 1)) * 100
              }%`,
            }"
            :class="{
              'border-indigo-500 bg-indigo-100 text-indigo-600 scale-110':
                Number(props.actual) === index + 1,
              'border-gray-300 text-gray-500': Number(props.actual) !== index + 1,
            }"
          >
            <span class="text-lg font-bold">{{ index + 1 }}</span>
          </div>
        </template>
      </div>
    </div>
    <div
      class="mt-6 grid"
      :class="`grid-cols-${
        props.items?.length || 5
      } justify-between text-sm font-medium text-gray-600 sm:flex`"
    >
      <template v-for="(item, index) in props.items" :key="index">
        <div class="flex items-center justify-center">
          <p
            class="text-center"
            :class="{ 'font-bold text-indigo-600': Number(props.actual) >= index + 1 }"
          >
            {{ item.description }}
          </p>
        </div>
      </template>
    </div>
  </div>
</template>
