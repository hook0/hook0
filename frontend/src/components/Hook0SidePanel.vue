<script setup lang="ts">
import { X } from 'lucide-vue-next';

/**
 * Hook0SidePanel - A slide-out side panel component
 *
 * Displays content in a panel that slides in from the side of the screen.
 * Commonly used for detail views, forms, or settings.
 */

interface Props {
  open: boolean;
  title?: string;
  side?: 'left' | 'right';
  width?: string;
}

withDefaults(defineProps<Props>(), {
  title: undefined,
  side: 'right',
  width: '400px',
});

const emit = defineEmits<{
  close: [];
}>();

function handleClose() {
  emit('close');
}

function handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    handleClose();
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="backdrop">
      <div v-if="open" class="hook0-side-panel-backdrop" @click="handleBackdropClick">
        <Transition :name="`slide-${side}`">
          <div
            v-if="open"
            class="hook0-side-panel"
            :class="`hook0-side-panel--${side}`"
            :style="{ width }"
            role="dialog"
            aria-modal="true"
          >
            <div class="hook0-side-panel__header">
              <h2 v-if="title" class="hook0-side-panel__title">{{ title }}</h2>
              <slot name="header" />
              <button
                class="hook0-side-panel__close"
                type="button"
                aria-label="Close panel"
                @click="handleClose"
              >
                <X :size="20" aria-hidden="true" />
              </button>
            </div>

            <div class="hook0-side-panel__content">
              <slot />
            </div>

            <div v-if="$slots.footer" class="hook0-side-panel__footer">
              <slot name="footer" />
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.hook0-side-panel-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
}

.hook0-side-panel {
  position: fixed;
  top: 0;
  bottom: 0;
  z-index: 50;
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg-primary);
  box-shadow: var(--shadow-xl);
}

.hook0-side-panel--left {
  left: 0;
  border-right: 1px solid var(--color-border);
}

.hook0-side-panel--right {
  right: 0;
  border-left: 1px solid var(--color-border);
}

.hook0-side-panel__header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-side-panel__title {
  flex: 1;
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-side-panel__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  padding: 0;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.hook0-side-panel__close:hover {
  background-color: var(--color-bg-secondary);
  color: var(--color-text-primary);
}

.hook0-side-panel__content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.hook0-side-panel__footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
}

/* Transitions */
.backdrop-enter-active,
.backdrop-leave-active {
  transition: opacity 0.2s ease;
}

.backdrop-enter-from,
.backdrop-leave-to {
  opacity: 0;
}

.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.3s ease;
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
}

.slide-left-enter-active,
.slide-left-leave-active {
  transition: transform 0.3s ease;
}

.slide-left-enter-from,
.slide-left-leave-to {
  transform: translateX(-100%);
}
</style>
