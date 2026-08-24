<script setup lang="ts">
import { ref } from 'vue';
import { beginResetPassword } from '@/pages/user/UserService';
import { isRateLimited } from '@/utils/rateLimited';
import { isFieldRefused } from '@/utils/problem';
import { isUnfulfilled } from '@/utils/unfulfilled';
import { routes } from '@/routes';
import { useI18n } from 'vue-i18n';
import { ArrowLeft, CheckCircle, LifeBuoy, Pencil } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ButtonGroup from '@/components/Hook0ButtonGroup.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';

const { t } = useI18n();

// Mirrors `BeginResetPasswordPost::email` (api/src/handlers/auth.rs). The
// request struct refuses a longer address before anything is looked up, so
// the field stops it here rather than spending a round trip to be told.
const EMAIL_MAXIMUM_LENGTH = 100;

// Form state
const email = ref<string>('');
const isLoading = ref<boolean>(false);
const isSuccess = ref<boolean>(false);
// Empty until the request is refused for pacing.
const rateLimitError = ref<string>('');
// Empty until the address itself is refused, which is about the characters
// typed and not about the account behind them.
const emailError = ref<string>('');
// Empty until the request never reached an answer the server formulated.
const requestFailedError = ref<string>('');
/**
 * Move the focus onto the confirmation once it is really there.
 *
 * The confirmation takes the place of the form inside a `mode="out-in"`
 * transition, so it does not exist yet on the tick after the request
 * resolves — the element is only inserted once the form has finished
 * leaving. A focus call scheduled any earlier lands on nothing, which is how
 * the confirmation went up unannounced while the markup looked right: the
 * live region is the weaker half on its own, because it is mounted with its
 * content already inside it, and that is the case screen readers most often
 * stay silent through. The transition's own "entered" hook is the first
 * moment there is anything to focus.
 *
 * The result is checked rather than assumed. Nothing about a confirmation
 * that fails to take the focus looks wrong on screen, so the only way this
 * stops being invisible is for it to say so.
 */
function takeFocus(element: Element) {
  if (!isSuccess.value) {
    // The form coming back from "use a different address"; its first field
    // takes the focus on its own.
    return;
  }
  if (element instanceof HTMLElement) {
    element.focus();
    if (document.activeElement === element) {
      return;
    }
  }
  console.error(
    'The password reset confirmation did not take the focus: a reader who cannot see it is left with no sign the request went through'
  );
}

function useDifferentAddress() {
  isSuccess.value = false;
  rateLimitError.value = '';
  emailError.value = '';
  requestFailedError.value = '';
}

function submit() {
  if (isLoading.value) return;
  isLoading.value = true;
  // A retry starts clean: leaving the previous refusal on screen while the
  // request is in flight reads as if it had just been refused again.
  rateLimitError.value = '';
  emailError.value = '';
  requestFailedError.value = '';

  beginResetPassword(email.value)
    .then(() => {
      isSuccess.value = true;
    })
    .catch((err: unknown) => {
      // Three failures are worth showing, and none is a judgement about the
      // address. The 429 comes from the limiter keyed on the caller's IP,
      // so it reads the same for an address that exists and one that does not
      // — showing it gives an attacker nothing. Staying silent about it, on the
      // other hand, promises a mail that will never be sent and closes a
      // legitimate user's only route back into their account.
      if (isRateLimited(err)) {
        rateLimitError.value = t('auth.forgotPassword.rateLimited');
        return;
      }

      // A refusal about the address as typed — too long, not an address at
      // all, control characters in it — is raised by the request struct's own
      // validators, which run before any lookup: the answer is the same for an
      // address that exists and one that does not, so it says nothing an
      // attacker could not have worked out from the characters they sent.
      // Reported as success instead, it is a typo the reader is told to wait
      // for a mail about.
      if (isFieldRefused(err, 'email')) {
        emailError.value = t('auth.forgotPassword.emailRefused', {
          count: EMAIL_MAXIMUM_LENGTH,
        });
        return;
      }

      // The request never got as far as a decision about the address: it timed
      // out, failed in transport, or the database call that claims the send
      // failed. Reporting success here promises a mail that was never minted,
      // on the one route back into an account. And it discloses nothing — none
      // of those outcomes depends on whether an account exists.
      if (isUnfulfilled(err)) {
        requestFailedError.value = t('auth.forgotPassword.requestFailed');
        return;
      }

      // Every other outcome is swallowed and reported as success. Whether the
      // address belongs to an account is precisely what the API stopped
      // answering; any difference this page draws from the failure — a toast, a
      // line in the console — hands that oracle straight back. Do not "simplify"
      // this into reporting the error: the silence is the fix.
      isSuccess.value = true;
    })
    .finally(() => {
      isLoading.value = false;
    });
}
</script>

<template>
  <Hook0PageLayout variant="fullscreen">
    <template #logo>
      <Hook0Logo variant="banner-white" size="lg" />
    </template>

    <Hook0Card>
      <Hook0CardHeader
        variant="centered"
        :title="t('auth.forgotPassword.title')"
        :subtitle="t('auth.forgotPassword.description')"
      />

      <Hook0CardContent>
        <Hook0Alert
          v-if="rateLimitError !== ''"
          type="warning"
          :description="rateLimitError"
          data-test="reset-password-rate-limited"
        />

        <Hook0Alert
          v-if="requestFailedError !== ''"
          type="warning"
          :description="requestFailedError"
          data-test="reset-password-request-failed"
        />

        <Transition name="swap" mode="out-in" @after-enter="takeFocus">
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
              :maxlength="EMAIL_MAXIMUM_LENGTH"
              :label="t('auth.forgotPassword.email')"
              :placeholder="t('auth.forgotPassword.emailPlaceholder')"
              autocomplete="email"
              :disabled="isLoading"
              :error="emailError"
              autofocus
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

          <div
            v-else
            class="reset-password__success"
            role="status"
            aria-live="polite"
            tabindex="-1"
            data-test="reset-password-success"
          >
            <div class="reset-password__success-icon">
              <CheckCircle :size="40" aria-hidden="true" />
            </div>
            <p class="reset-password__success-text">{{ t('auth.forgotPassword.emailSent') }}</p>
            <p class="reset-password__success-text">{{ t('auth.forgotPassword.spamNote') }}</p>

            <!-- The endpoint stopped saying whether the address is one it
                 knows, so a typo now looks exactly like a success. Retyping
                 the address is the only way left to recover from one, and
                 unmounting the form took it away. -->
            <Hook0ButtonGroup direction="column" gap="sm" full-width>
              <Hook0Button
                variant="secondary"
                size="lg"
                full-width
                data-test="reset-password-different-address"
                @click="useDifferentAddress"
              >
                <template #left>
                  <Pencil :size="16" aria-hidden="true" />
                </template>
                {{ t('auth.forgotPassword.useDifferentAddress') }}
              </Hook0Button>

              <Hook0Button variant="ghost" size="lg" href="mailto:support@hook0.com" full-width>
                <template #left>
                  <LifeBuoy :size="16" aria-hidden="true" />
                </template>
                {{ t('auth.forgotPassword.contactSupport') }}
              </Hook0Button>
            </Hook0ButtonGroup>
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
.reset-password__success:focus {
  outline: none;
}

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
