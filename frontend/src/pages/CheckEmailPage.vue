<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';
import { Mail, LifeBuoy, ArrowLeft, Send } from 'lucide-vue-next';

import * as UserService from '@/pages/user/UserService';
import {
  latestCooldownStart,
  readCooldownStart,
  remainingCooldownSeconds,
  writeCooldownStart,
  NO_COOLDOWN,
  type CooldownStart,
} from '@/utils/cooldown';
import { readCheckEmailHandover, type ResendTarget } from '@/utils/checkEmailHandover';
import { isRateLimited } from '@/utils/rateLimited';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ButtonGroup from '@/components/Hook0ButtonGroup.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';

const { t } = useI18n();

// Analytics tracking
const { trackEvent, trackPageWithDimensions } = useTracking();

// Seconds a user must wait between two resend attempts. Mirrors the server-side
// per-account cooldown so the button re-enables about when a new email would
// actually be sent again.
const RESEND_COOLDOWN_SECONDS = 60;

// Who to resend to, as handed over by the page that redirected here (signup, or
// a login refused because the address is unverified) — through History API
// state, never the URL, so the address never reaches analytics. Absence is a
// case of its own rather than an empty address: opened without a hand-off (a
// bookmarked /check-email), the resend action is simply not offered.
const resendTarget = readCheckEmailHandover(window.history.state);
const canResend = resendTarget.kind === 'address';

const isResending = ref<boolean>(false);

/**
 * Where the countdown stands on arrival: the later of what this browser recorded
 * when the button was last pressed, and the send the redirect declared it had
 * just made.
 *
 * Both matter. Storage is what survives a reload — the server-side cooldown
 * outlives the page, and since the endpoint answers 204 either way, a re-enabled
 * button would report a send that never happened. The declared send covers the
 * first arrival, where nothing is recorded yet and yet a verification email left
 * seconds ago: signing up sends one and stamps the account, so the very first
 * click would be throttled in silence.
 */
function initialCooldownStart(target: ResendTarget): CooldownStart {
  if (target.kind === 'none') {
    return NO_COOLDOWN;
  }
  const start = latestCooldownStart(
    readCooldownStart(window.sessionStorage, target.email),
    target.lastVerificationSend
  );
  if (start.kind === 'started') {
    // Record the hand-off too, so it still applies on an arrival that carries no
    // declaration of its own (coming back through the login form, say).
    writeCooldownStart(window.sessionStorage, target.email, start.atMs);
  }
  return start;
}

const cooldownStart = ref<CooldownStart>(initialCooldownStart(resendTarget));
const nowMs = ref<number>(Date.now());
let ticker = 0;

const cooldownRemaining = computed<number>(() =>
  remainingCooldownSeconds(cooldownStart.value, RESEND_COOLDOWN_SECONDS, nowMs.value)
);
const isCoolingDown = computed<boolean>(() => cooldownRemaining.value > 0);

function startCooldown(email: string) {
  const startedAtMs = Date.now();
  cooldownStart.value = { kind: 'started', atMs: startedAtMs };
  nowMs.value = startedAtMs;
  writeCooldownStart(window.sessionStorage, email, startedAtMs);
}

function resend() {
  if (isResending.value || isCoolingDown.value || resendTarget.kind !== 'address') {
    return;
  }
  const email = resendTarget.email;
  isResending.value = true;
  trackEvent('signup', 'resend-verification-email', 'check-email');

  UserService.resendVerificationEmail(email)
    .then(() => {
      // Start the cooldown regardless of whether the address actually matched an
      // account: the endpoint answers identically either way (anti-enumeration),
      // so the UI must not behave differently.
      //
      // For the same reason the message acknowledges the request and stops
      // there. A 204 means "accepted", never "sent": an unknown address, an
      // already-verified one, a cooldown still running or a spent daily
      // allowance all answer the same, with nothing leaving. Anything more
      // definite would either be a lie or tell the caller which addresses exist.
      startCooldown(email);
      toast.success(t('auth.checkEmail.resendSuccess'));
    })
    .catch((err: unknown) => {
      // A failed attempt also starts the cooldown: the rate limiter in front of
      // the endpoint is exactly what a user hammering the button runs into, and
      // retrying immediately can only fail again.
      startCooldown(email);
      // And that limiter is now a burst of five per IP rather than the hundred
      // a second it used to be, so the sixth person signing up from behind one
      // office address reaches it without doing anything wrong. "Try again in
      // a moment" sends them straight back into it; naming the case tells them
      // the address is fine and the wait is the whole of it.
      if (isRateLimited(err)) {
        toast.error(t('auth.checkEmail.resendRateLimited'));
        return;
      }
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
      <Hook0Logo variant="banner-white" size="lg" />
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
