<script setup lang="ts">
/**
 * Hook0SidePanel - Slide-out right panel for detail views
 *
 * Accessible side panel with:
 * - role="dialog", aria-modal="true"
 * - Escape key to close
 * - Focus trap (tab cycles within panel)
 * - Focus returns to trigger element on close
 * - Backdrop overlay (click to close)
 * - Slots: header, default (content), footer
 * - Slides in from right (250ms ease-out)
 * - Full-screen on mobile
 */
import { ref, watch, nextTick, onBeforeUnmount } from 'vue';
import { X } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';
import { useFocusTrap } from '@/composables/useFocusTrap';

type Props = {
  open: boolean;
  title?: string;
  width?: string;
}

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  width: '32rem',
});

const emit = defineEmits<{
  close: [];
}>();

defineSlots<{
  header(): unknown;
  default(): unknown;
  footer(): unknown;
}>();

const { t } = useI18n();

const panelRef = ref<HTMLElement | null>(null);

function emitClose() {
  emit('close');
}

const { activate, deactivate, handleKeydown } = useFocusTrap(panelRef, { onEscape: emitClose });

function handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emitClose();
  }
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';

      void nextTick().then(() => {
        activate();
      });
    } else {
      document.body.style.overflow = '';
      void nextTick().then(() => {
        deactivate();
      });
    }
  }
);

onBeforeUnmount(() => {
  document.body.style.overflow = '';
  if (props.open) {
    deactivate();
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="side-panel-backdrop">
      <div v-if="open" class="side-panel__backdrop" @click="handleBackdropClick">
        <Transition name="side-panel" appear>
          <div
            v-if="open"
            ref="panelRef"
            class="side-panel"
            :style="{ '--side-panel-width': width }"
            role="dialog"
            aria-modal="true"
            :aria-label="title"
            tabindex="-1"
            data-test="side-panel"
            @keydown="handleKeydown"
          >
            <!-- Header -->
            <div class="side-panel__header">
              <slot name="header">
                <h2 v-if="title" class="side-panel__title">{{ title }}</h2>
              </slot>
              <Hook0Button
                variant="ghost"
                size="sm"
                class="side-panel__close"
                :aria-label="t('common.close')"
                data-test="side-panel-close"
                @click="emit('close')"
              >
                <X :size="18" aria-hidden="true" />
              </Hook0Button>
            </div>

            <!-- Content -->
            <div class="side-panel__content">
              <slot />
            </div>

            <!-- Footer (optional) -->
            <div v-if="$slots.footer" class="side-panel__footer">
              <slot name="footer" />
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.side-panel__backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-overlay, 50);
  display: flex;
  justify-content: flex-end;
  background-color: var(--color-overlay, rgba(0, 0, 0, 0.3));
  backdrop-filter: blur(2px);
}

.side-panel {
  position: relative;
  width: 100%;
  max-width: var(--side-panel-width, 32rem);
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg-primary);
  border-left: 1px solid var(--color-border);
  box-shadow: var(--shadow-xl);
  outline: none;
  overflow: hidden;
}

/* Full-screen on mobile */
@media (max-width: 767px) {
  .side-panel {
    max-width: 100%;
    border-left: none;
  }
}

.side-panel__header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.side-panel__title {
  flex: 1;
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.side-panel__close {
  flex-shrink: 0;
  margin-left: auto;
}

.side-panel__content {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem;
}

.side-panel__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-bg-secondary);
  flex-shrink: 0;
}

/* Backdrop transition */
.side-panel-backdrop-enter-active,
.side-panel-backdrop-leave-active {
  transition: opacity 0.2s ease;
}

.side-panel-backdrop-enter-from,
.side-panel-backdrop-leave-to {
  opacity: 0;
}

/* Panel slide-in transition */
.side-panel-enter-active {
  transition: transform 0.25s ease-out;
}

.side-panel-leave-active {
  transition: transform 0.2s ease-in;
}

.side-panel-enter-from {
  transform: translateX(100%);
}

.side-panel-leave-to {
  transform: translateX(100%);
}
</style>
