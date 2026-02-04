<script setup lang="ts">
import { computed } from 'vue';

type HeaderVariant = 'default' | 'centered';

interface Props {
  title?: string;
  subtitle?: string;
  variant?: HeaderVariant;
}

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  subtitle: undefined,
  variant: 'default',
});

defineSlots<{
  header(): unknown;
  subtitle(): unknown;
  actions(): unknown;
}>();

const isCentered = computed(() => props.variant === 'centered');
</script>

<template>
  <!-- Centered variant (for auth pages) -->
  <div v-if="isCentered" class="hook0-card-header hook0-card-header--centered">
    <h1 v-if="title" class="hook0-card-header__title--centered">{{ title }}</h1>
    <slot v-else name="header" />
    <p v-if="subtitle" class="hook0-card-header__subtitle--centered">{{ subtitle }}</p>
    <slot v-else name="subtitle" />
  </div>

  <!-- Default variant -->
  <div v-else class="hook0-card-header">
    <div class="hook0-card-header__container">
      <div class="hook0-card-header__left">
        <h3 class="hook0-card-header__title">
          <slot name="header">{{ title }}</slot>
        </h3>
        <p v-if="subtitle || $slots.subtitle" class="hook0-card-header__subtitle">
          <slot name="subtitle">{{ subtitle }}</slot>
        </p>
      </div>
      <div v-if="$slots.actions" class="hook0-card-header__right">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Base styles */
.hook0-card-header {
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-bg-primary);
}

/* Default variant */
.hook0-card-header__container {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
}

.hook0-card-header__title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.125rem;
  line-height: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-card-header__subtitle {
  margin-top: 0.25rem;
  max-width: 42rem;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.hook0-card-header__right {
  display: flex;
  flex-shrink: 0;
  gap: 0.75rem;
}

/* Centered variant */
.hook0-card-header--centered {
  text-align: center;
  border-bottom: none;
  background-color: transparent;
  padding: 1.5rem 1.5rem 0;
  margin-bottom: 1rem;
}

.hook0-card-header__title--centered {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 0.5rem;
}

.hook0-card-header__subtitle--centered {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}
</style>
