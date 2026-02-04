<script setup lang="ts">
/**
 * Hook0SkeletonGroup - A wrapper for displaying multiple skeleton loaders
 *
 * DESIGN SYSTEM: Uses semantic variants instead of CSS values.
 * Provides consistent spacing and layout for skeleton loading states.
 */
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

type Direction = 'column' | 'row';
type Gap = 'sm' | 'md' | 'lg';

/**
 * Semantic variants for different loading contexts
 */
type SkeletonGroupVariant =
  | 'lines' // Standard text lines (default)
  | 'form' // Form fields with labels
  | 'table' // Table rows
  | 'cards'; // Card placeholders

interface Props {
  count?: number;
  direction?: Direction;
  gap?: Gap;
  /** Semantic variant for the skeleton group */
  variant?: SkeletonGroupVariant;
  /** Whether the last line should be truncated */
  lastLineTruncated?: boolean;
}

withDefaults(defineProps<Props>(), {
  count: 3,
  direction: 'column',
  gap: 'md',
  variant: 'lines',
  lastLineTruncated: true,
});

const gapMap: Record<Gap, string> = {
  sm: '0.5rem',
  md: '0.75rem',
  lg: '1rem',
};

// Map variant to skeleton size
const variantToSize: Record<SkeletonGroupVariant, 'text' | 'text-sm' | 'hero' | 'block'> = {
  lines: 'text',
  form: 'text',
  table: 'text-sm',
  cards: 'block',
};
</script>

<template>
  <div
    class="hook0-skeleton-group"
    :style="{
      '--skeleton-gap': gapMap[gap],
      '--skeleton-direction': direction,
    }"
    aria-hidden="true"
  >
    <slot>
      <Hook0Skeleton
        v-for="line in count"
        :key="line"
        :size="line === count && lastLineTruncated ? 'text-truncated' : variantToSize[variant]"
      />
    </slot>
  </div>
</template>

<style scoped>
.hook0-skeleton-group {
  display: flex;
  flex-direction: var(--skeleton-direction);
  gap: var(--skeleton-gap);
  padding: 1rem;
}
</style>
