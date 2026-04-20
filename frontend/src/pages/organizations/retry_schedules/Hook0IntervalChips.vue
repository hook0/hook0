<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { X } from 'lucide-vue-next';
import { formatDelay } from './retryScheduleFormatters';

type Props = {
  /** Delays in seconds. */
  values: number[];
  /** Show a remove button per chip and emit `remove(index)` on click. */
  removable?: boolean;
  /** Truncate when more than N chips; N+1th becomes a "…" badge. */
  max?: number;
};

const props = withDefaults(defineProps<Props>(), {
  removable: false,
  max: undefined,
});

const emit = defineEmits<{ remove: [index: number] }>();

const { t } = useI18n();

function displayed() {
  if (props.max === undefined) return props.values;
  return props.values.slice(0, props.max);
}

function truncated(): number {
  if (props.max === undefined) return 0;
  return Math.max(0, props.values.length - props.max);
}
</script>

<template>
  <div v-if="values.length === 0" class="interval-chips__empty">—</div>
  <div v-else class="interval-chips">
    <span v-for="(v, index) in displayed()" :key="index" class="interval-chips__chip">
      {{ formatDelay(v) }}
      <button
        v-if="removable"
        type="button"
        class="interval-chips__remove"
        :aria-label="t('common.remove')"
        @click="emit('remove', index)"
      >
        <X :size="12" aria-hidden="true" />
      </button>
    </span>
    <span v-if="truncated() > 0" class="interval-chips__chip interval-chips__chip--more">
      +{{ truncated() }}
    </span>
  </div>
</template>

<style scoped>
.interval-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.interval-chips__empty {
  color: var(--color-text-tertiary);
}

.interval-chips__chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-sm);
  background-color: var(--color-bg-secondary);
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  color: var(--color-text-secondary);
}

.interval-chips__chip--more {
  color: var(--color-text-tertiary);
}

.interval-chips__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  margin: 0;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.interval-chips__remove:hover {
  color: var(--color-error);
}
</style>
