<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http';
import { push } from 'notivue';
import { routes } from '@/routes';
import router from '@/router';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { ArrowRight, Check, Shield, CheckCircle } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardDivider from '@/components/Hook0CardDivider.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import Hook0TrustSection from '@/components/Hook0TrustSection.vue';
import Hook0Testimonial from '@/components/Hook0Testimonial.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0InputRow from '@/components/Hook0InputRow.vue';
import Hook0Captcha from '@/components/Hook0Captcha.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

import LogoFranceNuage from '@/components/logos/LogoFranceNuage.vue';
import LogoWoodWing from '@/components/logos/LogoWoodWing.vue';
import LogoOptery from '@/components/logos/LogoOptery.vue';
import LogoOkoora from '@/components/logos/LogoOkoora.vue';
import LogoIcona from '@/components/logos/LogoIcona.vue';
import LogoActiveAnts from '@/components/logos/LogoActiveAnts.vue';

const { t } = useI18n();

const authStore = useAuthStore();

// Form state
const email = ref<string>('');
const firstName = ref<string>('');
const lastName = ref<string>('');
const password = ref<string>('');
const isLoading = ref<boolean>(false);

// Captcha token
const captchaToken = ref<string>('');

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();
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
});

function submit() {
  if (isLoading.value) return;
  isLoading.value = true;

  trackEvent('signup', 'form-submit', 'register');

  authStore
    .register(
      email.value,
      firstName.value,
      lastName.value,
      password.value,
      captchaToken.value !== '' ? captchaToken.value : undefined
    )
    .then(() => {
      trackEvent('signup', 'form-success', 'register');
      return router.push({ name: routes.CheckEmail });
    })
    .catch((err) => {
      const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
      trackEvent('signup', 'form-error', problem.title || 'unknown');
      displayError(problem);
    })
    .finally(() => {
      isLoading.value = false;
    });
}

function displayError(err: Problem) {
  console.error(err);
  const options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}
</script>

<template>
  <Hook0PageLayout variant="fullscreen">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <Hook0Card variant="glow">
      <Hook0CardHeader variant="centered">
        <template #header>{{ t('auth.register.title') }}</template>
        <template #subtitle>
          <Hook0Badge variant="success">{{ t('auth.register.subtitle') }}</Hook0Badge>
          <Hook0Stack direction="column" gap="sm" align="start">
            <Hook0ListItem>
              <template #icon>
                <Check :size="16" aria-hidden="true" />
              </template>
              <template #left>{{ t('auth.register.benefit1') }}</template>
            </Hook0ListItem>
            <Hook0ListItem>
              <template #icon>
                <Check :size="16" aria-hidden="true" />
              </template>
              <template #left>{{ t('auth.register.benefit2') }}</template>
            </Hook0ListItem>
            <Hook0ListItem>
              <template #icon>
                <Check :size="16" aria-hidden="true" />
              </template>
              <template #left>{{ t('auth.register.benefit3') }}</template>
            </Hook0ListItem>
          </Hook0Stack>
        </template>
      </Hook0CardHeader>

      <Hook0CardContent>
        <Hook0Form data-test="register-form" :loading="isLoading" @submit="submit">
          <Hook0Input
            id="email"
            v-model="email"
            type="email"
            required
            :label="t('auth.register.email')"
            :placeholder="t('auth.register.emailPlaceholder')"
            autocomplete="email"
            data-test="register-email-input"
            :disabled="isLoading"
            @focus="handleFormStart"
          />

          <Hook0InputRow>
            <Hook0Input
              id="firstName"
              v-model="firstName"
              type="text"
              required
              :label="t('auth.register.firstName')"
              :placeholder="t('auth.register.firstNamePlaceholder')"
              autocomplete="given-name"
              data-test="register-firstname-input"
              :disabled="isLoading"
            />
            <Hook0Input
              id="lastName"
              v-model="lastName"
              type="text"
              required
              :label="t('auth.register.lastName')"
              :placeholder="t('auth.register.lastNamePlaceholder')"
              autocomplete="family-name"
              data-test="register-lastname-input"
              :disabled="isLoading"
            />
          </Hook0InputRow>

          <Hook0Input
            id="password"
            v-model="password"
            type="password"
            required
            show-password-toggle
            :label="t('auth.register.password')"
            :placeholder="t('auth.register.passwordPlaceholder')"
            autocomplete="new-password"
            data-test="register-password-input"
            :disabled="isLoading"
          />

          <Hook0Captcha v-model="captchaToken" action="registration" />

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

      <Hook0CardContent>
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
      <Hook0Stack align="center" justify="center" gap="lg" wrap>
        <Hook0Badge display="trust" variant="success">
          <template #icon>
            <Shield :size="20" aria-hidden="true" />
          </template>
          {{ t('auth.trust.openSource') }}
        </Hook0Badge>
        <Hook0Badge display="trust" variant="success">
          <template #icon>
            <CheckCircle :size="20" aria-hidden="true" />
          </template>
          {{ t('auth.trust.noCreditCard') }}
        </Hook0Badge>
        <Hook0Badge display="trust" variant="success">
          <template #icon>
            <CheckCircle :size="20" aria-hidden="true" />
          </template>
          {{ t('auth.trust.gdpr') }}
        </Hook0Badge>
      </Hook0Stack>

      <Hook0TrustSection :label="t('auth.register.trustedBy')">
        <Hook0Button
          variant="icon"
          href="https://github.com/France-Nuage/plateforme"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="'France Nuage'"
        >
          <LogoFranceNuage />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.woodwing.com/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="'WoodWing'"
        >
          <LogoWoodWing />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.optery.com/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="'Optery'"
        >
          <LogoOptery />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.icona.it/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="'Icona'"
        >
          <LogoIcona />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://okoora.com/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="'Okoora'"
        >
          <LogoOkoora />
        </Hook0Button>
        <Hook0Button
          variant="icon"
          href="https://www.activeants.com/fr/"
          target="_blank"
          rel="noopener noreferrer"
          :tooltip="'ActiveAnts'"
        >
          <LogoActiveAnts />
        </Hook0Button>
      </Hook0TrustSection>

      <Hook0Testimonial
        :quote="t('auth.register.testimonialQuote')"
        :author="t('auth.register.testimonialAuthor')"
        :role="t('auth.register.testimonialRole')"
        avatar-initial="M"
      />

      <Hook0Stack align="center" justify="center">
        <Hook0Badge>
          {{ t('auth.register.socialProof') }}
        </Hook0Badge>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* All styling is handled by Hook0* components and Hook0PageLayout */
</style>
