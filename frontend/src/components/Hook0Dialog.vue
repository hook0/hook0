<script setup lang="ts">
/**
 * Hook0Dialog - Confirmation dialog / modal component
 *
 * Accessible dialog with:
 * - role="dialog", aria-modal="true"
 * - Escape key to close
 * - Focus trap (tab cycles within dialog)
 * - Focus returns to trigger element on close
 * - Backdrop overlay (click to close unless persistent)
 * - Slots: title, default (content), actions
 * - Variants: default, danger (for delete confirmations)
 * - Animations: scale(0.95) opacity(0) -> scale(1) opacity(1) in 200ms
 */
import { ref, watch, nextTick, onBeforeUnmount, useId } from 'vue';
import { X } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';
import { useFocusTrap } from '@/composables/useFocusTrap';

type DialogVariant = 'default' | 'danger';

interface Props {
  open: boolean;
  variant?: DialogVariant;
  title?: string;
  persistent?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  title: undefined,
  persistent: false,
});

const emit = defineEmits<{
  close: [];
  confirm: [];
}>();

defineSlots<{
  title(): unknown;
  default(): unknown;
  actions(): unknown;
}>();

const { t } = useI18n();

const dialogId = useId();
const dialogRef = ref<HTMLElement | null>(null);

function emitClose() {
  emit('close');
}

const { activate, deactivate, handleKeydown } = useFocusTrap(dialogRef, { onEscape: emitClose });

function emitConfirm() {
  emit('confirm');
}

function handleBackdropClick(event: MouseEvent) {
  if (props.persistent) return;
  if (event.target === event.currentTarget) {
    emitClose();
  }
}

/**
 * When the dialog opens, move focus into the dialog.
 * When it closes, restore focus.
 */
watch(
  () => props.open,
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
  // Restore focus if component unmounts while open
  if (props.open) {
    deactivate();
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog-overlay">
      <div v-if="open" class="hook0-dialog__backdrop" @click="handleBackdropClick">
        <Transition name="dialog" appear>
          <div
            v-if="open"
            ref="dialogRef"
            class="hook0-dialog"
            :class="{ 'hook0-dialog--danger': variant === 'danger' }"
            role="dialog"
            aria-modal="true"
            :aria-labelledby="dialogId + '-title'"
            tabindex="-1"
            @keydown="handleKeydown"
          >
            <!-- Header -->
            <div class="hook0-dialog__header">
              <h2 :id="dialogId + '-title'" class="hook0-dialog__title">
                <slot name="title">{{ title }}</slot>
              </h2>
              <Hook0Button
                variant="ghost"
                size="sm"
                class="hook0-dialog__close"
                :aria-label="t('common.close')"
                @click="emitClose"
              >
                <X :size="18" aria-hidden="true" />
              </Hook0Button>
            </div>

            <!-- Content -->
            <div class="hook0-dialog__content">
              <slot />
            </div>

            <!-- Actions -->
            <div class="hook0-dialog__actions">
              <slot name="actions">
                <Hook0Button variant="secondary" @click="emitClose">
                  {{ t('common.cancel') }}
                </Hook0Button>
                <Hook0Button
                  :variant="variant === 'danger' ? 'danger' : 'primary'"
                  @click="emitConfirm"
                >
                  {{ variant === 'danger' ? t('common.delete') : t('common.confirm') }}
                </Hook0Button>
              </slot>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.hook0-dialog__backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal, 60);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background-color: var(--color-overlay, rgba(0, 0, 0, 0.5));
  backdrop-filter: blur(2px);
}

.hook0-dialog {
  position: relative;
  width: 100%;
  max-width: 28rem;
  max-height: calc(100vh - 2rem);
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
  outline: none;
  overflow: hidden;
}

/* Danger variant: subtle red accent on the header border */
.hook0-dialog--danger .hook0-dialog__header {
  border-bottom-color: var(--color-error-light);
}

.hook0-dialog--danger .hook0-dialog__title {
  color: var(--color-error);
}

.hook0-dialog__header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-dialog__title {
  flex: 1;
  margin: 0;
  font-size: 1.0625rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--color-text-primary);
}

.hook0-dialog__close {
  flex-shrink: 0;
}

.hook0-dialog__content {
  padding: 1.5rem;
  overflow-y: auto;
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--color-text-secondary);
}

.hook0-dialog__actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-bg-secondary);
}
</style>
