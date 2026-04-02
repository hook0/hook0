<script setup lang="ts">
import { useMediaQuery } from '@vueuse/core';

type Props = {
  /** Whether a detail item is selected */
  showDetail: boolean;
  /** Key for the Transition animation on detail swap */
  detailKey?: string;
};

defineProps<Props>();

defineSlots<{
  /** The table/list content */
  list(): unknown;
  /** The detail content */
  detail(): unknown;
  /** Back button content (shown on mobile when detail is visible) */
  back(): unknown;
}>();

const isDesktop = useMediaQuery('(min-width: 768px)');
</script>

<template>
  <div class="hook0-split-layout">
    <!-- Mobile: detail full-width with back button -->
    <template v-if="!isDesktop && showDetail">
      <div class="hook0-split-layout__detail hook0-split-layout__detail--mobile">
        <div class="hook0-split-layout__back">
          <slot name="back" />
        </div>
        <div class="hook0-split-layout__detail-content">
          <slot name="detail" />
        </div>
      </div>
    </template>

    <!-- Mobile: list (no detail selected) -->
    <template v-else-if="!isDesktop">
      <div class="hook0-split-layout__list">
        <slot name="list" />
      </div>
    </template>

    <!-- Desktop: split layout -->
    <template v-else>
      <div class="hook0-split-layout__list">
        <slot name="list" />
      </div>

      <div class="hook0-split-layout__detail">
        <Transition name="split-detail-fade" mode="out-in">
          <div v-if="showDetail" :key="detailKey">
            <slot name="detail" />
          </div>
        </Transition>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* Split layout — list left, detail right on desktop */
.hook0-split-layout {
  margin-top: 1rem;
}

@media (min-width: 768px) {
  .hook0-split-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .hook0-split-layout__list,
  .hook0-split-layout__detail {
    height: calc(100vh - 23rem);
  }
}

.hook0-split-layout__list {
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  min-width: 0;
}

.hook0-split-layout__detail {
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  padding: 1.25rem;
  isolation: isolate;
}

.hook0-split-layout__detail--mobile {
  position: static;
  padding: 0;
}

.hook0-split-layout__back {
  padding: 0.75rem 1.25rem;
  margin-top: 0.5rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-split-layout__detail-content {
  padding: 1.25rem;
}

/* Detail panel fade out → fade in on detail swap */
.split-detail-fade-enter-active {
  transition: opacity 200ms ease;
}

.split-detail-fade-leave-active {
  transition: opacity 100ms ease;
}

.split-detail-fade-enter-from,
.split-detail-fade-leave-to {
  opacity: 0;
}
</style>
