<script setup lang="ts">
import { RouteLocationRaw } from 'vue-router';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Button from '@/components/Hook0Button.vue';

interface Props {
  title: string;
  description?: string;
  to?: RouteLocationRaw;
}

withDefaults(defineProps<Props>(), {
  description: undefined,
  to: undefined,
});

const emit = defineEmits<{
  click: [e: MouseEvent];
}>();

defineSlots<{
  icon(): unknown;
}>();

function handleClick(e: MouseEvent) {
  emit('click', e);
}
</script>

<template>
  <Hook0Card class="hook0-action-card">
    <Hook0Button class="hook0-action-card__button" variant="ghost" :to="to" @click="handleClick">
      <div class="hook0-action-card__content">
        <div class="hook0-action-card__icon">
          <slot name="icon" />
        </div>
        <h3 class="hook0-action-card__title">{{ title }}</h3>
        <p v-if="description" class="hook0-action-card__description">{{ description }}</p>
      </div>
    </Hook0Button>
  </Hook0Card>
</template>

<style scoped>
.hook0-action-card {
  border-style: dashed;
  border-width: 2px;
  border-color: var(--color-border);
  background-color: transparent;
  transition: all 0.2s ease;
}

.hook0-action-card:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.hook0-action-card__button {
  width: 100%;
  height: 100%;
  min-height: 12rem;
  padding: 0;
  border: none;
  background: transparent;
}

.hook0-action-card__button:hover {
  background: transparent;
}

.hook0-action-card__content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2rem 1.5rem;
  width: 100%;
  height: 100%;
}

.hook0-action-card__icon {
  width: 3rem;
  height: 3rem;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-tertiary);
  margin-bottom: 0.75rem;
  transition: all 0.15s ease;
}

.hook0-action-card:hover .hook0-action-card__icon {
  background-color: var(--color-primary);
  color: #ffffff;
  transform: scale(1.05);
}

.hook0-action-card__title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 0.25rem;
  transition: color 0.15s ease;
}

.hook0-action-card:hover .hook0-action-card__title {
  color: var(--color-primary);
}

.hook0-action-card__description {
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
}
</style>
