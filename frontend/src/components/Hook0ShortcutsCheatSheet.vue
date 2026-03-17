<script setup lang="ts">
import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';
import { X } from 'lucide-vue-next';
import { useFocusTrap } from '@/composables/useFocusTrap';

type Shortcut = {
  keys: string[];
  description: string;
}

type ShortcutGroup = {
  title: string;
  shortcuts: Shortcut[];
}

const { t } = useI18n();
const uiStore = useUiStore();

const isMac =
  (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform
    ?.toLowerCase()
    .includes('mac') ??
  navigator.platform?.toUpperCase().includes('MAC') ??
  false;
const metaKey = isMac ? '⌘' : 'Ctrl';

const groups = computed<ShortcutGroup[]>(() => [
  {
    title: t('shortcuts.general'),
    shortcuts: [
      { keys: [metaKey, 'K'], description: t('shortcuts.openCommandPalette') },
      { keys: ['?'], description: t('shortcuts.showShortcuts') },
      { keys: ['Esc'], description: t('shortcuts.closeOverlay') },
    ],
  },
  {
    title: t('shortcuts.navigation'),
    shortcuts: [
      { keys: ['↑', '↓'], description: t('shortcuts.navigateList') },
      { keys: ['Enter'], description: t('shortcuts.selectItem') },
      { keys: ['Tab'], description: t('shortcuts.nextField') },
    ],
  },
]);

const cheatSheetRef = ref<HTMLElement | null>(null);

function close() {
  uiStore.closeShortcutsCheatSheet();
}

const { activate, deactivate, handleKeydown } = useFocusTrap(cheatSheetRef, { onEscape: close });

watch(
  () => uiStore.shortcutsCheatSheetOpen,
  (isOpen) => {
    if (isOpen) {
      void nextTick().then(() => {
        activate();
      });
    } else {
      void nextTick().then(() => {
        deactivate();
      });
    }
  }
);

onBeforeUnmount(() => {
  if (uiStore.shortcutsCheatSheetOpen) {
    deactivate();
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="shortcuts-sheet">
      <div
        v-if="uiStore.shortcutsCheatSheetOpen"
        ref="cheatSheetRef"
        class="shortcuts-overlay"
        @click.self="close"
        @keydown="handleKeydown"
      >
        <div
          class="shortcuts-sheet"
          role="dialog"
          aria-modal="true"
          :aria-label="t('shortcuts.title')"
        >
          <div class="shortcuts-sheet__header">
            <h2 class="shortcuts-sheet__title">{{ t('shortcuts.title') }}</h2>
            <button
              type="button"
              class="shortcuts-sheet__close"
              :aria-label="t('common.close')"
              @click="close"
            >
              <X :size="20" aria-hidden="true" />
            </button>
          </div>

          <div class="shortcuts-sheet__body">
            <div v-for="group in groups" :key="group.title" class="shortcuts-sheet__group">
              <h3 class="shortcuts-sheet__group-title">{{ group.title }}</h3>
              <div
                v-for="shortcut in group.shortcuts"
                :key="shortcut.description"
                class="shortcuts-sheet__row"
              >
                <span class="shortcuts-sheet__description">{{ shortcut.description }}</span>
                <span class="shortcuts-sheet__keys">
                  <kbd v-for="key in shortcut.keys" :key="key" class="shortcuts-sheet__key">
                    {{ key }}
                  </kbd>
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.shortcuts-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal, 60);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-overlay, rgba(0, 0, 0, 0.5));
  backdrop-filter: blur(4px);
}

.shortcuts-sheet {
  width: 100%;
  max-width: 28rem;
  margin: 0 1rem;
  background-color: var(--color-bg-elevated);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  border: 1px solid var(--color-border);
  overflow: hidden;
}

.shortcuts-sheet__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--color-border);
}

.shortcuts-sheet__title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.shortcuts-sheet__close {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem;
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
}

.shortcuts-sheet__close:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-secondary);
}

.shortcuts-sheet__close:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.shortcuts-sheet__body {
  padding: 1rem 1.25rem;
}

.shortcuts-sheet__group {
  margin-bottom: 1.25rem;
}

.shortcuts-sheet__group:last-child {
  margin-bottom: 0;
}

.shortcuts-sheet__group-title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  margin: 0 0 0.5rem;
}

.shortcuts-sheet__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.375rem 0;
}

.shortcuts-sheet__description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.shortcuts-sheet__keys {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.shortcuts-sheet__key {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.5rem;
  padding: 0.125rem 0.375rem;
  font-family: var(--font-sans);
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  box-shadow: 0 1px 0 var(--color-border);
}

/* Transitions */
.shortcuts-sheet-enter-active,
.shortcuts-sheet-leave-active {
  transition: opacity 0.15s ease;
}

.shortcuts-sheet-enter-active .shortcuts-sheet,
.shortcuts-sheet-leave-active .shortcuts-sheet {
  transition: transform 0.15s ease;
}

.shortcuts-sheet-enter-from,
.shortcuts-sheet-leave-to {
  opacity: 0;
}

.shortcuts-sheet-enter-from .shortcuts-sheet {
  transform: scale(0.95);
}

.shortcuts-sheet-leave-to .shortcuts-sheet {
  transform: scale(0.95);
}
</style>
