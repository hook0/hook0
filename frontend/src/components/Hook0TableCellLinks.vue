<script setup lang="ts">
import { RouteLocation } from 'vue-router';

import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';

defineOptions({
  inheritAttrs: false,
});

interface LinkParams {
  value: string;
  icon?: string;
  onClick?: () => void;
  href?: string;
  to?: RouteLocation;
  disabled?: boolean;
  dataTest?: string;
}

interface Props {
  links: LinkParams[];
}

defineProps<Props>();
</script>

<template>
  <div class="grid grid-flow-col auto-cols-max gap-1">
    <Hook0TableCellLink
      v-for="(link, index) in links"
      :key="index"
      :value="link.value"
      :icon="link.icon"
      :on-click="link.onClick"
      :href="link.href"
      :to="link.to"
      :disabled="link.disabled"
      :data-test="link.dataTest"
    />
  </div>
</template>
