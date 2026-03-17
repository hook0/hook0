<script setup lang="ts">
import { ref } from 'vue';
import { AlertTriangle, Trash2 } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';

const { t } = useI18n();

type Props = {
  title: string;
  subtitle: string;
  warningMessage: string;
  confirmMessage: string;
  loading?: boolean;
  dataTest?: string;
};

withDefaults(defineProps<Props>(), {
  loading: false,
  dataTest: undefined,
});

const emit = defineEmits<{
  confirm: [];
}>();

const showDeleteDialog = ref(false);

function requestDelete(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();
  showDeleteDialog.value = true;
}

function confirmDelete() {
  showDeleteDialog.value = false;
  emit('confirm');
}
</script>

<template>
  <Hook0Card :data-test="dataTest">
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="danger">
            <AlertTriangle :size="18" aria-hidden="true" />
          </Hook0IconBadge>
          <span class="danger-zone__title">{{ title }}</span>
        </Hook0Stack>
      </template>
      <template #subtitle>{{ subtitle }}</template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0Alert type="alert">
        <template #description>{{ warningMessage }}</template>
      </Hook0Alert>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button
        variant="danger"
        type="button"
        :loading="loading"
        :data-test="dataTest ? `${dataTest.replace(/-card$/, '')}-button` : undefined"
        @click="requestDelete($event)"
      >
        <Trash2 :size="16" aria-hidden="true" />
        {{ t('common.delete') }}
      </Hook0Button>
    </Hook0CardFooter>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="title"
      @close="showDeleteDialog = false"
      @confirm="confirmDelete()"
    >
      <p>{{ confirmMessage }}</p>
    </Hook0Dialog>
  </Hook0Card>
</template>

<style scoped>
.danger-zone__title {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
