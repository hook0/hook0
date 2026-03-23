<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Home, ArrowLeft } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ButtonGroup from '@/components/Hook0ButtonGroup.vue';
import Hook0Error404Illustration from '@/components/Hook0Error404Illustration.vue';
import Hook0Error404Background from '@/components/Hook0Error404Background.vue';

const { t } = useI18n();
const router = useRouter();

function goHome() {
  void router.push('/');
}

function goBack() {
  router.back();
}
</script>

<template>
  <Hook0PageLayout variant="fullscreen" data-test="error-404-page">
    <template #background>
      <Hook0Error404Background />
    </template>

    <Hook0Card variant="glow">
      <Hook0CardContent>
        <Hook0EmptyState
          :title="t('error404.title')"
          :description="t('error404.description')"
          data-test="error-404-title"
        >
          <template #illustration>
            <Hook0Error404Illustration />
          </template>

          <template #action>
            <Hook0ButtonGroup direction="column" responsive>
              <Hook0Button
                variant="primary"
                size="lg"
                data-test="error-404-dashboard-button"
                @click="goHome"
              >
                <template #left>
                  <Home :size="20" aria-hidden="true" />
                </template>
                {{ t('error404.goToDashboard') }}
              </Hook0Button>
              <Hook0Button
                variant="secondary"
                size="lg"
                data-test="error-404-back-button"
                @click="goBack"
              >
                <template #left>
                  <ArrowLeft :size="20" aria-hidden="true" />
                </template>
                {{ t('error404.goBack') }}
              </Hook0Button>
            </Hook0ButtonGroup>
          </template>
        </Hook0EmptyState>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0* components handle all styling via variants */
</style>
