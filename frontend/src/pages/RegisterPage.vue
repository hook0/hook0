<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { register } from '@/iam';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { push } from 'notivue';
import { routes } from '@/routes.ts';
import router from '@/router.ts';
import VueTurnstile from 'vue-turnstile';
import { getInstanceConfig } from '@/utils/instance-config';
import { useCardGlow } from '@/composables/useCardGlow';
import { useTracking } from '@/composables/useTracking';
import IconCheck from '@/components/icons/IconCheck.vue';
import IconCheckCircle from '@/components/icons/IconCheckCircle.vue';
import IconShieldCheck from '@/components/icons/IconShieldCheck.vue';
import LogoFranceNuage from '@/components/logos/LogoFranceNuage.vue';
import LogoWoodWing from '@/components/logos/LogoWoodWing.vue';
import LogoOptery from '@/components/logos/LogoOptery.vue';
import LogoOkoora from '@/components/logos/LogoOkoora.vue';
import LogoIcona from '@/components/logos/LogoIcona.vue';
import LogoActiveAnts from '@/components/logos/LogoActiveAnts.vue';

// Form state
const email = ref<string>('');
const firstName = ref<string>('');
const lastName = ref<string>('');
const password = ref<string>('');
const showPassword = ref<boolean>(false);
const isLoading = ref<boolean>(false);

// Turnstile
const turnstile_site_key = ref<null | string>(null);
const turnstile_token = ref<string>('');

// Mouse tracking for card glow effect
const { cardRef, mouseX, mouseY, handleMouseMove } = useCardGlow();

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
  // Track page view with custom dimensions
  trackPageWithDimensions('auth', 'view', 'signup-form');
  trackEvent('signup', 'page-view', 'register');

  getInstanceConfig()
    .then((instanceConfig) => {
      if (instanceConfig.cloudflare_turnstile_site_key) {
        turnstile_site_key.value = instanceConfig.cloudflare_turnstile_site_key;
      }
    })
    .catch(console.error);
});

function submit() {
  if (isLoading.value) return;
  isLoading.value = true;

  // Track form submission attempt
  trackEvent('signup', 'form-submit', 'register');

  register(
    email.value,
    firstName.value,
    lastName.value,
    password.value,
    turnstile_token.value !== '' ? turnstile_token.value : undefined
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

function togglePasswordVisibility() {
  showPassword.value = !showPassword.value;
}
</script>

<template>
  <div class="register-page">
    <!-- Background Effects -->
    <div class="register-page__background">
      <div class="register-page__grid-pattern"></div>
      <div class="register-page__blur-circle register-page__blur-circle--indigo"></div>
      <div class="register-page__blur-circle register-page__blur-circle--green"></div>
    </div>

    <!-- Main Content -->
    <div class="register-page__content">
      <!-- Logo -->
      <div class="register-page__logo">
        <img src="/logo.svg" alt="Hook0" class="w-auto h-12" />
      </div>

      <!-- Card -->
      <div
        ref="cardRef"
        class="register-page__card"
        :style="{ '--mouse-x': mouseX, '--mouse-y': mouseY }"
        @mousemove="handleMouseMove"
      >
        <!-- Header -->
        <div class="register-page__header">
          <h1 class="register-page__title">Start Your Free Trial</h1>
          <p class="register-page__subtitle register-page__subtitle--highlight">
            No Credit Card Required
          </p>

          <!-- Benefits List -->
          <ul class="register-page__benefits">
            <li class="register-page__benefit">
              <IconCheck />
              <span>Up to 100 free events/day</span>
            </li>
            <li class="register-page__benefit">
              <IconCheck />
              <span>Set up in 5 minutes</span>
            </li>
            <li class="register-page__benefit">
              <IconCheck />
              <span>Cancel anytime</span>
            </li>
          </ul>
        </div>

        <!-- Form -->
        <form class="register-page__form" data-test="register-form" @submit.prevent="submit">
          <!-- Email Field -->
          <div class="register-page__field">
            <label for="email" class="register-page__label">Email</label>
            <input
              id="email"
              v-model="email"
              type="email"
              required
              placeholder="you@company.com"
              class="register-page__input"
              autocomplete="email"
              data-test="register-email-input"
              :disabled="isLoading"
              @focus="handleFormStart"
            />
          </div>

          <!-- Name Fields -->
          <div class="register-page__field-row">
            <div class="register-page__field register-page__field--half">
              <label for="firstName" class="register-page__label">First name</label>
              <input
                id="firstName"
                v-model="firstName"
                type="text"
                required
                placeholder="John"
                class="register-page__input"
                autocomplete="given-name"
                data-test="register-firstname-input"
                :disabled="isLoading"
              />
            </div>
            <div class="register-page__field register-page__field--half">
              <label for="lastName" class="register-page__label">Last name</label>
              <input
                id="lastName"
                v-model="lastName"
                type="text"
                required
                placeholder="Doe"
                class="register-page__input"
                autocomplete="family-name"
                data-test="register-lastname-input"
                :disabled="isLoading"
              />
            </div>
          </div>

          <!-- Password Field -->
          <div class="register-page__field">
            <label for="password" class="register-page__label">Password</label>
            <div class="register-page__password-wrapper">
              <input
                id="password"
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                required
                placeholder="Create a strong password"
                class="register-page__input register-page__input--password"
                autocomplete="new-password"
                data-test="register-password-input"
                :disabled="isLoading"
              />
              <button
                type="button"
                class="register-page__password-toggle"
                :aria-label="showPassword ? 'Hide password' : 'Show password'"
                @click="togglePasswordVisibility"
              >
                <svg
                  v-if="!showPassword"
                  xmlns="http://www.w3.org/2000/svg"
                  class="w-5 h-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-hidden="true"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                  />
                </svg>
                <svg
                  v-else
                  xmlns="http://www.w3.org/2000/svg"
                  class="w-5 h-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-hidden="true"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                  />
                </svg>
              </button>
            </div>
          </div>

          <!-- Turnstile -->
          <div v-if="turnstile_site_key" class="register-page__turnstile">
            <VueTurnstile
              v-model="turnstile_token"
              :site-key="turnstile_site_key"
              size="flexible"
              action="registration"
              theme="dark"
            />
          </div>

          <!-- Submit Button -->
          <button type="submit" class="register-page__submit" data-test="register-submit-button" :disabled="isLoading">
            <span v-if="!isLoading">Create account</span>
            <span v-else class="register-page__loading">
              <svg
                class="w-5 h-5 animate-spin"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                ></circle>
                <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
              </svg>
              Creating account...
            </span>
          </button>
        </form>

        <!-- Divider -->
        <div class="register-page__divider">
          <span>Already have an account?</span>
        </div>

        <!-- Sign in link -->
        <router-link :to="{ name: routes.Login }" class="register-page__signin" data-test="register-login-link">
          Sign in
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="ml-2 w-4 h-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 7l5 5m0 0l-5 5m5-5H6"
            />
          </svg>
        </router-link>
      </div>

      <!-- Trust Indicators -->
      <div class="register-page__trust">
        <div class="register-page__trust-item">
          <IconShieldCheck />
          <span>Open Source, here to last</span>
        </div>
        <div class="register-page__trust-item">
          <IconCheckCircle />
          <span>No credit card required</span>
        </div>
        <div class="register-page__trust-item">
          <IconCheckCircle />
          <span>GDPR compliant, EU hosted</span>
        </div>
      </div>

      <!-- Client Logos -->
      <div class="register-page__clients">
        <p class="register-page__clients-label">Trusted by teams at</p>
        <div class="register-page__clients-logos">
          <a
            href="https://github.com/France-Nuage/plateforme"
            target="_blank"
            rel="noopener noreferrer"
            class="register-page__client-logo"
            title="France Nuage"
          >
            <LogoFranceNuage />
          </a>
          <a
            href="https://www.woodwing.com/"
            target="_blank"
            rel="noopener noreferrer"
            class="register-page__client-logo"
            title="WoodWing"
          >
            <LogoWoodWing />
          </a>
          <a
            href="https://www.optery.com/"
            target="_blank"
            rel="noopener noreferrer"
            class="register-page__client-logo"
            title="Optery"
          >
            <LogoOptery />
          </a>
          <a
            href="https://www.icona.it/"
            target="_blank"
            rel="noopener noreferrer"
            class="register-page__client-logo"
            title="Icona"
          >
            <LogoIcona />
          </a>
          <a
            href="https://okoora.com/"
            target="_blank"
            rel="noopener noreferrer"
            class="register-page__client-logo"
            title="Okoora"
          >
            <LogoOkoora />
          </a>
          <a
            href="https://www.activeants.com/fr/"
            target="_blank"
            rel="noopener noreferrer"
            class="register-page__client-logo"
            title="ActiveAnts"
          >
            <LogoActiveAnts />
          </a>
        </div>
      </div>

      <!-- Testimonial -->
      <div class="register-page__testimonial">
        <blockquote class="register-page__testimonial-quote">
          "Hook0 replaced our homegrown webhook system in a day. Delivery reliability went from 94%
          to 99.9%."
        </blockquote>
        <div class="register-page__testimonial-author">
          <div class="register-page__testimonial-avatar">M</div>
          <div class="register-page__testimonial-info">
            <span class="register-page__testimonial-name">Marc D.</span>
            <span class="register-page__testimonial-role">CTO, SaaS Startup</span>
          </div>
        </div>
      </div>

      <!-- Social Proof -->
      <div class="register-page__social-proof">
        <span>Join 500+ companies using Hook0</span>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.register-page {
  @apply min-h-screen w-full relative overflow-hidden;
  background: linear-gradient(
    180deg,
    theme('colors.surface.primary') 0%,
    theme('colors.surface.secondary') 100%
  );
  font-family:
    'Inter',
    system-ui,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    Roboto,
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;

  &__background {
    @apply absolute inset-0 pointer-events-none;
  }

  &__grid-pattern {
    @apply absolute inset-0;
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.02) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.02) 1px, transparent 1px);
    background-size: 60px 60px;
    opacity: 0.5;
  }

  &__blur-circle {
    @apply absolute rounded-full pointer-events-none;
    filter: blur(100px);

    &--indigo {
      @apply bg-indigo-500;
      width: 400px;
      height: 400px;
      top: -200px;
      left: -200px;
      opacity: 0.15;
    }

    &--green {
      @apply bg-green-500;
      width: 350px;
      height: 350px;
      bottom: -150px;
      right: -150px;
      opacity: 0.1;
    }
  }

  &__content {
    @apply relative z-10 min-h-screen flex flex-col items-center justify-center px-4 py-12;
  }

  &__logo {
    @apply mb-8;
  }

  &__card {
    @apply w-full max-w-md p-8 rounded-2xl;
    background: theme('colors.surface.tertiary');
    border: 1px solid rgba(255, 255, 255, 0.05);
    position: relative;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);

    &::before {
      content: '';
      position: absolute;
      inset: 0;
      border-radius: inherit;
      opacity: 0;
      transition: opacity 0.3s ease;
      background: radial-gradient(
        600px circle at var(--mouse-x, 50%) var(--mouse-y, 50%),
        rgba(99, 102, 241, 0.15),
        transparent 40%
      );
      pointer-events: none;
    }

    &:hover::before {
      opacity: 1;
    }

    &:hover {
      border-color: rgba(255, 255, 255, 0.1);
      transform: translateY(-2px);
    }
  }

  &__header {
    @apply text-center mb-8;
  }

  &__title {
    @apply text-2xl font-bold text-white mb-2;
  }

  &__subtitle {
    @apply text-gray-400 text-sm;

    &--highlight {
      @apply text-green-400 font-medium;
    }
  }

  &__benefits {
    @apply mt-4 space-y-2 text-left;
  }

  &__benefit {
    @apply flex items-center gap-2 text-sm text-gray-300;

    svg {
      @apply flex-shrink-0;
    }
  }

  &__form {
    @apply space-y-5;
  }

  &__field {
    @apply space-y-2;

    &--half {
      @apply flex-1;
    }
  }

  &__field-row {
    @apply flex gap-4;
  }

  &__label {
    @apply block text-sm font-medium text-gray-300;
  }

  &__input {
    @apply block w-full px-4 py-3 text-sm rounded-xl text-white transition-all outline-none;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);

    &::placeholder {
      @apply text-gray-500;
    }

    &:focus {
      @apply border-indigo-500 ring-2 ring-indigo-500 ring-offset-2;
      --tw-ring-offset-color: theme('colors.surface.tertiary');
    }

    &:disabled {
      @apply opacity-50 cursor-not-allowed;
    }

    &--password {
      @apply pr-12;
    }
  }

  &__password-wrapper {
    @apply relative;
  }

  &__password-toggle {
    @apply absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white transition-colors p-1 rounded;

    &:focus {
      @apply outline-none ring-2 ring-indigo-500;
    }
  }

  &__turnstile {
    @apply flex justify-center;
  }

  &__submit {
    @apply w-full py-3 px-6 rounded-xl font-semibold text-white transition-all bg-green-500;

    &:hover:not(:disabled) {
      @apply bg-green-400;
      transform: translateY(-1px);
      box-shadow: 0 0 40px rgba(34, 197, 94, 0.3);
    }

    &:focus {
      @apply outline-none ring-2 ring-green-500 ring-offset-2;
      --tw-ring-offset-color: theme('colors.surface.tertiary');
    }

    &:disabled {
      @apply opacity-70 cursor-not-allowed;
      transform: none;
      box-shadow: none;
    }
  }

  &__loading {
    @apply flex items-center justify-center gap-2;
  }

  &__divider {
    @apply relative my-6 flex items-center justify-center;

    &::before,
    &::after {
      content: '';
      @apply flex-1;
      border-top: 1px solid rgba(255, 255, 255, 0.1);
    }

    span {
      @apply px-4 text-sm text-gray-500;
    }
  }

  &__signin {
    @apply flex items-center justify-center w-full py-3 px-6 rounded-xl font-medium text-white transition-all;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);

    &:hover {
      background: rgba(255, 255, 255, 0.15);
      transform: translateY(-1px);
    }

    &:focus {
      @apply outline-none ring-2 ring-white/50 ring-offset-2;
      --tw-ring-offset-color: theme('colors.surface.tertiary');
    }

    svg {
      @apply transition-transform;
    }

    &:hover svg {
      transform: translateX(4px);
    }
  }

  &__trust {
    @apply flex items-center justify-center gap-6 mt-8 flex-wrap;
  }

  &__trust-item {
    @apply flex items-center gap-2 text-sm text-gray-400;
  }

  &__social-proof {
    @apply mt-4 text-center text-sm text-gray-500;

    span {
      @apply px-3 py-1 rounded-full;
      background: rgba(99, 102, 241, 0.1);
      border: 1px solid rgba(99, 102, 241, 0.2);
    }
  }

  &__clients {
    @apply mt-8 text-center;
  }

  &__clients-label {
    @apply text-xs text-gray-500 uppercase tracking-wider mb-4;
  }

  &__clients-logos {
    @apply flex items-center justify-center gap-x-8 gap-y-4 flex-wrap;
  }

  &__client-logo {
    @apply text-gray-400 opacity-60 hover:opacity-100 transition-all;
    height: 24px;
    display: flex;
    align-items: center;

    svg {
      @apply h-full w-auto;
    }
  }

  @media (max-width: 640px) {
    &__clients-logos {
      @apply gap-x-6 gap-y-3;
    }

    &__client-logo {
      height: 18px;
    }
  }

  &__testimonial {
    @apply mt-8 max-w-md mx-auto text-center;
  }

  &__testimonial-quote {
    @apply text-sm text-gray-300 italic leading-relaxed mb-4;
  }

  &__testimonial-author {
    @apply flex items-center justify-center gap-3;
  }

  &__testimonial-avatar {
    @apply w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold text-white;
    background: linear-gradient(135deg, theme('colors.indigo.500'), theme('colors.purple.500'));
  }

  &__testimonial-info {
    @apply flex flex-col items-start text-left;
  }

  &__testimonial-name {
    @apply text-sm font-medium text-white;
  }

  &__testimonial-role {
    @apply text-xs text-gray-500;
  }
}

@media (max-width: 640px) {
  .register-page {
    &__card {
      @apply p-6;
    }

    &__field-row {
      @apply flex-col gap-5;
    }

    &__trust {
      @apply flex-col gap-3;
    }
  }
}
</style>
