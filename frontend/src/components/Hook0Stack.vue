<script setup lang="ts">
/**
 * Hook0Stack - Flexible layout component with flex and grid modes
 *
 * DESIGN SYSTEM: Uses semantic variants instead of CSS values.
 * For grid layout, use gridSize variant instead of minWidth CSS values.
 */
import { computed } from 'vue';

type Layout = 'flex' | 'grid';
type Direction = 'row' | 'column';
type Align = 'start' | 'center' | 'end' | 'stretch' | 'baseline';
type Justify = 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';
type Gap = 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';

/**
 * Semantic grid size variants for different card/item types
 */
type GridSize =
  | 'compact' // Small items (200px min) - tags, chips, small cards
  | 'cards' // Standard cards (280px min) - default
  | 'wide' // Wide cards (360px min) - dashboard cards
  | 'full'; // Full width items (100% min) - single column

type Props = {
  /** Layout mode: flex (default) or grid (auto-fill responsive grid) */
  layout?: Layout;
  /** Flex direction (only for layout="flex") */
  direction?: Direction;
  align?: Align;
  justify?: Justify;
  gap?: Gap;
  wrap?: boolean;
  inline?: boolean;
  responsive?: boolean;
  /** Semantic grid size variant (only for layout="grid") */
  gridSize?: GridSize;
};

const props = withDefaults(defineProps<Props>(), {
  layout: 'flex',
  direction: 'row',
  align: 'stretch',
  justify: 'start',
  gap: 'md',
  wrap: false,
  inline: false,
  responsive: false,
  gridSize: 'cards',
});

defineSlots<{
  default(): unknown;
}>();

const gapMap: Record<Gap, string> = {
  none: '0',
  xs: '0.25rem',
  sm: '0.5rem',
  md: '0.75rem',
  lg: '1rem',
  xl: '1.5rem',
  '2xl': '2rem',
};

const alignMap: Record<Align, string> = {
  start: 'flex-start',
  center: 'center',
  end: 'flex-end',
  stretch: 'stretch',
  baseline: 'baseline',
};

const justifyMap: Record<Justify, string> = {
  start: 'flex-start',
  center: 'center',
  end: 'flex-end',
  between: 'space-between',
  around: 'space-around',
  evenly: 'space-evenly',
};

const gridSizeMap: Record<GridSize, string> = {
  compact: '200px',
  cards: '280px',
  wide: '360px',
  full: '100%',
};

const styleVars = computed(() => ({
  '--stack-direction': props.direction,
  '--stack-align': alignMap[props.align],
  '--stack-justify': justifyMap[props.justify],
  '--stack-gap': gapMap[props.gap],
  '--stack-min-width': gridSizeMap[props.gridSize],
}));

const isGrid = computed(() => props.layout === 'grid');
</script>

<template>
  <div
    class="hook0-stack"
    :class="{
      'hook0-stack--grid': isGrid,
      'hook0-stack--wrap': wrap,
      'hook0-stack--inline': inline,
      'hook0-stack--responsive': responsive,
    }"
    :style="styleVars"
  >
    <slot />
  </div>
</template>

<style scoped>
.hook0-stack {
  display: flex;
  flex-direction: var(--stack-direction);
  align-items: var(--stack-align);
  justify-content: var(--stack-justify);
  gap: var(--stack-gap);
}

.hook0-stack--wrap {
  flex-wrap: wrap;
}

.hook0-stack--inline {
  display: inline-flex;
}

/* Responsive: column on mobile, row on desktop */
.hook0-stack--responsive {
  flex-direction: column;
}

@media (min-width: 640px) {
  .hook0-stack--responsive {
    flex-direction: var(--stack-direction);
  }
}

/* Grid layout mode */
.hook0-stack--grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--stack-min-width), 1fr));
}
</style>
