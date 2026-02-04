<script setup lang="ts">
import { computed } from 'vue';
import Hook0Text from '@/components/Hook0Text.vue';

interface Props {
  type?: 'split' | 'full-width' | 'split-content-component' | 'stacked' | 'columns';
}
const props = defineProps<Props>();
const type = computed(() => props.type ?? 'split');

defineSlots<{
  label(): unknown;
  content(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-content-line" :class="type">
    <dt>
      <Hook0Text class="label">
        <slot name="label"></slot>
      </Hook0Text>
    </dt>
    <dd class="hook0-card-content-line-content">
      <slot name="content"></slot>
    </dd>
  </div>
</template>

<style scoped>
.hook0-card-content-line {
  padding: 1rem 0;
}

@media (min-width: 640px) {
  .hook0-card-content-line {
    padding: 1.25rem 1.5rem;
  }
}

.hook0-card-content-line.stacked {
  grid-template-rows: repeat(2, minmax(0, 1fr));
}

.hook0-card-content-line.columns .hook0-card-content-line-content {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: auto;
  column-gap: 1.75rem;
}

@media (min-width: 640px) {
  .hook0-card-content-line.split,
  .hook0-card-content-line.split-content-component {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1rem;
  }
}

.hook0-card-content-line.full-width .hook0-card-content-line-content {
  padding: 1rem 0;
}

.hook0-card-content-line.split-content-component .hook0-card-content-line-content {
  padding-top: 0;
  margin-top: 0;
}

.hook0-card-content-line-content {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  color: #111827;
}

@media (min-width: 640px) {
  .hook0-card-content-line-content {
    margin-top: 0;
    grid-column: span 2 / span 2;
  }
}
</style>
