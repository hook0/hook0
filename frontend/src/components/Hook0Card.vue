<script setup lang="ts">
import { ref, computed } from 'vue';

type CardVariant = 'default' | 'glow' | 'interactive' | 'dashed';

type Props = {
  variant?: CardVariant;
  as?: 'div' | 'button';
};

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  as: 'div',
});

defineSlots<{
  default(): unknown;
}>();

// Mouse tracking for glow effect
const mouseX = ref('50%');
const mouseY = ref('50%');

function handleMouseMove(event: MouseEvent) {
  if (props.variant !== 'glow') return;

  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = ((event.clientX - rect.left) / rect.width) * 100;
  const y = ((event.clientY - rect.top) / rect.height) * 100;
  mouseX.value = `${x}%`;
  mouseY.value = `${y}%`;
}

const cardClasses = computed(() => ({
  'hook0-card': true,
  'hook0-card--glow': props.variant === 'glow',
  'hook0-card--interactive': props.variant === 'interactive',
  'hook0-card--dashed': props.variant === 'dashed',
}));

const cardStyle = computed(() => {
  if (props.variant === 'glow') {
    return {
      '--mouse-x': mouseX.value,
      '--mouse-y': mouseY.value,
    };
  }
  return {};
});
</script>

<template>
  <component
    :is="as"
    v-bind="$attrs"
    :class="cardClasses"
    :style="cardStyle"
    @mousemove="handleMouseMove"
  >
    <slot />
  </component>
</template>

<style scoped>
.hook0-card {
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-sm);
  overflow: hidden;
  margin-bottom: 1rem;
  text-align: left;
  display: flex;
  flex-direction: column;
}

/* Interactive variant - for clickable cards with hover effects */
.hook0-card--interactive {
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    transform 0.2s ease;
  cursor: default;
}

.hook0-card--interactive:hover {
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-md);
  transform: translateY(-1px);
}

/* Dashed variant - for "create new" cards */
.hook0-card--dashed {
  border-style: dashed;
  border-width: 2px;
  border-color: var(--color-border);
  background-color: transparent;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
  justify-content: center;
}

.hook0-card--dashed:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.hook0-card--dashed:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Glow variant */
.hook0-card--glow {
  position: relative;
  box-shadow: var(--shadow-xl);
  transition: box-shadow 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  margin-bottom: 0;
}

.hook0-card--glow::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  opacity: 0;
  transition: opacity 0.3s ease;
  background: radial-gradient(
    600px circle at var(--mouse-x, 50%) var(--mouse-y, 50%),
    color-mix(in srgb, var(--color-primary) 10%, transparent),
    transparent 40%
  );
  pointer-events: none;
  z-index: 0;
}

.hook0-card--glow:hover::before {
  opacity: 1;
}

.hook0-card--glow:hover {
  border-color: var(--color-border-strong);
  transform: translateY(-2px);
}

/* Ensure content is above the glow effect */
.hook0-card--glow > :deep(*) {
  position: relative;
  z-index: 1;
}
</style>
