<script setup lang="ts">
import type { Component } from 'vue';

type TrustItem = {
  href: string;
  title: string;
  icon: Component;
};

type Props = {
  label?: string;
  items?: TrustItem[];
};

withDefaults(defineProps<Props>(), {
  label: undefined,
  items: undefined,
});

defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="hook0-trust-section">
    <p v-if="label" class="hook0-trust-section__label">{{ label }}</p>
    <div class="hook0-trust-section__logos">
      <!-- Auto-generated items from props -->
      <a
        v-for="item in items"
        :key="item.href"
        :href="item.href"
        :title="item.title"
        target="_blank"
        rel="noopener noreferrer"
        class="hook0-trust-section__link"
      >
        <component :is="item.icon" :size="24" aria-hidden="true" />
      </a>
      <!-- Slot for custom content -->
      <slot />
    </div>
  </div>
</template>

<style scoped>
.hook0-trust-section {
  text-align: center;
}

.hook0-trust-section__label {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 1rem;
}

.hook0-trust-section__logos {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 2rem;
  flex-wrap: wrap;
}

.hook0-trust-section__logos :deep(a),
.hook0-trust-section__logos :deep(> *) {
  color: var(--color-text-tertiary);
  opacity: 0.6;
  height: 24px;
  display: flex;
  align-items: center;
  transition: opacity 0.15s ease;
}

.hook0-trust-section__logos :deep(a:hover),
.hook0-trust-section__logos :deep(> *:hover) {
  opacity: 1;
}

.hook0-trust-section__logos :deep(svg) {
  height: 24px;
  width: auto;
}

@media (max-width: 640px) {
  .hook0-trust-section__logos {
    gap: 1.5rem;
  }

  .hook0-trust-section__logos :deep(a),
  .hook0-trust-section__logos :deep(> *) {
    height: 18px;
  }

  .hook0-trust-section__logos :deep(svg) {
    height: 18px;
  }
}
</style>
