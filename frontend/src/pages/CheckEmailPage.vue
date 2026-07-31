<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';
import { Mail, LifeBuoy, ArrowLeft, Send } from 'lucide-vue-next';

import * as UserService from '@/pages/user/UserService';
import { remainingCooldownSeconds } from '@/utils/cooldown';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ButtonGroup from '@/components/Hook0ButtonGroup.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';

const { t } = useI18n();
const route = useRoute();

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();

// Seconds a user must wait between two resend attempts. Mirrors the server-side
// per-account cooldown so the button re-enables about when a new email would
// actually be sent again.
const RESEND_COOLDOWN_SECONDS = 60;

// Address to resend to, carried over from the signup redirect. Empty when the
// page is opened without it (e.g. a bookmarked /check-email): the resend action
// is simply hidden in that case.
const emailParam = route.query.email;
const email = typeof emailParam === 'string' ? emailParam : '';
const canResend = email.length > 0;

const isResending = ref<boolean>(false);
const cooldownStartedAt = ref<number>(0);
const nowMs = ref<number>(Date.now());
let ticker = 0;

const cooldownRemaining = computed<number>(() =>
  remainingCooldownSeconds(cooldownStartedAt.value, RESEND_COOLDOWN_SECONDS, nowMs.value)
);
const isCoolingDown = computed<boolean>(() => cooldownRemaining.value > 0);

function resend() {
  if (isResending.value || isCoolingDown.value || !canResend) {
    return;
  }
  isResending.value = true;
  trackEvent('signup', 'resend-verification-email', 'check-email');

  UserService.resendVerificationEmail(email)
    .then(() => {
      // Start the cooldown regardless of whether the address actually matched an
      // account: the endpoint answers identically either way (anti-enumeration),
      // so the UI must not behave differently.
      cooldownStartedAt.value = Date.now();
      nowMs.value = Date.now();
      toast.success(t('auth.checkEmail.resendSuccess'));
    })
    .catch(() => {
      toast.error(t('auth.checkEmail.resendError'));
    })
    .finally(() => {
      isResending.value = false;
    });
}

onMounted(() => {
  trackPageWithDimensions('auth', 'view', 'email-sent');
  trackEvent('signup', 'page-view', 'check-email');
  ticker = window.setInterval(() => {
    nowMs.value = Date.now();
  }, 1000);
});

onBeforeUnmount(() => {
  window.clearInterval(ticker);
});
</script>

<template>
  <Hook0PageLayout variant="fullscreen" data-test="check-email-page">
    <template #logo>
      <Hook0Logo variant="image" size="lg" />
    </template>

    <Hook0Card variant="glow">
      <Hook0CardContent>
        <Hook0EmptyState
          :title="t('auth.checkEmail.title')"
          :description="t('auth.checkEmail.description')"
        >
          <template #icon>
            <Mail :size="64" aria-hidden="true" />
          </template>
        </Hook0EmptyState>

        <Hook0Alert type="info">
          <template #description>
            {{ t('auth.checkEmail.spamNote') }}
          </template>
        </Hook0Alert>

        <Hook0ButtonGroup direction="column" gap="sm" full-width>
          <Hook0Button
            v-if="canResend"
            variant="primary"
            size="lg"
            :loading="isResending"
            :disabled="isResending || isCoolingDown"
            full-width
            data-test="resend-verification-email-button"
            @click="resend"
          >
            <template #left>
              <Send :size="18" aria-hidden="true" />
            </template>
            {{
              isCoolingDown
                ? t('auth.checkEmail.resendCooldown', { seconds: cooldownRemaining })
                : t('auth.checkEmail.resend')
            }}
          </Hook0Button>

          <Hook0Button
            :variant="canResend ? 'secondary' : 'primary'"
            size="lg"
            :to="{ name: routes.Login }"
            full-width
          >
            <template #left>
              <ArrowLeft :size="20" aria-hidden="true" />
            </template>
            {{ t('auth.checkEmail.backToLogin') }}
          </Hook0Button>

          <Hook0Button variant="ghost" size="lg" href="mailto:support@hook0.com" full-width>
            <template #left>
              <LifeBuoy :size="16" aria-hidden="true" />
            </template>
            {{ t('auth.checkEmail.contactSupport') }}
          </Hook0Button>
        </Hook0ButtonGroup>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
/* No custom styles - Hook0 components handle layout */
</style>
