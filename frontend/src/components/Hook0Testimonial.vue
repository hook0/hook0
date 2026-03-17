<script setup lang="ts">
type Props = {
  quote: string;
  author: string;
  role?: string;
  avatarInitial?: string;
}

const props = withDefaults(defineProps<Props>(), {
  role: undefined,
  avatarInitial: undefined,
});

const initial = props.avatarInitial || props.author.charAt(0).toUpperCase();
</script>

<template>
  <div class="hook0-testimonial">
    <blockquote class="hook0-testimonial__quote">"{{ quote }}"</blockquote>
    <div class="hook0-testimonial__author">
      <div class="hook0-testimonial__avatar">{{ initial }}</div>
      <div class="hook0-testimonial__info">
        <span class="hook0-testimonial__name">{{ author }}</span>
        <span v-if="role" class="hook0-testimonial__role">{{ role }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hook0-testimonial {
  max-width: 28rem;
  margin-left: auto;
  margin-right: auto;
  text-align: center;
}

.hook0-testimonial__quote {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  font-style: italic;
  line-height: 1.6;
  margin-bottom: 1rem;
}

.hook0-testimonial__author {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.hook0-testimonial__avatar {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 9999px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.875rem;
  font-weight: 600;
  color: white;
  background: linear-gradient(135deg, var(--color-primary), #8b5cf6);
}

.hook0-testimonial__info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  text-align: left;
}

.hook0-testimonial__name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.hook0-testimonial__role {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}
</style>
