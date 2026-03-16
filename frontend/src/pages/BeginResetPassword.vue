<script setup lang="ts">
import { ref } from 'vue';
import { push } from 'notivue';
import { useAuthErrorHandler } from '@/composables/useAuthErrorHandler';
import { beginResetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes';
import { useI18n } from 'vue-i18n';
import { ArrowLeft } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();

const { handleAuthError } = useAuthErrorHandler();

// Form state
const email = ref<string>('');
const isLoading = ref<boolean>(false);

function submit() {
  if (isLoading.value) return;
  isLoading.value = true;

  beginResetPassword(email.value)
    .then(() => {
      push.success({
        title: t('common.success'),
        message: t('auth.forgotPassword.emailSent'),
        duration: 5000,
      });
    })
    .catch((err) => handleAuthError(err))
    .finally(() => {
      isLoading.value = false;
    });
}
</script>

<template>
  <Hook0PageLayout variant="fullscreen">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <Hook0Card variant="glow">
      <Hook0CardHeader
        variant="centered"
        :title="t('auth.forgotPassword.title')"
        :subtitle="t('auth.forgotPassword.description')"
      />

      <Hook0CardContent>
        <Hook0Form data-test="reset-password-form" :loading="isLoading" @submit="submit">
          <Hook0Input
            id="email"
            v-model="email"
            type="email"
            required
            :label="t('auth.forgotPassword.email')"
            :placeholder="t('auth.forgotPassword.emailPlaceholder')"
            autocomplete="email"
            :disabled="isLoading"
            data-test="reset-password-email-input"
          />

          <Hook0Button
            variant="primary"
            size="lg"
            submit
            :loading="isLoading"
            :disabled="isLoading"
            full-width
            data-test="reset-password-submit-button"
          >
            {{ isLoading ? t('auth.forgotPassword.submitting') : t('auth.forgotPassword.submit') }}
          </Hook0Button>
        </Hook0Form>

        <Hook0Button
          variant="ghost"
          size="lg"
          :to="{ name: routes.Login }"
          full-width
          data-test="reset-password-back-link"
        >
          <template #left>
            <ArrowLeft :size="16" aria-hidden="true" />
          </template>
          {{ t('auth.forgotPassword.backToLogin') }}
        </Hook0Button>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0PageLayout variant="fullscreen" handles card width (28rem max) */
</style>
