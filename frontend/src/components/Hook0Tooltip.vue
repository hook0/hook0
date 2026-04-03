<script setup lang="ts">
import { onBeforeUnmount, ref, useId, nextTick } from 'vue';

type TooltipPosition = 'top' | 'bottom' | 'left' | 'right';

type Props = {
  content: string;
  position?: TooltipPosition;
  delay?: number;
};

const props = withDefaults(defineProps<Props>(), {
  position: 'top',
  delay: 200,
});

defineSlots<{
  default(): unknown;
}>();

const tooltipId = `hook0-tooltip-${useId()}`;
const visible = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const tooltipStyle = ref<Record<string, string>>({});
let showTimeout: ReturnType<typeof setTimeout> | null = null;

const TOOLTIP_GAP_PX = 4;

function updatePosition() {
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  const gap = TOOLTIP_GAP_PX;

  switch (props.position) {
    case 'top':
      tooltipStyle.value = {
        left: `${rect.left + rect.width / 2}px`,
        top: `${rect.top - gap}px`,
        transform: 'translate(-50%, -100%)',
      };
      break;
    case 'bottom':
      tooltipStyle.value = {
        left: `${rect.left + rect.width / 2}px`,
        top: `${rect.bottom + gap}px`,
        transform: 'translate(-50%, 0)',
      };
      break;
    case 'left':
      tooltipStyle.value = {
        left: `${rect.left - gap}px`,
        top: `${rect.top + rect.height / 2}px`,
        transform: 'translate(-100%, -50%)',
      };
      break;
    case 'right':
      tooltipStyle.value = {
        left: `${rect.right + gap}px`,
        top: `${rect.top + rect.height / 2}px`,
        transform: 'translate(0, -50%)',
      };
      break;
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    hide();
  }
}

function show() {
  if (showTimeout !== null) return;
  showTimeout = setTimeout(() => {
    visible.value = true;
    document.addEventListener('keydown', onKeydown);
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
  if (showTimeout !== null) {
    clearTimeout(showTimeout);
  }
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
          role="tooltip"
          class="hook0-tooltip__content"
          :style="tooltipStyle"
        >
          {{ content }}
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

/* Arrow */
.hook0-tooltip__arrow {
  position: absolute;
  width: 0;
  height: 0;
  border: 4px solid transparent;
}

.hook0-tooltip__arrow--top {
  bottom: -4px;
  left: 50%;
  transform: translateX(-50%);
  border-top-color: var(--color-text-primary);
  border-bottom: none;
}

.hook0-tooltip__arrow--bottom {
  top: -4px;
  left: 50%;
  transform: translateX(-50%);
  border-bottom-color: var(--color-text-primary);
  border-top: none;
}

.hook0-tooltip__arrow--left {
  right: -4px;
  top: 50%;
  transform: translateY(-50%);
  border-left-color: var(--color-text-primary);
  border-right: none;
}

.hook0-tooltip__arrow--right {
  left: -4px;
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
