<script setup lang="ts">
/**
 * Hook0Illustration - Design system illustration component
 *
 * Displays predefined illustrations with consistent sizing and styling.
 * All illustrations are bundled in the component - no external src prop.
 */
import { computed } from 'vue';

type IllustrationVariant = 'tutorial' | 'empty' | 'error' | 'success';
type IllustrationSize = 'sm' | 'md' | 'lg' | 'hero';

type Props = {
  variant: IllustrationVariant;
  size?: IllustrationSize;
  alt?: string;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'lg',
  alt: 'Illustration',
});

// Map variants to image sources
const illustrationSources: Record<IllustrationVariant, string> = {
  tutorial: '/illustration-tutorial.png',
  empty: '/illustration-empty.png',
  error: '/illustration-error.png',
  success: '/illustration-success.png',
};

const src = computed(() => illustrationSources[props.variant]);
</script>

<template>
  <div class="hook0-illustration" :class="`hook0-illustration--${size}`">
    <img :src="src" :alt="alt" class="hook0-illustration__img" />
  </div>
</template>

<style scoped>
.hook0-illustration {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
}

.hook0-illustration__img {
  object-fit: cover;
  border-radius: var(--radius-lg);
  max-width: 100%;
}

/* Size variants */
.hook0-illustration--sm .hook0-illustration__img {
  height: 8rem;
  width: 60%;
}

.hook0-illustration--md .hook0-illustration__img {
  height: 12rem;
  width: 70%;
}

.hook0-illustration--lg .hook0-illustration__img {
  height: 16rem;
  width: 80%;
}

.hook0-illustration--hero .hook0-illustration__img {
  height: 20rem;
  width: 83%;
}
</style>
