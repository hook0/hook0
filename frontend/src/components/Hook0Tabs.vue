<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import { type Component } from 'vue';

type Tab = {
  id: string;
  label: string;
  icon?: Component;
};

type Props = {
  tabs: Tab[];
};

const props = defineProps<Props>();

const model = defineModel<string>({ required: true });

const tabRefs = ref<HTMLButtonElement[]>([]);

const activeIndex = computed(() => {
  return props.tabs.findIndex((t) => t.id === model.value);
});

function setTabRef(el: HTMLButtonElement | null, index: number) {
  if (el) {
    tabRefs.value[index] = el;
  }
}

function selectTab(tabId: string) {
  model.value = tabId;
}

function focusTab(index: number) {
  const tab = tabRefs.value[index];
  if (tab) {
    tab.focus();
  }
}

function onKeydown(event: KeyboardEvent) {
  const currentIdx = activeIndex.value;
  const count = props.tabs.length;

  if (count === 0) {
    return;
  }

  let targetIdx = -1;

  switch (event.key) {
    case 'ArrowRight':
      event.preventDefault();
      targetIdx = (currentIdx + 1) % count;
      break;
    case 'ArrowLeft':
      event.preventDefault();
      targetIdx = (currentIdx - 1 + count) % count;
      break;
    case 'Home':
      event.preventDefault();
      targetIdx = 0;
      break;
    case 'End':
      event.preventDefault();
      targetIdx = count - 1;
      break;
    default:
      return;
  }

  selectTab(props.tabs[targetIdx].id);
  void nextTick(() => {
    focusTab(targetIdx);
  });
}

function tabPanelId(tabId: string): string {
  return `hook0-tabpanel-${tabId}`;
}

function tabButtonId(tabId: string): string {
  return `hook0-tab-${tabId}`;
}
</script>

<template>
  <div class="hook0-tabs">
    <div class="hook0-tabs__list" role="tablist">
      <button
        v-for="(tab, index) in tabs"
        :id="tabButtonId(tab.id)"
        :key="tab.id"
        :ref="(el) => setTabRef(el as HTMLButtonElement | null, index)"
        role="tab"
        class="hook0-tabs__tab"
        :class="{ 'hook0-tabs__tab--active': model === tab.id }"
        :aria-selected="model === tab.id"
        :aria-controls="tabPanelId(tab.id)"
        :tabindex="model === tab.id ? 0 : -1"
        @click="selectTab(tab.id)"
        @keydown="onKeydown"
      >
        <component
          :is="tab.icon"
          v-if="tab.icon"
          :size="16"
          aria-hidden="true"
          class="hook0-tabs__tab-icon"
        />
        <span class="hook0-tabs__tab-label">{{ tab.label }}</span>
      </button>
    </div>

    <div
      v-for="tab in tabs"
      :id="tabPanelId(tab.id)"
      :key="tab.id"
      role="tabpanel"
      :aria-labelledby="tabButtonId(tab.id)"
      :hidden="model !== tab.id || undefined"
      class="hook0-tabs__panel"
      :tabindex="0"
    >
      <slot v-if="model === tab.id" :name="tab.id" />
    </div>
  </div>
</template>

<style scoped>
.hook0-tabs {
  display: flex;
  flex-direction: column;
}

.hook0-tabs__list {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 1rem;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.hook0-tabs__list::-webkit-scrollbar {
  display: none;
}

.hook0-tabs__tab {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
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
  white-space: nowrap;
}

.hook0-tabs__tab:hover {
  color: var(--color-text-primary);
}

.hook0-tabs__tab--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.hook0-tabs__tab:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
  border-radius: var(--radius-sm);
}

.hook0-tabs__tab-icon {
  flex-shrink: 0;
}

.hook0-tabs__tab-label {
  display: inline-flex;
  align-items: center;
}

.hook0-tabs__panel {
  outline: none;
}

.hook0-tabs__panel:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.hook0-tabs__panel[hidden] {
  display: none;
}
</style>
