<script setup lang="ts">
import { ref } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { useRouter } from 'vue-router';
import { routes } from '@/routes';
import { useAuthErrorHandler } from '@/composables/useAuthErrorHandler';
import { useForm } from 'vee-validate';
import * as OrganizationService from './organizations/OrganizationService';
import * as ApplicationService from './organizations/applications/ApplicationService';
import { loginSchema } from './login.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { ArrowRight } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardDivider from '@/components/Hook0CardDivider.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0AuthTrustBadges from '@/components/Hook0AuthTrustBadges.vue';

const { t } = useI18n();

const router = useRouter();
const authStore = useAuthStore();

// Analytics tracking
const { trackEvent } = useTracking();
const { handleAuthError } = useAuthErrorHandler();

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit } = useForm({
  validationSchema: toTypedSchema(loginSchema),
});

const [email, emailAttrs] = defineField('email');
const [password, passwordAttrs] = defineField('password');

const isLoading = ref<boolean>(false);

const onSubmit = handleSubmit((values) => {
  if (isLoading.value) return;
  isLoading.value = true;

  authStore
    .login(values.email, values.password)
    .then(() => {
      trackEvent('auth', 'login', 'success');

      return OrganizationService.list();
    })
    .then((organizations) => {
      // No organizations → show tutorial
      if (organizations.length === 0) {
        return router.push({ name: routes.Tutorial });
      }

      // Single organization → check if it has applications
      if (organizations.length === 1) {
        return ApplicationService.list(organizations[0].organization_id).then((applications) => {
          const destination = applications.length === 0 ? routes.Tutorial : routes.Home;
          return router.push({ name: destination });
        });
      }

      // Multiple organizations → go to home
      return router.push({ name: routes.Home });
    })
    .catch((err) => {
      handleAuthError(err);
      trackEvent('auth', 'login', 'error');
    })
    .finally(() => {
      isLoading.value = false;
    });
});
</script>

<template>
  <Hook0PageLayout variant="fullscreen">
    <template #logo>
      <Hook0Logo variant="image" size="lg" data-test="login-logo" />
    </template>

    <Hook0Card variant="glow">
      <Hook0CardHeader
        variant="centered"
        :title="t('auth.login.title')"
        :subtitle="t('auth.login.subtitle')"
      />

      <Hook0CardContent>
        <Hook0Form data-test="login-form" :loading="isLoading" @submit="onSubmit">
          <Hook0Input
            id="email"
            v-model="email"
            v-bind="emailAttrs"
            type="email"
            required
            :label="t('auth.login.email')"
            :placeholder="t('auth.login.emailPlaceholder')"
            :error="errors.email"
            autocomplete="email"
            data-test="login-email-input"
            :disabled="isLoading"
          />

          <Hook0Input
            id="password"
            v-model="password"
            v-bind="passwordAttrs"
            type="password"
            required
            show-password-toggle
            :label="t('auth.login.password')"
            :placeholder="t('auth.login.passwordPlaceholder')"
            :error="errors.password"
            autocomplete="current-password"
            data-test="login-password-input"
            :disabled="isLoading"
          />

          <Hook0Stack justify="end">
            <Hook0Button
              variant="link"
              size="sm"
              :to="{ name: routes.BeginResetPassword }"
              data-test="login-forgot-password-link"
            >
              {{ t('auth.login.forgotPassword') }}
            </Hook0Button>
          </Hook0Stack>

          <Hook0Button
            variant="primary"
            size="lg"
            submit
            :loading="isLoading"
            :disabled="isLoading"
            full-width
            data-test="login-submit-button"
          >
            {{ isLoading ? t('auth.login.submitting') : t('auth.login.submit') }}
          </Hook0Button>
        </Hook0Form>
      </Hook0CardContent>

      <Hook0CardDivider>{{ t('auth.login.newToHook0') }}</Hook0CardDivider>

      <Hook0CardContent removeTopBorder>
        <Hook0Button
          variant="secondary"
          size="lg"
          :to="{ name: routes.Register }"
          full-width
          data-test="login-register-link"
        >
          {{ t('auth.login.createAccount') }}
          <template #right>
            <ArrowRight :size="16" aria-hidden="true" />
          </template>
        </Hook0Button>
      </Hook0CardContent>
    </Hook0Card>

    <template #footer>
      <Hook0AuthTrustBadges />
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* No custom styles - Hook0 components handle layout */
</style>
