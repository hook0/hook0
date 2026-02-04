<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';

/**
 * Hook0Popover - A floating popover component
 *
 * Displays content in a floating panel anchored to a trigger element.
 * Supports multiple placement options and click-outside to close.
 */

interface Props {
  placement?: 'top' | 'bottom' | 'left' | 'right';
  offset?: number;
}

const props = withDefaults(defineProps<Props>(), {
  placement: 'bottom',
  offset: 8,
});

const isOpen = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const popoverRef = ref<HTMLElement | null>(null);

function toggle() {
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    void nextTick(() => positionPopover());
  }
}

function close() {
  isOpen.value = false;
}

function positionPopover() {
  if (!triggerRef.value || !popoverRef.value) return;

  const trigger = triggerRef.value.getBoundingClientRect();
  const popover = popoverRef.value;

  let top = 0;
  let left = 0;

  switch (props.placement) {
    case 'top':
      top = trigger.top - popover.offsetHeight - props.offset;
      left = trigger.left + trigger.width / 2 - popover.offsetWidth / 2;
      break;
    case 'bottom':
      top = trigger.bottom + props.offset;
      left = trigger.left + trigger.width / 2 - popover.offsetWidth / 2;
      break;
    case 'left':
      top = trigger.top + trigger.height / 2 - popover.offsetHeight / 2;
      left = trigger.left - popover.offsetWidth - props.offset;
      break;
    case 'right':
      top = trigger.top + trigger.height / 2 - popover.offsetHeight / 2;
      left = trigger.right + props.offset;
      break;
  }

  popover.style.top = `${top}px`;
  popover.style.left = `${left}px`;
}

function handleClickOutside(event: MouseEvent) {
  if (
    isOpen.value &&
    triggerRef.value &&
    popoverRef.value &&
    !triggerRef.value.contains(event.target as Node) &&
    !popoverRef.value.contains(event.target as Node)
  ) {
    close();
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  window.addEventListener('resize', positionPopover);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  window.removeEventListener('resize', positionPopover);
});

defineExpose({ open: () => (isOpen.value = true), close, toggle });
</script>

<template>
  <div class="hook0-popover-container">
    <div ref="triggerRef" class="hook0-popover-trigger" @click="toggle">
      <slot name="trigger" />
    </div>

    <Teleport to="body">
      <Transition name="popover">
        <div
          v-if="isOpen"
          ref="popoverRef"
          class="hook0-popover"
          :class="`hook0-popover--${placement}`"
          role="dialog"
        >
          <slot :close="close" />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.hook0-popover-container {
  display: inline-block;
}

.hook0-popover-trigger {
  cursor: pointer;
}

.hook0-popover {
  position: fixed;
  z-index: 50;
  min-width: 12rem;
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: 0.5rem;
}

.popover-enter-active,
.popover-leave-active {
  transition: all 0.15s ease;
}

.popover-enter-from,
.popover-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
