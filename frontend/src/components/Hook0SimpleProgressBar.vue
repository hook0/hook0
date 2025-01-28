<script setup lang="ts">
interface Props {
  percentage: number;
}

const props = defineProps<Props>();
</script>

<template>
  <div class="w-full bg-gray-200 rounded-lg">
    <div
      class="text-md text-center p-1 leading-none rounded-lg font-extrabold"
      :class="[
        props.percentage < 3 ? 'text-black' : 'text-white',
        props.percentage > 0
          ? props.percentage >= 80
            ? props.percentage >= 100
              ? 'bg-red-500'
              : 'bg-orange-500'
            : 'bg-indigo-500'
          : '',
      ]"
      :style="'width: ' + (props.percentage >= 100 ? 100 : props.percentage) + '%'"
    >
      {{ props.percentage >= 100 ? 100 : props.percentage }}%
    </div>
  </div>
</template>
