<script setup lang="ts">
import * as UserService from './UserService.ts';
import { Problem } from '@/http.ts';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import router from '@/router.ts';
import { stripTokenFromUrl } from '@/utils/stripTokenFromUrl';
import { routes } from '@/routes.ts';
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

const isLoading = ref<boolean>(true);
const isUnsubscribed = ref<boolean>(false);
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

function _onLoad() {
  const token = route.query.token as string;
  // The opt-out token is long-lived, so it must not linger anywhere URLs are
  // observed (analytics page views and referrers, browser history).
  stripTokenFromUrl(router);
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: t('userSettings.unsubscribe.invalidToken'),
      detail: t('userSettings.unsubscribe.tokenRequired'),
    });
    return;
  }

  UserService.unsubscribeReactivation(token)
    .then(() => {
      isLoading.value = false;
      isUnsubscribed.value = true;
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
  <Hook0PageLayout variant="fullscreen" data-test="unsubscribe-page">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <!-- Loading State -->
    <Hook0Card v-if="isLoading" variant="glow" data-test="unsubscribe-loading">
      <Hook0CardContent>
        <Hook0Stack direction="column" align="center" justify="center" gap="md">
          <Hook0Spinner :size="48" />
          <span class="unsubscribe__message">
            {{ t('userSettings.unsubscribe.working') }}
          </span>
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Confirmation -->
    <Hook0Card v-else-if="isUnsubscribed" variant="glow" data-test="unsubscribe-confirmation">
      <Hook0CardContent>
        <Hook0Stack direction="column" gap="lg">
          <Hook0Alert
            type="success"
            :title="t('userSettings.unsubscribe.doneTitle')"
            :description="t('userSettings.unsubscribe.doneDescription')"
            data-test="unsubscribe-confirmation-title"
          />

          <Hook0Button
            variant="ghost"
            size="lg"
            :to="{ name: routes.Login }"
            full-width
            data-test="unsubscribe-back-to-login"
          >
            <template #left>
              <ArrowLeft :size="16" aria-hidden="true" />
            </template>
            {{ t('userSettings.unsubscribe.backToLogin') }}
          </Hook0Button>
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error Card -->
    <Hook0Card v-else-if="alert.visible" variant="glow" data-test="unsubscribe-error">
      <Hook0CardContent>
        <Hook0Stack direction="column" gap="lg">
          <Hook0Alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
            data-test="unsubscribe-error-title"
          />

          <Hook0Button
            variant="ghost"
            size="lg"
            :to="{ name: routes.Login }"
            full-width
            data-test="unsubscribe-back-to-login"
          >
            <template #left>
              <ArrowLeft :size="16" aria-hidden="true" />
            </template>
            {{ t('userSettings.unsubscribe.backToLogin') }}
          </Hook0Button>
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
.unsubscribe__message {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}
</style>
