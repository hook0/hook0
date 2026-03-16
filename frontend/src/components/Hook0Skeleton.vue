<script setup lang="ts">
/**
 * Hook0Skeleton - Loading skeleton placeholder with semantic sizing
 *
 * DESIGN SYSTEM: Uses semantic size variants instead of CSS values.
 * Never use CSS-like props (width, height) - use size variant instead.
 */
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type SkeletonSize =
  | 'text' // Single line of text (1rem height, 100% width)
  | 'text-sm' // Small text line (0.875rem height, 100% width)
  | 'text-truncated' // Truncated text (1rem height, 80% width)
  | 'title' // Section title (1.25rem height, 60% width)
  | 'heading' // Card heading (1.5rem height, 50% width)
  | 'hero' // Large hero text (2rem height, 100% width)
  | 'block' // Multi-line block (4rem height, 100% width)
  | 'avatar' // Circular avatar (2.5rem x 2.5rem)
  | 'avatar-sm' // Small avatar (2rem x 2rem)
  | 'icon' // Icon placeholder (1.25rem x 1.25rem)
  | 'button' // Button placeholder (2.25rem height, 6rem width)
  | 'card-header'; // Card header title (1.25rem height, 12rem width)

type SkeletonVariant = 'default' | 'inline';

interface Props {
  /** Semantic size variant */
  size?: SkeletonSize;
  /** Display variant */
  variant?: SkeletonVariant;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'text',
  variant: 'default',
});

// Size configurations: [width, height, borderRadius]
type SizeConfig = { width: string; height: string; radius: 'md' | 'full' | 'lg' };

const sizeMap: Record<SkeletonSize, SizeConfig> = {
  text: { width: '100%', height: '1rem', radius: 'md' },
  'text-sm': { width: '100%', height: '0.875rem', radius: 'md' },
  'text-truncated': { width: '80%', height: '1rem', radius: 'md' },
  title: { width: '60%', height: '1.25rem', radius: 'md' },
  heading: { width: '50%', height: '1.5rem', radius: 'md' },
  hero: { width: '100%', height: '2rem', radius: 'md' },
  block: { width: '100%', height: '4rem', radius: 'lg' },
  avatar: { width: '2.5rem', height: '2.5rem', radius: 'full' },
  'avatar-sm': { width: '2rem', height: '2rem', radius: 'full' },
  icon: { width: '1.25rem', height: '1.25rem', radius: 'md' },
  button: { width: '6rem', height: '2.25rem', radius: 'md' },
  'card-header': { width: '12rem', height: '1.25rem', radius: 'md' },
};

const config = computed(() => sizeMap[props.size]);
</script>

<template>
  <div
    class="hook0-skeleton"
    :class="[`hook0-skeleton--radius-${config.radius}`, `hook0-skeleton--${variant}`]"
    :style="{ width: config.width, height: config.height }"
    role="status"
    aria-busy="true"
    :aria-label="t('common.loading')"
  />
</template>

<style scoped>
.hook0-skeleton {
  background: linear-gradient(
    90deg,
    var(--color-bg-tertiary) 25%,
    var(--color-bg-secondary) 50%,
    var(--color-bg-tertiary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

.hook0-skeleton--radius-md {
  border-radius: var(--radius-md);
}

.hook0-skeleton--radius-lg {
  border-radius: var(--radius-lg);
}

.hook0-skeleton--radius-full {
  border-radius: 9999px;
}

.hook0-skeleton--inline {
  display: inline-block;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
