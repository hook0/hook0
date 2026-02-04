<script setup lang="ts">
import { computed } from 'vue';

interface Tab {
  id: string;
  label: string;
  icon?: string;
}

interface Props {
  tabs: Tab[];
}

defineProps<Props>();

const model = defineModel<string>({ required: true });

const activeIndex = computed(() => model.value);
</script>

<template>
  <div class="hook0-tabs" role="tablist">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      role="tab"
      class="hook0-tab"
      :class="{ active: activeIndex === tab.id }"
      :aria-selected="activeIndex === tab.id"
      :tabindex="activeIndex === tab.id ? 0 : -1"
      @click="model = tab.id"
      @keydown.right.prevent="
        () => {
          const idx = tabs.findIndex((t) => t.id === model);
          model = tabs[(idx + 1) % tabs.length].id;
        }
      "
      @keydown.left.prevent="
        () => {
          const idx = tabs.findIndex((t) => t.id === model);
          model = tabs[(idx - 1 + tabs.length) % tabs.length].id;
        }
      "
    >
      {{ tab.label }}
    </button>
  </div>
</template>

<style scoped>
.hook0-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 1rem;
}

.hook0-tab {
  padding: 0.625rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition:
    color 0.15s ease,
    border-color 0.15s ease;
  margin-bottom: -1px;
}

.hook0-tab:hover {
  color: var(--color-text-primary);
}

.hook0-tab.active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.hook0-tab:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
  border-radius: var(--radius-sm);
}
</style>
