<script setup lang="ts">
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { defineProps } from 'vue';

interface ProgressBarItem {
  icon?: string;
  title: string;
}

interface ProgressBarProps {
  actual: string;
  items: ProgressBarItem[];
}

const props = defineProps<ProgressBarProps>();
</script>

<template>
  <div class="bg-gray-200 h-1 flex items-center justify-between">
    <template v-for="(item, index) in props.items" :key="index">
      <div class="flex items-center">
        <div
          v-if="index + 1 < Number(props.actual)"
          class="bg-indigo-600 h-10 w-10 rounded-full shadow flex items-center justify-center"
        >
          <FontAwesomeIcon :icon="['fas', 'check']" color="white" size="lg"></FontAwesomeIcon>
        </div>
        <div
          v-else-if="index + 1 === Number(props.actual)"
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
          v-if="index + 1 === Number(props.actual)"
          class="mt-20 mb-2 hidden lg:block"
          :class="index !== 0 ? '-ml-24' : '-ml-14'"
        >
          <div class="relative bg-white shadow-lg px-2 py-1 rounded">
            <div class="flex items-center">
              <FontAwesomeIcon
                v-if="item.icon"
                :icon="['fas', item.icon]"
                size="sm"
                class="text-indigo-600"
              ></FontAwesomeIcon>
              <p class="ml-2 text-indigo-600 font-bold">
                ({{ index + 1 }}/{{ props.items.length }})
                {{ item.title }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div
        v-if="index < props.items.length - 1"
        class="flex-grow h-1"
        :class="{
          'bg-indigo-600': index + 1 < Number(props.actual),
          'bg-gray-200': index + 1 >= Number(props.actual),
        }"
      ></div>
    </template>
  </div>
</template>
