<script setup lang="ts">
import { ref } from 'vue';
import { push } from 'notivue';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { beginResetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes.ts';
import { useCardGlow } from '@/composables/useCardGlow';

// Form state
const email = ref<string>('');
const isLoading = ref<boolean>(false);

// Mouse tracking for card glow effect
const { cardRef, mouseX, mouseY, handleMouseMove } = useCardGlow();

function submit() {
  if (isLoading.value) return;
  isLoading.value = true;

  beginResetPassword(email.value)
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Email sent successfully. Please check your email to reset your password.',
        duration: 5000,
      });
    })
    .catch((err) => {
      const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
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
  <div class="reset-page">
    <!-- Background Effects -->
    <div class="reset-page__background">
      <div class="reset-page__grid-pattern"></div>
      <div class="reset-page__blur-circle reset-page__blur-circle--indigo"></div>
      <div class="reset-page__blur-circle reset-page__blur-circle--green"></div>
    </div>

    <!-- Main Content -->
    <div class="reset-page__content">
      <!-- Logo -->
      <div class="reset-page__logo">
        <img src="/logo.svg" alt="Hook0" class="h-12 w-auto" />
      </div>

      <!-- Card -->
      <div
        ref="cardRef"
        class="reset-page__card"
        :style="{ '--mouse-x': mouseX, '--mouse-y': mouseY }"
        @mousemove="handleMouseMove"
      >
        <!-- Header -->
        <div class="reset-page__header">
          <h1 class="reset-page__title">Reset your password</h1>
          <p class="reset-page__subtitle">
            Enter your email and we'll send you a link to reset your password
          </p>
        </div>

        <!-- Form -->
        <form class="reset-page__form" @submit.prevent="submit">
          <!-- Email Field -->
          <div class="reset-page__field">
            <label for="email" class="reset-page__label">Email</label>
            <input
              id="email"
              v-model="email"
              type="email"
              required
              placeholder="you@company.com"
              class="reset-page__input"
              autocomplete="email"
              :disabled="isLoading"
            />
          </div>

          <!-- Submit Button -->
          <button type="submit" class="reset-page__submit" :disabled="isLoading">
            <span v-if="!isLoading">Send reset link</span>
            <span v-else class="reset-page__loading">
              <svg
                class="animate-spin h-5 w-5"
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
              Sending...
            </span>
          </button>
        </form>

        <!-- Back to login link -->
        <router-link :to="{ name: routes.Login }" class="reset-page__back">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4 mr-2"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M11 17l-5-5m0 0l5-5m-5 5h12"
            />
          </svg>
          Back to login
        </router-link>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.reset-page {
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
  }

  &__form {
    @apply space-y-5;
  }

  &__field {
    @apply space-y-2;
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

  &__back {
    @apply flex items-center justify-center w-full py-3 px-6 mt-6 rounded-xl font-medium text-gray-400 transition-all;

    &:hover {
      @apply text-white;
    }

    &:focus {
      @apply outline-none text-white;
    }

    svg {
      @apply transition-transform;
    }

    &:hover svg {
      transform: translateX(-4px);
    }
  }
}

@media (max-width: 640px) {
  .reset-page {
    &__card {
      @apply p-6;
    }
  }
}
</style>
