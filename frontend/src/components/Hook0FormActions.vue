<script setup lang="ts">
import { useI18n } from 'vue-i18n';

import Hook0Button from '@/components/Hook0Button.vue';

type FormActionsVariant = 'default' | 'danger';

interface Props {
  submitLabel: string;
  cancelLabel: string;
  loading: boolean;
  disabled: boolean;
  variant: FormActionsVariant;
}

const props = withDefaults(defineProps<Props>(), {
  submitLabel: '',
  cancelLabel: '',
  loading: false,
  disabled: false,
  variant: 'default',
});

const emit = defineEmits<{
  submit: [];
  cancel: [];
}>();

const { t } = useI18n();

function resolvedSubmitLabel(): string {
  return props.submitLabel || t('common.save');
}

function resolvedCancelLabel(): string {
  return props.cancelLabel || t('common.cancel');
}
</script>

<template>
  <div class="hook0-form-actions">
    <Hook0Button variant="secondary" :disabled="props.loading" @click="emit('cancel')">
      {{ resolvedCancelLabel() }}
    </Hook0Button>
    <Hook0Button
      :variant="props.variant === 'danger' ? 'danger' : 'primary'"
      :loading="props.loading"
      :disabled="props.disabled"
      submit
      @click="emit('submit')"
    >
      {{ resolvedSubmitLabel() }}
    </Hook0Button>
  </div>
</template>

<style scoped>
.hook0-form-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 0;
}
</style>
