<script setup lang="ts">
import { defineProps, computed } from 'vue';

interface ProgressBarProps {
  value: number;
  max: number;
  nextValue?: number;
}

const props = defineProps<ProgressBarProps>();

const progressPercentage = computed(() => {
  const percentage = (props.value / props.max) * 100;
  return Math.round(Math.min(Math.max(percentage, 0), 100));
});

const nextProgressPercentage = computed(() => {
  if (props.nextValue === undefined) return null;
  const percentage = (props.nextValue / props.max) * 100;
  return Math.min(Math.max(percentage, 0), 100);
});

const nextProgressBarWidth = computed(() => {
  if (nextProgressPercentage.value === null) return null;
  return nextProgressPercentage.value - progressPercentage.value;
});
</script>

<template>
  <div class="w-full">
    <div class="w-full bg-gray-200 rounded-lg h-4 relative overflow-hidden">
      <div
        class="h-4 transition-all duration-300 ease-in-out flex items-center justify-center relative"
        :style="{ width: `${progressPercentage}%` }"
      >
        <div class="absolute inset-0 bg-gradient-to-r from-indigo-500 to-indigo-600"></div>
        <span v-if="progressPercentage > 5" class="text-sm font-bold text-white z-10 relative">
          {{ progressPercentage }}%
        </span>
      </div>
      <div
        v-if="nextProgressPercentage !== null && nextProgressPercentage > progressPercentage"
        class="absolute top-0 left-0 h-4 bg-indigo-200 opacity-800 transition-all duration-300 ease-in-out"
        :style="{
          width: `${nextProgressBarWidth}%`,
          left: `${progressPercentage}%`,
        }"
      ></div>
    </div>
  </div>
</template>