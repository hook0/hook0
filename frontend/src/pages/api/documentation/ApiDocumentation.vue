<script setup lang="ts">
import { onMounted } from 'vue';
import { ExternalLink, BookOpen } from 'lucide-vue-next';
import featureFlags from '@/feature-flags';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const { trackEvent } = useTracking();

const swaggerJsonUrl =
  featureFlags.getOrElse('API_ENDPOINT', import.meta.env.VITE_API_ENDPOINT ?? '') + '/swagger.json';

const docsUrl = 'https://documentation.hook0.com/';

onMounted(() => {
  trackEvent('api-docs', 'page-view', 'documentation');
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>{{ t('apiDocs.title') }}</template>
      <template #subtitle>{{ t('apiDocs.subtitle') }}</template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0Stack direction="column" align="center" gap="lg">
        <Hook0IconBadge variant="primary" size="lg">
          <BookOpen :size="24" aria-hidden="true" />
        </Hook0IconBadge>
        <Hook0Text variant="secondary" block>
          {{ t('apiDocs.description') }}
        </Hook0Text>
        <Hook0Stack direction="column" align="center" gap="sm">
          <Hook0Button variant="primary" :href="docsUrl" target="_blank" rel="noopener noreferrer">
            <template #left>
              <BookOpen :size="18" aria-hidden="true" />
            </template>
            {{ t('apiDocs.openDocs') }}
            <template #right>
              <ExternalLink :size="14" aria-hidden="true" />
            </template>
          </Hook0Button>
          <Hook0Button
            variant="secondary"
            :href="swaggerJsonUrl"
            target="_blank"
            rel="noopener noreferrer"
          >
            {{ t('apiDocs.viewSpec') }}
            <template #right>
              <ExternalLink :size="14" aria-hidden="true" />
            </template>
          </Hook0Button>
        </Hook0Stack>
      </Hook0Stack>
    </Hook0CardContent>
  </Hook0Card>
</template>

<style scoped>
/* All styling handled by Hook0* components */
</style>
