<script setup lang="ts">
import { computed } from 'vue';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';

// Search for icons here: https://fontawesome.com/search?m=free&s=solid
// and don't forgot to import in main.ts (see "library.add(..."), the icon you want in order to keep the js bundle size small

interface Props {
  name: string;
  title?: string;
}

const props = defineProps<Props>();
const namesStd = computed(() => props.name.split(' ').map((x) => x.replace(/^fa-/, '')));
</script>

<template>
  <span :title="title">
    <FontAwesomeIcon
      fixed-width
      v-bind="{ ...$props, ...$attrs }"
      :icon="['fas'].concat(namesStd)"
    ></FontAwesomeIcon>
  </span>
</template>

<style lang="scss" scoped>
.darkmode {
  @apply text-gray-200;
}
</style>
