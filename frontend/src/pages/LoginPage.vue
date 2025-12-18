<script setup lang="ts">
import { ref } from 'vue';
import { login } from '@/iam';
import { handleError, Problem } from '@/http.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import * as OrganizationService from './organizations/OrganizationService';
import * as ApplicationService from './organizations/applications/ApplicationService';

const router = useRouter();

// Form state
const email = ref<string>('');
const password = ref<string>('');
const showPassword = ref<boolean>(false);
const isLoading = ref<boolean>(false);

// Mouse tracking for card glow effect
const cardRef = ref<HTMLElement | null>(null);
const mouseX = ref<string>('50%');
const mouseY = ref<string>('50%');

function handleMouseMove(event: MouseEvent) {
  if (!cardRef.value) return;
  const rect = cardRef.value.getBoundingClientRect();
  mouseX.value = `${event.clientX - rect.left}px`;
  mouseY.value = `${event.clientY - rect.top}px`;
}

async function submit() {
  if (isLoading.value) return;
  isLoading.value = true;

  try {
    await login(email.value, password.value);

    push.success({
      title: 'Success',
      message: 'You have successfully logged in.',
      duration: 5000,
    });

    const organizations = await OrganizationService.list();

    if (organizations.length < 1) {
      return router.push({ name: routes.Tutorial });
    } else if (organizations.length === 1) {
      const applications = await ApplicationService.list(organizations[0].organization_id);
      if (applications.length < 1) {
        return router.push({ name: routes.Tutorial });
      } else {
        return router.push({ name: routes.Home });
      }
    } else {
      return router.push({ name: routes.Home });
    }
  } catch (err) {
    const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
    displayError(problem);
  } finally {
    isLoading.value = false;
  }
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
  <div class="login-page">
    <!-- Background Effects -->
    <div class="login-page__background">
      <div class="login-page__grid-pattern"></div>
      <div class="login-page__blur-circle login-page__blur-circle--indigo"></div>
      <div class="login-page__blur-circle login-page__blur-circle--green"></div>
    </div>

    <!-- Main Content -->
    <div class="login-page__content">
      <!-- Logo -->
      <div class="login-page__logo">
        <img src="/logo.svg" alt="Hook0" class="h-12 w-auto" />
      </div>

      <!-- Card -->
      <div
        ref="cardRef"
        class="login-page__card"
        :style="{ '--mouse-x': mouseX, '--mouse-y': mouseY }"
        @mousemove="handleMouseMove"
      >
        <!-- Header -->
        <div class="login-page__header">
          <h1 class="login-page__title">Welcome back</h1>
          <p class="login-page__subtitle">Sign in to your Hook0 account</p>
        </div>

        <!-- Form -->
        <form class="login-page__form" @submit.prevent="submit">
          <!-- Email Field -->
          <div class="login-page__field">
            <label for="email" class="login-page__label">Email</label>
            <input
              id="email"
              v-model="email"
              type="email"
              required
              autofocus
              placeholder="you@company.com"
              class="login-page__input"
              :disabled="isLoading"
            />
          </div>

          <!-- Password Field -->
          <div class="login-page__field">
            <label for="password" class="login-page__label">Password</label>
            <div class="login-page__password-wrapper">
              <input
                id="password"
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                required
                placeholder="Enter your password"
                class="login-page__input login-page__input--password"
                :disabled="isLoading"
              />
              <button
                type="button"
                class="login-page__password-toggle"
                :aria-label="showPassword ? 'Hide password' : 'Show password'"
                @click="togglePasswordVisibility"
              >
                <!-- Eye icon (show) -->
                <svg
                  v-if="!showPassword"
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-5 w-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
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
                <!-- Eye-off icon (hide) -->
                <svg
                  v-else
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-5 w-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
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

          <!-- Forgot password link -->
          <div class="login-page__options">
            <router-link :to="{ name: routes.BeginResetPassword }" class="login-page__forgot">
              Forgot password?
            </router-link>
          </div>

          <!-- Submit Button -->
          <button type="submit" class="login-page__submit" :disabled="isLoading">
            <span v-if="!isLoading">Sign in</span>
            <span v-else class="login-page__loading">
              <svg
                class="animate-spin h-5 w-5"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
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
              Signing in...
            </span>
          </button>
        </form>

        <!-- Divider -->
        <div class="login-page__divider">
          <span>New to Hook0?</span>
        </div>

        <!-- Sign up link -->
        <router-link :to="{ name: routes.Register }" class="login-page__signup">
          Create an account
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4 ml-2"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
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
      <div class="login-page__trust">
        <div class="login-page__trust-item">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5 text-green-500"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fill-rule="evenodd"
              d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clip-rule="evenodd"
            />
          </svg>
          <span>Secure & encrypted</span>
        </div>
        <div class="login-page__trust-item">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5 text-green-500"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clip-rule="evenodd"
            />
          </svg>
          <span>No credit card required</span>
        </div>
        <div class="login-page__trust-item">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5 text-green-500"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clip-rule="evenodd"
            />
          </svg>
          <span>Hosted in Europe</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.login-page {
  @apply min-h-screen w-full relative overflow-hidden;
  background: linear-gradient(180deg, #0a0a0f 0%, #111118 100%);
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

  // Background
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

  // Content container
  &__content {
    @apply relative z-10 min-h-screen flex flex-col items-center justify-center px-4 py-12;
  }

  // Logo
  &__logo {
    @apply mb-8;
  }

  // Card
  &__card {
    @apply w-full max-w-md p-8 rounded-2xl;
    background: #1a1a24;
    border: 1px solid rgba(255, 255, 255, 0.05);
    position: relative;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);

    // Glow effect on hover
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

  // Header
  &__header {
    @apply text-center mb-8;
  }

  &__title {
    @apply text-2xl font-bold text-white mb-2;
  }

  &__subtitle {
    @apply text-gray-400 text-sm;
  }

  // Form
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
      @apply border-indigo-500 ring-2 ring-indigo-500;
      --tw-ring-offset-width: 2px;
      --tw-ring-offset-color: #1a1a24;
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

  // Options row
  &__options {
    @apply flex items-center justify-end;
  }

  &__forgot {
    @apply text-sm text-indigo-400 hover:text-indigo-300 transition-colors;
  }

  // Submit button
  &__submit {
    @apply w-full py-3 px-6 rounded-xl font-semibold text-white transition-all;
    background: #22c55e;

    &:hover:not(:disabled) {
      @apply bg-green-400;
      transform: translateY(-1px);
      box-shadow: 0 0 40px rgba(34, 197, 94, 0.3);
    }

    &:focus {
      @apply outline-none ring-2 ring-green-500;
      --tw-ring-offset-width: 2px;
      --tw-ring-offset-color: #1a1a24;
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

  // Divider
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

  // Sign up link
  &__signup {
    @apply flex items-center justify-center w-full py-3 px-6 rounded-xl font-medium text-white transition-all;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);

    &:hover {
      background: rgba(255, 255, 255, 0.15);
      transform: translateY(-1px);
    }

    &:focus {
      @apply outline-none ring-2 ring-white/50;
      --tw-ring-offset-width: 2px;
      --tw-ring-offset-color: #1a1a24;
    }

    svg {
      @apply transition-transform;
    }

    &:hover svg {
      transform: translateX(4px);
    }
  }

  // Trust indicators
  &__trust {
    @apply flex items-center justify-center gap-6 mt-8 flex-wrap;
  }

  &__trust-item {
    @apply flex items-center gap-2 text-sm text-gray-400;
  }
}

// Responsive adjustments
@media (max-width: 640px) {
  .login-page {
    &__card {
      @apply p-6;
    }

    &__trust {
      @apply flex-col gap-3;
    }
  }
}
</style>
