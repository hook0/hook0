<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { push } from 'notivue';
import { Problem } from '@/http.ts';
import { resetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes.ts';
import router from '@/router.ts';

// Form state
const new_password = ref<string>('');
const confirm_new_password = ref<string>('');
const showPassword = ref<boolean>(false);
const showConfirmPassword = ref<boolean>(false);
const isLoading = ref<boolean>(false);
let token: string = '';

// Alert state
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

// Mouse tracking for card glow effect
const cardRef = ref<HTMLElement | null>(null);
const mouseX = ref<string>('50%');
const mouseY = ref<string>('50%');
let rafId: number | null = null;

function handleMouseMove(event: MouseEvent) {
  if (!cardRef.value) return;
  if (rafId !== null) return;

  rafId = requestAnimationFrame(() => {
    if (!cardRef.value) {
      rafId = null;
      return;
    }
    const rect = cardRef.value.getBoundingClientRect();
    mouseX.value = `${event.clientX - rect.left}px`;
    mouseY.value = `${event.clientY - rect.top}px`;
    rafId = null;
  });
}

async function submit() {
  if (isLoading.value) return;

  if (new_password.value !== confirm_new_password.value) {
    push.warning({
      title: 'Warning',
      message: 'Passwords do not match.',
      duration: 5000,
    });
    return;
  }

  isLoading.value = true;

  try {
    await resetPassword(token, new_password.value);
    push.success({
      title: 'Success',
      message: 'Your password has been reset successfully. Please login.',
      duration: 5000,
    });
    await router.push({ name: routes.Login });
  } catch (err) {
    displayError(err as Problem);
  } finally {
    isLoading.value = false;
  }
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;
  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function _onLoad() {
  token = router.currentRoute.value.query.token as string;
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: 'Invalid token',
      detail: 'Token is required to reset password',
    });
  }
}

onMounted(() => {
  _onLoad();
});
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

      <!-- Error Card -->
      <div v-if="alert.visible" class="reset-page__card reset-page__card--error">
        <div
          class="reset-page__alert"
          :class="{ 'reset-page__alert--warning': alert.type === 'warning' }"
        >
          <svg
            v-if="alert.type === 'alert'"
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
          <svg
            v-else
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div>
            <h3 class="reset-page__alert-title">{{ alert.title }}</h3>
            <p class="reset-page__alert-description">{{ alert.description }}</p>
          </div>
        </div>
        <router-link :to="{ name: routes.Login }" class="reset-page__back">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4 mr-2"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
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

      <!-- Form Card -->
      <div
        v-else
        ref="cardRef"
        class="reset-page__card"
        :style="{ '--mouse-x': mouseX, '--mouse-y': mouseY }"
        @mousemove="handleMouseMove"
      >
        <!-- Header -->
        <div class="reset-page__header">
          <h1 class="reset-page__title">Set new password</h1>
          <p class="reset-page__subtitle">Enter your new password below</p>
        </div>

        <!-- Form -->
        <form class="reset-page__form" @submit.prevent="submit">
          <!-- New Password Field -->
          <div class="reset-page__field">
            <label for="new_password" class="reset-page__label">New password</label>
            <div class="reset-page__password-wrapper">
              <input
                id="new_password"
                v-model="new_password"
                :type="showPassword ? 'text' : 'password'"
                required
                placeholder="Enter your new password"
                class="reset-page__input reset-page__input--password"
                :disabled="isLoading"
              />
              <button
                type="button"
                class="reset-page__password-toggle"
                :aria-label="showPassword ? 'Hide password' : 'Show password'"
                @click="showPassword = !showPassword"
              >
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

          <!-- Confirm Password Field -->
          <div class="reset-page__field">
            <label for="confirm_password" class="reset-page__label">Confirm new password</label>
            <div class="reset-page__password-wrapper">
              <input
                id="confirm_password"
                v-model="confirm_new_password"
                :type="showConfirmPassword ? 'text' : 'password'"
                required
                placeholder="Confirm your new password"
                class="reset-page__input reset-page__input--password"
                :disabled="isLoading"
              />
              <button
                type="button"
                class="reset-page__password-toggle"
                :aria-label="showConfirmPassword ? 'Hide password' : 'Show password'"
                @click="showConfirmPassword = !showConfirmPassword"
              >
                <svg
                  v-if="!showConfirmPassword"
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

          <!-- Submit Button -->
          <button type="submit" class="reset-page__submit" :disabled="isLoading">
            <span v-if="!isLoading">Set new password</span>
            <span v-else class="reset-page__loading">
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
              Updating...
            </span>
          </button>
        </form>
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

    &--error {
      &::before {
        display: none;
      }

      &:hover {
        transform: none;
      }
    }
  }

  &__alert {
    @apply flex items-start gap-4 p-4 rounded-xl mb-6 text-red-400;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);

    &--warning {
      @apply text-yellow-400;
      background: rgba(234, 179, 8, 0.1);
      border-color: rgba(234, 179, 8, 0.2);
    }
  }

  &__alert-title {
    @apply font-semibold text-sm mb-1;
  }

  &__alert-description {
    @apply text-sm opacity-80;
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
    @apply flex items-center justify-center w-full py-3 px-6 rounded-xl font-medium text-gray-400 transition-all;

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
