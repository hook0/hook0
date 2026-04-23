<script setup lang="ts">
// Editable chip for a single custom retry interval. Uses v-model<number> (seconds) on the outside,
// but keeps a local formatted string for typing so the user can enter "1h30min" instead of raw seconds.
// Validation toasts fire on blur; invalid input reverts to the last committed value.
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';
import { formatDuration, parseDuration } from '@/utils/duration';

type Props = {
  min: number;
  max: number;
  index: number;
};

const props = defineProps<Props>();
const model = defineModel<number>({ required: true });
const emit = defineEmits<{ remove: [] }>();

const { t } = useI18n();

const text = ref(formatDuration(model.value));

// Re-sync local text when the parent replaces the value (e.g. form reset on hydration)
watch(model, (next) => {
  text.value = formatDuration(next);
});

function commit() {
  const parsed = parseDuration(text.value);
  if (parsed === null) {
    toast.error(t('retrySchedules.fields.intervalInvalid'));
  } else if (parsed < props.min) {
    toast.error(t('retrySchedules.fields.intervalBelowMin', { min: formatDuration(props.min) }));
  } else if (parsed > props.max) {
    toast.error(t('retrySchedules.fields.intervalAboveMax', { max: formatDuration(props.max) }));
  } else {
    model.value = parsed;
  }
  // Always reformat — either reflects the new value or reverts stale user input
  text.value = formatDuration(model.value);
}
</script>

<template>
  <div class="custom-chip" data-test="retry-schedule-custom-chip">
    <input
      v-model="text"
      type="text"
      class="custom-chip__input"
      :data-test="`retry-schedule-custom-chip-input-${index}`"
      :aria-label="t('retrySchedules.fields.retryNumber', { number: index + 1 })"
      @blur="commit"
      @keydown.enter="($event.target as HTMLInputElement).blur()"
    />
    <button
      type="button"
      class="custom-chip__remove"
      :data-test="`retry-schedule-custom-chip-remove-${index}`"
      :aria-label="t('retrySchedules.fields.removeInterval', { number: index + 1 })"
      @click="emit('remove')"
    >
      ×
    </button>
  </div>
</template>

<style scoped>
.custom-chip {
  display: inline-flex;
  align-items: center;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 500;
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  overflow: hidden;
  transition: border-color 0.15s ease;
}

.custom-chip:focus-within {
  border-color: var(--color-primary);
}

.custom-chip__input {
  border: none;
  background: transparent;
  font-size: 0.75rem;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-secondary);
  padding: 0.25rem 0.5rem;
  width: 5rem;
  text-align: center;
  outline: none;
}

.custom-chip__input:focus {
  color: var(--color-text-primary);
}

.custom-chip__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-left: 1px solid var(--color-border);
  color: var(--color-text-tertiary);
  cursor: pointer;
  padding: 0.25rem 0.375rem;
  font-size: 0.875rem;
  line-height: 1;
  transition: all 0.15s ease;
}

.custom-chip__remove:hover {
  background-color: var(--color-error-light);
  color: var(--color-error);
}
</style>
