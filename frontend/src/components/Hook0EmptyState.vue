<script setup lang="ts">
import { computed } from 'vue';
import { Inbox } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

type Props = {
  title?: string;
  description?: string;
};

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  description: undefined,
});

const resolvedTitle = computed(() => props.title ?? t('common.noData'));
const resolvedDescription = computed(() => props.description ?? t('common.getStarted'));

defineSlots<{
  illustration(): unknown;
  icon(): unknown;
  action(): unknown;
  code(): unknown;
}>();
</script>

<template>
  <div class="hook0-empty-state">
    <!-- Custom illustration slot (for complex animations like Error404) -->
    <div v-if="$slots.illustration" class="hook0-empty-state-illustration">
      <slot name="illustration" />
    </div>
    <!-- Standard icon slot -->
    <div v-else class="hook0-empty-state-icon">
      <slot name="icon">
        <Inbox :size="48" aria-hidden="true" />
      </slot>
    </div>
    <h3 class="hook0-empty-state-title">{{ resolvedTitle }}</h3>
    <p class="hook0-empty-state-description">{{ resolvedDescription }}</p>
    <div v-if="$slots.code" class="hook0-empty-state-code">
      <slot name="code" />
    </div>
    <div v-if="$slots.action" class="hook0-empty-state-action">
      <slot name="action" />
    </div>
  </div>
</template>

<style scoped>
.hook0-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 3rem 1.5rem;
  text-align: center;
  gap: 0.75rem;
}

.hook0-empty-state-icon {
  color: var(--color-text-muted);
  opacity: 0.5;
}

.hook0-empty-state-illustration {
  margin-bottom: 1rem;
}

.hook0-empty-state-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-empty-state-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  max-width: 24rem;
}

.hook0-empty-state-code {
  margin-top: 0.5rem;
  width: 100%;
  max-width: 32rem;
  text-align: left;
}

.hook0-empty-state-action {
  margin-top: 0.5rem;
}
</style>
