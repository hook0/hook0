<script setup lang="ts">
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { defineProps } from 'vue';

interface ProgressBarItem {
  icon?: string;
  title: string;
}

interface ProgressBarProps {
  current: number;
  items: ProgressBarItem[];
}

const props = defineProps<ProgressBarProps>();
</script>

<template>
  <div class="flex items-center w-11/12 mx-auto">
    <template v-for="(item, index) in props.items" :key="index">
      <div class="flex items-center relative">
        <div
          v-if="index + 1 < props.current"
          class="bg-indigo-600 h-10 w-10 rounded-full shadow flex items-center justify-center"
        >
          <FontAwesomeIcon :icon="['fas', 'check']" color="white" size="lg"></FontAwesomeIcon>
        </div>
        <div
          v-else-if="index + 1 === props.current"
          class="bg-white h-10 w-10 rounded-full shadow flex items-center justify-center border-2 border-indigo-600"
        >
          <FontAwesomeIcon
            :icon="['fas', 'circle']"
            size="sm"
            class="text-indigo-600"
          ></FontAwesomeIcon>
        </div>
        <div v-else class="bg-white h-10 w-10 rounded-full border-2 border-gray-300"></div>

        <div
          class="hidden lg:flex absolute top-12 left-1/2 -translate-x-1/2 items-center w-max bg-white shadow rounded"
        >
          <div class="flex items-center px-3 py-1">
            <FontAwesomeIcon
              v-if="item.icon"
              :icon="['fas', item.icon]"
              size="sm"
              class="text-indigo-600"
            ></FontAwesomeIcon>
            <p class="ml-2 text-indigo-600 font-bold">
              ({{ index + 1 }}/{{ props.items.length }}) {{ item.title }}
            </p>
          </div>
        </div>
      </div>

      <div
        v-if="index < props.items.length - 1"
        class="flex-1 h-1"
        :class="{
          'bg-indigo-600': index + 1 < props.current,
          'bg-gray-300': index + 1 >= props.current,
        }"
      ></div>
    </template>
  </div>
</template>
