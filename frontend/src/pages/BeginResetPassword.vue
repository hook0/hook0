<script setup lang="ts">
import { ref } from 'vue';
import { useAuthErrorHandler } from '@/composables/useAuthErrorHandler';
import { beginResetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes';
import { useI18n } from 'vue-i18n';
import { ArrowLeft, CheckCircle } from 'lucide-vue-next';

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
const isSuccess = ref<boolean>(false);

function submit() {
  if (isLoading.value || isSuccess.value) return;
  isLoading.value = true;

  beginResetPassword(email.value)
    .then(() => {
      isSuccess.value = true;
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
        <Transition name="swap" mode="out-in">
          <Hook0Form
            v-if="!isSuccess"
            data-test="reset-password-form"
            :loading="isLoading"
            @submit="submit"
          >
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
              {{
                isLoading ? t('auth.forgotPassword.submitting') : t('auth.forgotPassword.submit')
              }}
            </Hook0Button>
          </Hook0Form>

          <div v-else class="reset-password__success" data-test="reset-password-success">
            <div class="reset-password__success-icon">
              <CheckCircle :size="40" aria-hidden="true" />
            </div>
            <p class="reset-password__success-text">{{ t('auth.forgotPassword.emailSent') }}</p>
          </div>
        </Transition>

        <Hook0Button
          variant="link"
          size="lg"
          :to="{ name: routes.Login }"
          full-width
          class="reset-password__back-link"
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
.reset-password__back-link {
  margin-top: 0.75rem;
}

/* Success state */
.reset-password__success {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 0;
  text-align: center;
}

.reset-password__success-icon {
  color: var(--color-success);
  display: flex;
  align-items: center;
  justify-content: center;
}

.reset-password__success-text {
  font-size: 0.9375rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

/* Form exit — fade up */
.swap-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}
.swap-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

/* Success enter — scale + fade + blur (Jakub recipe) */
.swap-enter-active {
  transition:
    opacity 0.35s cubic-bezier(0.34, 1.56, 0.64, 1),
    transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1),
    filter 0.35s ease;
}
.swap-enter-from {
  opacity: 0;
  transform: scale(0.72);
  filter: blur(4px);
}

/* Check icon pop animation */
.reset-password__success-icon svg {
  animation: icon-pop 0.45s cubic-bezier(0.34, 1.56, 0.64, 1) 0.2s both;
}

@keyframes icon-pop {
  from {
    transform: scale(0.4);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .swap-leave-active,
  .swap-enter-active {
    transition: opacity 0.15s ease;
  }
  .swap-enter-from {
    transform: none;
    filter: none;
  }
  .reset-password__success-icon svg {
    animation: none;
  }
}
</style>
