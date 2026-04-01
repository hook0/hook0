<script setup lang="ts">
import { onBeforeUnmount, ref, useId, nextTick } from 'vue';

type TooltipPosition = 'top' | 'bottom' | 'left' | 'right';

type Props = {
  content?: string;
  position?: TooltipPosition;
  delay?: number;
  /** Keeps tooltip open on hover so users can interact with its content (e.g. copy button) */
  interactive?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  content: undefined,
  position: 'top',
  delay: 200,
  interactive: false,
});

defineSlots<{
  default(): unknown;
  content(): unknown;
}>();

const tooltipId = `hook0-tooltip-${useId()}`;
const visible = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const tooltipStyle = ref<Record<string, string>>({});
let showTimeout: ReturnType<typeof setTimeout> | null = null;
let hideTimeout: ReturnType<typeof setTimeout> | null = null;

const TOOLTIP_GAP_PX = 8;
const VIEWPORT_PADDING = 8;

function clampHorizontal(left: number, translateX: string): { left: number; translateX: string } {
  if (props.position !== 'top' && props.position !== 'bottom') {
    return { left, translateX };
  }
  // Fallback 200px: tooltip may not be rendered yet on first positioning pass
  const tooltipWidth = tooltipRef.value?.offsetWidth ?? 200;
  const halfWidth = tooltipWidth / 2;
  const viewportWidth = window.innerWidth;

  if (left + halfWidth > viewportWidth - VIEWPORT_PADDING) {
    return { left: viewportWidth - VIEWPORT_PADDING, translateX: '-100%' };
  }
  if (left - halfWidth < VIEWPORT_PADDING) {
    return { left: VIEWPORT_PADDING, translateX: '0%' };
  }
  return { left, translateX };
}

function updatePosition() {
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  const gap = TOOLTIP_GAP_PX;

  let left: number;
  let top: string;
  let translateX: string;
  let translateY: string;

  switch (props.position) {
    case 'top': {
      left = rect.left + rect.width / 2;
      top = `${rect.top - gap}px`;
      translateX = '-50%';
      translateY = '-100%';
      const clamped = clampHorizontal(left, translateX);
      left = clamped.left;
      translateX = clamped.translateX;
      break;
    }
    case 'bottom': {
      left = rect.left + rect.width / 2;
      top = `${rect.bottom + gap}px`;
      translateX = '-50%';
      translateY = '0';
      const clamped = clampHorizontal(left, translateX);
      left = clamped.left;
      translateX = clamped.translateX;
      break;
    }
    case 'left':
      left = rect.left - gap;
      top = `${rect.top + rect.height / 2}px`;
      translateX = '-100%';
      translateY = '-50%';
      break;
    case 'right':
      left = rect.right + gap;
      top = `${rect.top + rect.height / 2}px`;
      translateX = '0';
      translateY = '-50%';
      break;
  }

  tooltipStyle.value = {
    left: `${left}px`,
    top,
    transform: `translate(${translateX}, ${translateY})`,
  };
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    hide();
  }
}

function show() {
  // Cancel pending hide so re-entering during grace period keeps tooltip open
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
  if (showTimeout !== null || visible.value) return;
  showTimeout = setTimeout(() => {
    visible.value = true;
    showTimeout = null;
    document.addEventListener('keydown', onKeydown);
    // Capture phase needed to reposition on scroll of ANY ancestor, not just window
    window.addEventListener('scroll', updatePosition, { capture: true, passive: true });
    window.addEventListener('resize', updatePosition, { passive: true });
    void nextTick(updatePosition);
  }, props.delay);
}

function hide() {
  if (showTimeout !== null) {
    clearTimeout(showTimeout);
    showTimeout = null;
  }
  if (props.interactive) {
    hideTimeout = setTimeout(() => {
      doHide();
      hideTimeout = null;
    }, 150);
  } else {
    doHide();
  }
}

function doHide() {
  if (visible.value) {
    document.removeEventListener('keydown', onKeydown);
    window.removeEventListener('scroll', updatePosition, { capture: true } as EventListenerOptions);
    window.removeEventListener('resize', updatePosition);
  }
  visible.value = false;
}

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown);
  window.removeEventListener('scroll', updatePosition, { capture: true } as EventListenerOptions);
  window.removeEventListener('resize', updatePosition);
  if (showTimeout !== null) clearTimeout(showTimeout);
  if (hideTimeout !== null) clearTimeout(hideTimeout);
});
</script>

<template>
  <div
    ref="triggerRef"
    class="hook0-tooltip"
    @mouseenter="show"
    @mouseleave="hide"
    @focusin="show"
    @focusout="hide"
  >
    <div class="hook0-tooltip__trigger" :aria-describedby="tooltipId">
      <slot />
    </div>

    <Teleport to="body">
      <transition name="hook0-tooltip-fade">
        <div
          v-if="visible"
          :id="tooltipId"
          ref="tooltipRef"
          role="tooltip"
          class="hook0-tooltip__content"
          :class="{ 'hook0-tooltip__content--interactive': interactive }"
          :style="tooltipStyle"
          @mouseenter="interactive && show()"
          @mouseleave="interactive && hide()"
        >
          <slot name="content">{{ content }}</slot>
          <span
            class="hook0-tooltip__arrow"
            :class="[`hook0-tooltip__arrow--${position}`]"
            aria-hidden="true"
          />
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<style scoped>
.hook0-tooltip {
  position: relative;
  display: inline-flex;
}

.hook0-tooltip__trigger {
  display: inline-flex;
}

.hook0-tooltip__trigger:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.hook0-tooltip__content {
  position: fixed;
  background-color: var(--color-text-primary);
  color: var(--color-bg-primary);
  padding: 0.375rem 0.625rem;
  border-radius: var(--radius-md);
  font-size: 0.75rem;
  white-space: pre-line;
  pointer-events: none;
  z-index: var(--z-tooltip, 9999);
  box-shadow: var(--shadow-lg);
}

.hook0-tooltip__content--interactive {
  pointer-events: auto;
  white-space: nowrap;
}

/* Arrow */
.hook0-tooltip__arrow {
  position: absolute;
  width: 0;
  height: 0;
  border: 4px solid transparent;
}

.hook0-tooltip__arrow--top {
  bottom: -8px;
  left: 50%;
  transform: translateX(-50%);
  border-top-color: var(--color-text-primary);
  border-bottom: none;
}

.hook0-tooltip__arrow--bottom {
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  border-bottom-color: var(--color-text-primary);
  border-top: none;
}

.hook0-tooltip__arrow--left {
  right: -8px;
  top: 50%;
  transform: translateY(-50%);
  border-left-color: var(--color-text-primary);
  border-right: none;
}

.hook0-tooltip__arrow--right {
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  border-right-color: var(--color-text-primary);
  border-left: none;
}

/* Fade transition */
.hook0-tooltip-fade-enter-active {
  transition: opacity 100ms ease;
}

.hook0-tooltip-fade-leave-active {
  transition: opacity 75ms ease;
}

.hook0-tooltip-fade-enter-from,
.hook0-tooltip-fade-leave-to {
  opacity: 0;
}
</style>
