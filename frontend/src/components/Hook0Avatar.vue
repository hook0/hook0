<script setup lang="ts">
import { computed } from 'vue';

type AvatarSize = 'sm' | 'md' | 'lg' | 'xl';
type AvatarVariant = 'square' | 'rounded';

type Props = {
  name: string;
  size?: AvatarSize;
  variant?: AvatarVariant;
  gradient?: string;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  variant: 'rounded',
  gradient: undefined,
});

const initials = computed(() => {
  const parts = props.name.trim().split(/[\s-]+/);
  if (parts.length >= 2) {
    return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
  }
  return props.name.slice(0, 2).toUpperCase();
});

const avatarClasses = computed(() => [
  'hook0-avatar',
  `hook0-avatar--${props.size}`,
  `hook0-avatar--${props.variant}`,
  { 'hook0-avatar--gradient': props.gradient },
]);

const avatarStyle = computed(() => {
  if (props.gradient) {
    return { background: props.gradient };
  }
  return {};
});
</script>

<template>
  <span :class="avatarClasses" :style="avatarStyle" :aria-label="name" role="img">
    {{ initials }}
  </span>
</template>

<style scoped>
.hook0-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-primary);
  color: var(--color-bg-primary, #ffffff);
  font-weight: 700;
  letter-spacing: 0.02em;
  user-select: none;
  flex-shrink: 0;
}

/* Variant: rounded (default) */
.hook0-avatar--rounded {
  border-radius: var(--radius-full);
}

/* Variant: square with rounded corners */
.hook0-avatar--square {
  border-radius: var(--radius-lg);
}

/* Gradient override */
.hook0-avatar--gradient {
  background-color: transparent;
}

/* Sizes */
.hook0-avatar--sm {
  width: 1.5rem;
  height: 1.5rem;
  font-size: 0.5rem;
}

.hook0-avatar--md {
  width: 2rem;
  height: 2rem;
  font-size: 0.625rem;
}

.hook0-avatar--lg {
  width: 2.5rem;
  height: 2.5rem;
  font-size: 0.8125rem;
}

.hook0-avatar--xl {
  width: 3rem;
  height: 3rem;
  font-size: 1rem;
}
</style>
