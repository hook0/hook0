<script setup lang="ts">
import { watch, ref, onMounted, onUnmounted } from 'vue';
import { X } from 'lucide-vue-next';

interface Props {
  open?: boolean;
  title?: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  close: [];
}>();
defineSlots<{
  default(): unknown;
  footer(): unknown;
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);

function close() {
  emit('close');
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === dialogRef.value) {
    close();
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) {
    e.preventDefault();
    close();
  }
}

watch(
  () => props.open,
  (isOpen) => {
    if (!dialogRef.value) return;
    if (isOpen) {
      dialogRef.value.showModal();
    } else {
      dialogRef.value.close();
    }
  }
);

onMounted(() => {
  document.addEventListener('keydown', onKeydown);
  if (props.open && dialogRef.value) {
    dialogRef.value.showModal();
  }
});

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <dialog
      ref="dialogRef"
      class="hook0-dialog"
      aria-modal="true"
      :aria-label="title"
      @click="onBackdropClick"
    >
      <div class="hook0-dialog-panel">
        <div class="hook0-dialog-header">
          <h2 class="hook0-dialog-title">{{ title }}</h2>
          <button class="hook0-dialog-close" aria-label="Close dialog" type="button" @click="close">
            <X :size="20" aria-hidden="true" />
          </button>
        </div>
        <div class="hook0-dialog-body">
          <slot />
        </div>
        <div v-if="$slots.footer" class="hook0-dialog-footer">
          <slot name="footer" />
        </div>
      </div>
    </dialog>
  </Teleport>
</template>

<style scoped>
.hook0-dialog {
  border: none;
  padding: 0;
  margin: auto;
  max-width: 32rem;
  width: calc(100% - 2rem);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  background-color: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.hook0-dialog::backdrop {
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.hook0-dialog[open] {
  animation: dialog-enter 0.2s ease-out;
}

.hook0-dialog-panel {
  display: flex;
  flex-direction: column;
}

.hook0-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-dialog-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-dialog-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.hook0-dialog-close:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-dialog-body {
  padding: 1.5rem;
}

.hook0-dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-bg-secondary);
  border-radius: 0 0 var(--radius-lg) var(--radius-lg);
}

@keyframes dialog-enter {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-0.5rem);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
