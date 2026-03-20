<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router';

type Props = {
  value: string[];
  to?: RouteLocationRaw;
};

withDefaults(defineProps<Props>(), {
  to: undefined,
});
</script>

<template>
  <div class="cell-event-types">
    <RouterLink
      v-for="et in value"
      :key="et"
      :to="to ?? ''"
      class="cell-event-types__item"
      :class="{ 'cell-event-types__item--link': !!to }"
      @click.stop
    >
      {{ et }}
    </RouterLink>
  </div>
</template>

<style scoped>
.cell-event-types {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  max-width: 16rem;
}

.cell-event-types__item {
  flex: 0 0 100%;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
  text-decoration: none;
}

.cell-event-types__item--link:hover {
  color: var(--color-text-primary);
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>
