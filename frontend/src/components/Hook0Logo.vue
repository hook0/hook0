<script setup lang="ts">
import { computed } from 'vue';
import { Webhook } from 'lucide-vue-next';

type LogoVariant = 'icon' | 'image' | 'image-white';
type LogoSize = 'sm' | 'md' | 'lg' | 'xl';

type Props = {
  variant?: LogoVariant;
  size?: LogoSize;
  showText?: boolean;
  alt?: string;
};

const props = withDefaults(defineProps<Props>(), {
  variant: 'icon',
  size: 'md',
  showText: true,
  alt: 'Hook0',
});

// Icon sizes for variant="icon"
const iconSizeMap: Record<LogoSize, number> = {
  sm: 20,
  md: 24,
  lg: 32,
  xl: 40,
};

// Image heights for variant="image" / "image-white"
const imageSizeMap: Record<LogoSize, string> = {
  sm: '2rem',
  md: '3rem',
  lg: '4rem',
  xl: '5rem',
};

const iconSize = computed(() => iconSizeMap[props.size]);
const imageHeight = computed(() => imageSizeMap[props.size]);

const imageSrc = computed(() => {
  return props.variant === 'image-white' ? '/logo-white.svg' : '/logo.svg';
});

const isImageVariant = computed(() => props.variant === 'image' || props.variant === 'image-white');
</script>

<template>
  <!-- Image variant -->
  <img
    v-if="isImageVariant"
    :src="imageSrc"
    :alt="alt"
    class="hook0-logo-image"
    :style="{ height: imageHeight }"
  />

  <!-- Icon + text variant -->
  <div v-else class="hook0-logo">
    <Webhook :size="iconSize" aria-hidden="true" />
    <span v-if="showText" class="hook0-logo__text" :class="`hook0-logo__text--${size}`">
      Hook0
    </span>
  </div>
</template>

<style scoped>
.hook0-logo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-primary);
}

.hook0-logo__text {
  font-weight: 700;
  letter-spacing: -0.025em;
  color: var(--color-text-primary);
}

.hook0-logo__text--sm {
  font-size: 0.875rem;
}

.hook0-logo__text--md {
  font-size: 1.125rem;
}

.hook0-logo__text--lg {
  font-size: 1.5rem;
}

.hook0-logo__text--xl {
  font-size: 2rem;
}

.hook0-logo-image {
  width: auto;
  display: block;
  border-radius: var(--radius-lg);
}
</style>
