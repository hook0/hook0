<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { routes } from '@/routes';
import router from '@/router';
import { useAuthErrorHandler } from '@/composables/useAuthErrorHandler';
import { useForm } from 'vee-validate';
import { createRegisterSchema } from './register.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { ArrowRight, Check } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardDivider from '@/components/Hook0CardDivider.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0TrustSection from '@/components/Hook0TrustSection.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0InputRow from '@/components/Hook0InputRow.vue';
import Hook0Captcha from '@/components/Hook0Captcha.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0AuthTrustBadges from '@/components/Hook0AuthTrustBadges.vue';

import CustomerLogo from '@/components/logos/CustomerLogo.vue';

const { t } = useI18n();

const authStore = useAuthStore();

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit } = useForm({
  validationSchema: toTypedSchema(createRegisterSchema()),
});

const [email, emailAttrs] = defineField('email');
const [firstName, firstNameAttrs] = defineField('firstName');
const [lastName, lastNameAttrs] = defineField('lastName');
const [password, passwordAttrs] = defineField('password');

const isLoading = ref<boolean>(false);

// Captcha token
const captchaToken = ref<string>('');

// Google Ads click identifier captured from the URL when the user lands
// here after clicking an ad. Forwarded to the API for server-side
// conversion upload (no gtag.js, no client-side trackers, no PII).
const gclid = ref<string>('');

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();
const { handleAuthError } = useAuthErrorHandler();
const formStarted = ref<boolean>(false);

function handleFormStart() {
  if (!formStarted.value) {
    formStarted.value = true;
    trackEvent('signup', 'form-start', 'register');
  }
}

onMounted(() => {
  trackPageWithDimensions('auth', 'view', 'signup-form');
  trackEvent('signup', 'page-view', 'register');
  const params = new URLSearchParams(window.location.search);
  const fromUrl = params.get('gclid');
  if (fromUrl) {
    gclid.value = fromUrl;
  }
});

const onSubmit = handleSubmit((values) => {
  if (isLoading.value) return;
  isLoading.value = true;

  trackEvent('signup', 'form-submit', 'register');

  authStore
    .register(
      values.email,
      values.firstName,
      values.lastName,
      values.password,
      captchaToken.value !== '' ? captchaToken.value : undefined,
      gclid.value !== '' ? gclid.value : undefined
    )
    .then(() => {
      trackEvent('signup', 'form-success', 'register');
      return router.push({ name: routes.CheckEmail });
    })
    .catch((err) => {
      const problem = handleAuthError(err);
      trackEvent('signup', 'form-error', problem.title || 'unknown');
    })
    .finally(() => {
      isLoading.value = false;
    });
});
</script>

<template>
  <Hook0PageLayout variant="fullscreen">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <Hook0Card>
      <Hook0CardHeader variant="centered">
        <template #header>{{ t('auth.register.title') }}</template>
        <template #subtitle>
          <div class="register-subtitle">
            <Hook0Badge variant="success">{{ t('auth.register.subtitle') }}</Hook0Badge>
          </div>
          <ul class="benefit-list">
            <li class="benefit-list__item">
              <Check :size="16" aria-hidden="true" class="benefit-list__icon" />
              <span class="benefit-list__text">{{ t('auth.register.benefit1') }}</span>
            </li>
            <li class="benefit-list__item">
              <Check :size="16" aria-hidden="true" class="benefit-list__icon" />
              <span class="benefit-list__text">{{ t('auth.register.benefit2') }}</span>
            </li>
            <li class="benefit-list__item">
              <Check :size="16" aria-hidden="true" class="benefit-list__icon" />
              <span class="benefit-list__text">{{ t('auth.register.benefit3') }}</span>
            </li>
          </ul>
        </template>
      </Hook0CardHeader>

      <Hook0CardContent>
        <Hook0Form data-test="register-form" :loading="isLoading" @submit="onSubmit">
          <Hook0Input
            id="email"
            v-model="email"
            v-bind="emailAttrs"
            type="email"
            required
            :label="t('auth.register.email')"
            :placeholder="t('auth.register.emailPlaceholder')"
            :error="errors.email"
            autocomplete="email"
            data-test="register-email-input"
            :disabled="isLoading"
            autofocus
            @focus="handleFormStart"
          />

          <Hook0InputRow>
            <Hook0Input
              id="firstName"
              v-model="firstName"
              v-bind="firstNameAttrs"
              type="text"
              required
              :label="t('auth.register.firstName')"
              :placeholder="t('auth.register.firstNamePlaceholder')"
              :error="errors.firstName"
              autocomplete="given-name"
              data-test="register-firstname-input"
              :disabled="isLoading"
            />
            <Hook0Input
              id="lastName"
              v-model="lastName"
              v-bind="lastNameAttrs"
              type="text"
              required
              :label="t('auth.register.lastName')"
              :placeholder="t('auth.register.lastNamePlaceholder')"
              :error="errors.lastName"
              autocomplete="family-name"
              data-test="register-lastname-input"
              :disabled="isLoading"
            />
          </Hook0InputRow>

          <Hook0Input
            id="password"
            v-model="password"
            v-bind="passwordAttrs"
            type="password"
            required
            show-password-toggle
            :label="t('auth.register.password')"
            :placeholder="t('auth.register.passwordPlaceholder')"
            :error="errors.password"
            autocomplete="new-password"
            data-test="register-password-input"
            :disabled="isLoading"
          />

          <Hook0Captcha v-model="captchaToken" action="registration" class="register-captcha" />

          <Hook0Button
            variant="primary"
            size="lg"
            submit
            :loading="isLoading"
            :disabled="isLoading"
            full-width
            data-test="register-submit-button"
          >
            {{ isLoading ? t('auth.register.submitting') : t('auth.register.submit') }}
          </Hook0Button>
        </Hook0Form>
      </Hook0CardContent>

      <Hook0CardDivider>{{ t('auth.register.hasAccount') }}</Hook0CardDivider>

      <Hook0CardContent remove-top-border>
        <Hook0Button
          variant="secondary"
          size="lg"
          :to="{ name: routes.Login }"
          full-width
          data-test="register-login-link"
        >
          {{ t('auth.register.signIn') }}
          <template #right>
            <ArrowRight :size="16" aria-hidden="true" />
          </template>
        </Hook0Button>
      </Hook0CardContent>
    </Hook0Card>

    <template #footer>
      <Hook0AuthTrustBadges
        :badges="['auth.trust.openSource', 'auth.trust.noCreditCard', 'auth.trust.gdpr']"
      />

      <Hook0TrustSection :label="t('auth.register.trustedBy')">
        <Hook0Button
          variant="icon"
          href="https://github.com/France-Nuage/plateforme"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="t('auth.register.customers.franceNuage')"
          :aria-label="t('auth.register.customers.franceNuage')"
        >
          <CustomerLogo variant="france-nuage" />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.woodwing.com/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="t('auth.register.customers.woodWing')"
          :aria-label="t('auth.register.customers.woodWing')"
        >
          <CustomerLogo variant="woodwing" />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.optery.com/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="t('auth.register.customers.optery')"
          :aria-label="t('auth.register.customers.optery')"
        >
          <CustomerLogo variant="optery" />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.icona.it/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="t('auth.register.customers.icona')"
          :aria-label="t('auth.register.customers.icona')"
        >
          <CustomerLogo variant="icona" />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://okoora.com/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="t('auth.register.customers.okoora')"
          :aria-label="t('auth.register.customers.okoora')"
        >
          <CustomerLogo variant="okoora" />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.activeants.com/fr/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="t('auth.register.customers.activeAnts')"
          :aria-label="t('auth.register.customers.activeAnts')"
        >
          <CustomerLogo variant="active-ants" />
        </Hook0Button>
      </Hook0TrustSection>

      <Hook0Stack align="center" justify="center">
        <Hook0Badge>
          {{ t('auth.register.socialProof') }}
        </Hook0Badge>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.register-subtitle {
  margin-top: 0.5rem;
}

.benefit-list {
  list-style: none;
  padding: 0;
  margin: 0.75rem 0 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.125rem;
}

.benefit-list__item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.25rem 0;
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.benefit-list__icon {
  flex-shrink: 0;
  color: var(--color-success);
}

.benefit-list__text {
  color: var(--color-text-primary);
  text-align: left;
}

.register-captcha {
  margin-top: 0.5rem;
}
</style>
