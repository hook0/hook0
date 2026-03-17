<script setup lang="ts">
import { onMounted } from 'vue';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { Mail, LifeBuoy, ArrowLeft } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ButtonGroup from '@/components/Hook0ButtonGroup.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';

const { t } = useI18n();

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();

onMounted(() => {
  trackPageWithDimensions('auth', 'view', 'email-sent');
  trackEvent('signup', 'page-view', 'check-email');
});
</script>

<template>
  <Hook0PageLayout variant="fullscreen" data-test="check-email-page">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <Hook0Card variant="glow">
      <Hook0CardContent>
        <Hook0EmptyState
          :title="t('auth.checkEmail.title')"
          :description="t('auth.checkEmail.description')"
        >
          <template #icon>
            <Mail :size="64" aria-hidden="true" />
          </template>
        </Hook0EmptyState>

        <Hook0Alert type="info">
          <template #description>
            {{ t('auth.checkEmail.spamNote') }}
          </template>
        </Hook0Alert>

        <Hook0ButtonGroup direction="column" gap="sm" full-width>
          <Hook0Button variant="primary" size="lg" href="mailto:support@hook0.com" full-width>
            <template #left>
              <LifeBuoy :size="20" aria-hidden="true" />
            </template>
            {{ t('auth.checkEmail.contactSupport') }}
          </Hook0Button>

          <Hook0Button variant="ghost" size="lg" :to="{ name: routes.Login }" full-width>
            <template #left>
              <ArrowLeft :size="16" aria-hidden="true" />
            </template>
            {{ t('auth.checkEmail.backToLogin') }}
          </Hook0Button>
        </Hook0ButtonGroup>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
/* No custom styles - Hook0 components handle layout */
</style>
