<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, useId } from 'vue';
import { onClickOutside } from '@vueuse/core';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type PopoverPosition = 'top' | 'bottom' | 'left' | 'right';

interface Props {
  position?: PopoverPosition;
  ariaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  position: 'bottom',
  ariaLabel: undefined,
});

defineSlots<{
  trigger(props: {
    open: () => void;
    close: () => void;
    toggle: () => void;
    isOpen: boolean;
    ariaExpanded: boolean;
    ariaControls: string;
  }): unknown;
  default(): unknown;
}>();

const popoverId = `hook0-popover-${useId()}`;
const isOpen = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const popoverRef = ref<HTMLElement | null>(null);
const containerRef = ref<HTMLElement | null>(null);

function open() {
  isOpen.value = true;
  void nextTick().then(() => {
    if (popoverRef.value) {
      popoverRef.value.focus();
    }
  });
}

function close() {
  isOpen.value = false;
  void nextTick().then(() => {
    const trigger = triggerRef.value?.querySelector(
      'button, [role="button"], a, [tabindex]'
    ) as HTMLElement | null;
    if (trigger) {
      trigger.focus();
    }
  });
}

function toggle() {
  if (isOpen.value) {
    close();
  } else {
    open();
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    event.preventDefault();
    close();
  }
}

onClickOutside(
  containerRef,
  () => {
    if (isOpen.value) {
      close();
    }
  },
  { detectIframe: true }
);

onMounted(() => {
  document.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div ref="containerRef" class="hook0-popover">
    <div ref="triggerRef" class="hook0-popover__trigger">
      <slot
        name="trigger"
        :open="open"
        :close="close"
        :toggle="toggle"
        :is-open="isOpen"
        :aria-expanded="isOpen"
        :aria-controls="popoverId"
      />
    </div>

    <transition name="hook0-popover-fade">
      <div
        v-if="isOpen"
        :id="popoverId"
        ref="popoverRef"
        class="hook0-popover__content"
        :class="[`hook0-popover__content--${props.position}`]"
        role="dialog"
        :aria-label="props.ariaLabel ?? t('common.popover')"
        tabindex="-1"
      >
        <span
          class="hook0-popover__arrow"
          :class="[`hook0-popover__arrow--${props.position}`]"
          aria-hidden="true"
        />
        <slot />
      </div>
    </transition>
  </div>
</template>

<style scoped>
.hook0-popover {
  position: relative;
  display: inline-flex;
}

.hook0-popover__trigger {
  display: inline-flex;
}

.hook0-popover__content {
  position: absolute;
  background-color: var(--color-bg-elevated);
  color: var(--color-text-primary);
  padding: 0.75rem;
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-lg);
  z-index: var(--z-popover, 70);
  min-width: 12rem;
}

.hook0-popover__content:focus:not(:focus-visible) {
  outline: none;
}

/* Positioning */
.hook0-popover__content--top {
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 0.5rem;
}

.hook0-popover__content--bottom {
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 0.5rem;
}

.hook0-popover__content--left {
  right: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-right: 0.5rem;
}

.hook0-popover__content--right {
  left: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-left: 0.5rem;
}

/* Arrow */
.hook0-popover__arrow {
  position: absolute;
  width: 0;
  height: 0;
  border: 6px solid transparent;
}

.hook0-popover__arrow--top {
  bottom: -12px;
  left: 50%;
  transform: translateX(-50%);
  border-top-color: var(--color-border);
  border-bottom: none;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.05));
}

.hook0-popover__arrow--top::after {
  content: '';
  position: absolute;
  top: -7px;
  left: -5px;
  border: 5px solid transparent;
  border-top-color: var(--color-bg-elevated);
  border-bottom: none;
}

.hook0-popover__arrow--bottom {
  top: -12px;
  left: 50%;
  transform: translateX(-50%);
  border-bottom-color: var(--color-border);
  border-top: none;
  filter: drop-shadow(0 -1px 1px rgba(0, 0, 0, 0.05));
}

.hook0-popover__arrow--bottom::after {
  content: '';
  position: absolute;
  bottom: -7px;
  left: -5px;
  border: 5px solid transparent;
  border-bottom-color: var(--color-bg-elevated);
  border-top: none;
}

.hook0-popover__arrow--left {
  right: -12px;
  top: 50%;
  transform: translateY(-50%);
  border-left-color: var(--color-border);
  border-right: none;
  filter: drop-shadow(1px 0 1px rgba(0, 0, 0, 0.05));
}

.hook0-popover__arrow--left::after {
  content: '';
  position: absolute;
  right: 2px;
  top: -5px;
  border: 5px solid transparent;
  border-left-color: var(--color-bg-elevated);
  border-right: none;
}

.hook0-popover__arrow--right {
  left: -12px;
  top: 50%;
  transform: translateY(-50%);
  border-right-color: var(--color-border);
  border-left: none;
  filter: drop-shadow(-1px 0 1px rgba(0, 0, 0, 0.05));
}

.hook0-popover__arrow--right::after {
  content: '';
  position: absolute;
  left: 2px;
  top: -5px;
  border: 5px solid transparent;
  border-right-color: var(--color-bg-elevated);
  border-left: none;
}

/* Fade transition */
.hook0-popover-fade-enter-active {
  transition: opacity 150ms ease;
}

.hook0-popover-fade-leave-active {
  transition: opacity 100ms ease;
}

.hook0-popover-fade-enter-from,
.hook0-popover-fade-leave-to {
  opacity: 0;
}
</style>
