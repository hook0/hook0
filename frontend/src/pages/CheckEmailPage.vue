<script setup lang="ts">
import { onMounted } from 'vue';
import { routes } from '@/routes.ts';
import { useCardGlow } from '@/composables/useCardGlow';
import { useTracking } from '@/composables/useTracking';

// Mouse tracking for card glow effect
const { cardRef, mouseX, mouseY, handleMouseMove } = useCardGlow();

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();

onMounted(() => {
  trackPageWithDimensions('auth', 'view', 'email-sent');
  trackEvent('Signup', 'PageView', 'check-email');
});
</script>

<template>
  <div class="check-email-page">
    <!-- Background Effects -->
    <div class="check-email-page__background">
      <div class="check-email-page__grid-pattern"></div>
      <div class="check-email-page__blur-circle check-email-page__blur-circle--indigo"></div>
      <div class="check-email-page__blur-circle check-email-page__blur-circle--green"></div>
    </div>

    <!-- Main Content -->
    <div class="check-email-page__content">
      <!-- Logo -->
      <div class="check-email-page__logo">
        <img src="/logo.svg" alt="Hook0" class="h-12 w-auto" />
      </div>

      <!-- Card -->
      <div
        ref="cardRef"
        class="check-email-page__card"
        :style="{ '--mouse-x': mouseX, '--mouse-y': mouseY }"
        @mousemove="handleMouseMove"
      >
        <!-- Icon -->
        <div class="check-email-page__icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-16 w-16"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="1.5"
              d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
            />
          </svg>
        </div>

        <!-- Header -->
        <div class="check-email-page__header">
          <h1 class="check-email-page__title">Check your email</h1>
          <p class="check-email-page__subtitle">
            We've sent a verification email to your address. Please check your inbox and confirm
            your email to get started.
          </p>
        </div>

        <!-- Info Box -->
        <div class="check-email-page__info">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5 flex-shrink-0"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span>Didn't receive the email? Check your spam folder or contact support.</span>
        </div>

        <!-- Actions -->
        <div class="check-email-page__actions">
          <a
            href="mailto:support@hook0.com"
            class="check-email-page__button check-email-page__button--primary"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 mr-2"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z"
              />
            </svg>
            Contact support
          </a>
          <router-link
            :to="{ name: routes.Login }"
            class="check-email-page__button check-email-page__button--secondary"
          >
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
  </div>
</template>

<style lang="scss" scoped>
.check-email-page {
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
    background-image: linear-gradient(rgba(255, 255, 255, 0.02) 1px, transparent 1px),
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

  &__icon {
    @apply flex justify-center mb-6 text-green-500;
  }

  &__header {
    @apply text-center mb-6;
  }

  &__title {
    @apply text-2xl font-bold text-white mb-3;
  }

  &__subtitle {
    @apply text-gray-400 text-sm leading-relaxed;
  }

  &__info {
    @apply flex items-start gap-3 p-4 rounded-xl mb-6 text-gray-400 text-sm;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  &__actions {
    @apply space-y-3;
  }

  &__button {
    @apply flex items-center justify-center w-full py-3 px-6 rounded-xl font-medium transition-all;

    &--primary {
      @apply text-white bg-green-500;

      &:hover {
        @apply bg-green-400;
        transform: translateY(-1px);
        box-shadow: 0 0 40px rgba(34, 197, 94, 0.3);
      }

      &:focus {
        @apply outline-none ring-2 ring-green-500 ring-offset-2;
        --tw-ring-offset-color: theme('colors.surface.tertiary');
      }
    }

    &--secondary {
      @apply text-gray-400;

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
}

@media (max-width: 640px) {
  .check-email-page {
    &__card {
      @apply p-6;
    }
  }
}
</style>
