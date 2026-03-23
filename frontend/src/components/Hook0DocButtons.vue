<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { BookOpen, ExternalLink } from 'lucide-vue-next';

import Hook0Button from './Hook0Button.vue';
import Hook0Stack from './Hook0Stack.vue';

type Props = {
  docUrl: string;
  apiUrl: string;
};

defineProps<Props>();

const { t } = useI18n();
</script>

<template>
  <Hook0Stack direction="row" gap="sm">
    <Hook0Button variant="secondary" :href="docUrl" target="_blank">
      <template #left>
        <BookOpen :size="14" aria-hidden="true" />
      </template>
      {{ t('common.documentation') }}
    </Hook0Button>
    <Hook0Button variant="secondary" :href="apiUrl" target="_blank">
      <template #left>
        <ExternalLink :size="14" aria-hidden="true" />
      </template>
      {{ t('common.apiReference') }}
    </Hook0Button>
  </Hook0Stack>
</template>
