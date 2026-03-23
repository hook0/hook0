<script setup lang="ts">
import { Construction } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';

const { t } = useI18n();
</script>

<template>
  <Hook0PageLayout>
    <Hook0Card>
      <Hook0CardContent>
        <Hook0EmptyState :title="t('comingSoon.title')" :description="t('comingSoon.description')">
          <template #icon>
            <Construction :size="48" aria-hidden="true" />
          </template>
        </Hook0EmptyState>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0* components handle all styling via variants */
</style>
