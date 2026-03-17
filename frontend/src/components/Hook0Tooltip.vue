<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, useId } from 'vue';

type TooltipPosition = 'top' | 'bottom' | 'left' | 'right';

type Props = {
  content: string;
  position?: TooltipPosition;
  delay?: number;
}

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
let showTimeout: ReturnType<typeof setTimeout> | null = null;

function show() {
  showTimeout = setTimeout(() => {
    visible.value = true;
  }, props.delay);
}

function hide() {
  if (showTimeout !== null) {
    clearTimeout(showTimeout);
    showTimeout = null;
  }
  visible.value = false;
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && visible.value) {
    hide();
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown);
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

    <transition name="hook0-tooltip-fade">
      <div
        v-if="visible"
        :id="tooltipId"
        role="tooltip"
        class="hook0-tooltip__content"
        :class="[`hook0-tooltip__content--${position}`]"
      >
        {{ content }}
        <span
          class="hook0-tooltip__arrow"
          :class="[`hook0-tooltip__arrow--${position}`]"
          aria-hidden="true"
        />
      </div>
    </transition>
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
  position: absolute;
  background-color: var(--color-text-primary);
  color: var(--color-bg-primary);
  padding: 0.375rem 0.625rem;
  border-radius: var(--radius-md);
  font-size: 0.75rem;
  white-space: nowrap;
  pointer-events: none;
  z-index: var(--z-tooltip, 80);
}

/* Positioning */
.hook0-tooltip__content--top {
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 0.5rem;
}

.hook0-tooltip__content--bottom {
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 0.5rem;
}

.hook0-tooltip__content--left {
  right: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-right: 0.5rem;
}

.hook0-tooltip__content--right {
  left: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-left: 0.5rem;
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
