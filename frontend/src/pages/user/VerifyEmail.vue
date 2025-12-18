<script setup lang="ts">
import * as UserService from './UserService.ts';
import { Problem } from '@/http.ts';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import { useCardGlow } from '@/composables/useCardGlow';

const route = useRoute();

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

// Mouse tracking for card glow effect
const { cardRef, mouseX, mouseY, handleMouseMove } = useCardGlow();

function displayError(err: Problem) {
  console.error(err);
  isLoading.value = false;
  alert.value.visible = true;
  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function displaySuccess() {
  push.success({
    title: 'Email verified',
    message: 'You are successfully verified.',
    duration: 5000,
  });
}

function _onLoad() {
  const token = route.query.token as string;
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: 'Invalid token',
      detail: 'Token is required to verify email',
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
  <div class="verify-page">
    <!-- Background Effects -->
    <div class="verify-page__background">
      <div class="verify-page__grid-pattern"></div>
      <div class="verify-page__blur-circle verify-page__blur-circle--indigo"></div>
      <div class="verify-page__blur-circle verify-page__blur-circle--green"></div>
    </div>

    <!-- Main Content -->
    <div class="verify-page__content">
      <!-- Logo -->
      <div class="verify-page__logo">
        <img src="/logo.svg" alt="Hook0" class="h-12 w-auto" />
      </div>

      <!-- Loading State -->
      <div v-if="isLoading && !alert.visible" class="verify-page__card">
        <div class="verify-page__loading">
          <svg
            class="animate-spin h-12 w-12 text-green-500"
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
          <p class="verify-page__loading-text">Verifying your email...</p>
        </div>
      </div>

      <!-- Error Card -->
      <div
        v-else-if="alert.visible"
        ref="cardRef"
        class="verify-page__card"
        :style="{ '--mouse-x': mouseX, '--mouse-y': mouseY }"
        @mousemove="handleMouseMove"
      >
        <div
          class="verify-page__alert"
          :class="{ 'verify-page__alert--warning': alert.type === 'warning' }"
        >
          <svg
            v-if="alert.type === 'alert'"
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
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
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div>
            <h3 class="verify-page__alert-title">{{ alert.title }}</h3>
            <p class="verify-page__alert-description">{{ alert.description }}</p>
          </div>
        </div>
        <router-link :to="{ name: routes.Login }" class="verify-page__back">
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
.verify-page {
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

  &__loading {
    @apply flex flex-col items-center justify-center py-8 gap-4;
  }

  &__loading-text {
    @apply text-gray-400 text-sm;
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
  .verify-page {
    &__card {
      @apply p-6;
    }
  }
}
</style>
