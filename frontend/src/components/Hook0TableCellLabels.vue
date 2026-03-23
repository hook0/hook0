<script setup lang="ts">
import Hook0Badge from './Hook0Badge.vue';

type Props = {
  value: Record<string, string>;
};

defineProps<Props>();
</script>

<template>
  <div class="cell-labels">
    <Hook0Badge v-for="(val, key) in value" :key="key" variant="default" size="sm">
      {{ key }}={{ val }}
    </Hook0Badge>
  </div>
</template>

<style scoped>
.cell-labels {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  max-width: 16rem;
}
</style>
