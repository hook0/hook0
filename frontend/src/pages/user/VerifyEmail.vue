<script setup lang="ts">
import * as UserService from './UserService.ts';
import { Problem } from '@/http.ts';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import { toast } from 'vue-sonner';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { ArrowLeft } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Spinner from '@/components/Hook0Spinner.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();

const route = useRoute();

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();

// State
const isLoading = ref<boolean>(true);
const alert = ref<{
  visible: boolean;
  type: 'alert' | 'warning';
  title: string;
  description: string;
}>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function displayError(err: Problem) {
  console.error(err);
  isLoading.value = false;
  alert.value.visible = true;
  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function displaySuccess() {
  trackPageWithDimensions('auth', 'view', 'email-verified');
  trackEvent('signup', 'email-verified');
  toast.success(t('auth.verifyEmail.verified'), {
    description: t('auth.verifyEmail.redirecting'),
    duration: 5000,
  });
}

function _onLoad() {
  const token = route.query.token as string;
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: t('auth.verifyEmail.invalidToken'),
      detail: t('auth.verifyEmail.tokenRequired'),
    });
    return;
  }

  UserService.verifyEmail(token)
    .then(() => {
      displaySuccess();
      return router.push({ name: routes.Login });
    })
    .catch((err) => {
      displayError(err as Problem);
    });
}

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0PageLayout variant="fullscreen" data-test="verify-email-page">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <!-- Loading State -->
    <Hook0Card v-if="isLoading && !alert.visible" variant="glow" data-test="verify-email-loading">
      <Hook0CardContent>
        <Hook0Stack direction="column" align="center" justify="center" gap="md">
          <Hook0Spinner :size="48" />
          <span class="verify-email__message">
            {{ t('auth.verifyEmail.verifying') }}
          </span>
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error Card -->
    <Hook0Card v-else-if="alert.visible" variant="glow" data-test="verify-email-error">
      <Hook0CardContent>
        <Hook0Stack direction="column" gap="lg">
          <Hook0Alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
            data-test="verify-email-error-title"
          />

          <Hook0Button
            variant="ghost"
            size="lg"
            :to="{ name: routes.Login }"
            full-width
            data-test="verify-email-back-to-login"
          >
            <template #left>
              <ArrowLeft :size="16" aria-hidden="true" />
            </template>
            {{ t('auth.verifyEmail.backToLogin') }}
          </Hook0Button>
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
.verify-email__message {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}
</style>
