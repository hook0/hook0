<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { toast } from 'vue-sonner';
import { Problem } from '@/http';
import { resetPassword } from '@/pages/user/UserService';
import { passwordRejection } from '@/utils/passwordProblem';
import { DEFAULT_PASSWORD_MINIMUM_LENGTH, PASSWORD_MAXIMUM_LENGTH } from '@/utils/passwordPolicy';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { routes } from '@/routes';
import router from '@/router';
import { stripTokenFromUrl } from '@/utils/stripTokenFromUrl';
import { useI18n } from 'vue-i18n';
import { ArrowLeft } from 'lucide-vue-next';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Logo from '@/components/Hook0Logo.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();

// Form state
const new_password = ref<string>('');
const confirm_new_password = ref<string>('');
const isLoading = ref<boolean>(false);
// Empty until the password is refused; shown under the field it refers to.
const passwordError = ref<string>('');
// Whether retrying can still work. A refused password is worth another try; a
// missing or expired link is not, and offering a form under that error invites
// the user to fix something no password can fix.
const linkIsUsable = ref<boolean>(true);

// The floor is operator configuration; the ceiling is not, so it is mirrored.
// This page never learns the account's address — only the token — so the
// identity rules cannot run here, but the length rules can, and finding out
// about them from a round trip is the worst way to find out.
const { data: instanceConfig } = useInstanceConfig();
const passwordMinimumLength = computed(() => {
  const config = instanceConfig.value;
  if (config === undefined) {
    return DEFAULT_PASSWORD_MINIMUM_LENGTH;
  }
  return config.password_minimum_length;
});
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

function submit() {
  if (isLoading.value) return;

  // A retry starts clean: leaving the previous rejection on screen while the
  // user types a new password reads as if it had been refused again.
  alert.value.visible = false;
  passwordError.value = '';

  if (new_password.value !== confirm_new_password.value) {
    toast.warning(t('common.warning'), {
      description: t('auth.resetPassword.passwordsMismatch'),
      duration: 5000,
    });
    return;
  }

  // The two rules this page can check on its own. Everything else — the
  // blocklist, the account's own address and name — only the server knows.
  const length = [...new_password.value].length;
  if (length < passwordMinimumLength.value) {
    passwordError.value = t('validation.passwordMinLength', {
      count: passwordMinimumLength.value,
    });
    return;
  }
  if (length > PASSWORD_MAXIMUM_LENGTH) {
    passwordError.value = t('validation.passwordMaxLength', {
      count: PASSWORD_MAXIMUM_LENGTH,
    });
    return;
  }

  isLoading.value = true;

  resetPassword(token, new_password.value)
    .then(() => {
      toast.success(t('common.success'), {
        description: t('auth.resetPassword.resetSuccess'),
        duration: 5000,
      });
      return router.push({ name: routes.Login });
    })
    .catch((err) => {
      displayError(err as Problem);
      const rejection = passwordRejection(err);
      if (rejection.refused) {
        passwordError.value = rejection.reason;
        return;
      }
      // Only the link being dead removes the form. A server that was busy or
      // unreachable says nothing about the link, and taking the form away for
      // it would cost the user their reset for an outage that lasted a second.
      if (isDeadLink(err)) {
        linkIsUsable.value = false;
      }
    })
    .finally(() => {
      isLoading.value = false;
    });
}

/**
 * The errors that mean this particular link will never work again, whatever
 * the user types: a token the API cannot read or has expired, one it refuses
 * to authorize, and the client-side case of no token at all. Everything else —
 * a busy server, a dropped connection — is worth another attempt, so the form
 * stays.
 *
 * Raised by `reset_password` in api/src/handlers/auth.rs.
 */
const DEAD_LINK_PROBLEM_IDS: ReadonlySet<string> = new Set([
  'AuthEmailExpired',
  'Forbidden',
  'InvalidToken',
]);

function isDeadLink(err: unknown): boolean {
  if (err === null || typeof err !== 'object') {
    return false;
  }
  const id: unknown = (err as Record<string, unknown>).id;
  return typeof id === 'string' && DEAD_LINK_PROBLEM_IDS.has(id);
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
  stripTokenFromUrl(router);
  if (!token) {
    linkIsUsable.value = false;
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: t('auth.resetPassword.invalidToken'),
      detail: t('auth.resetPassword.tokenRequired'),
    });
  }
}

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0PageLayout variant="fullscreen">
    <template #logo>
      <Hook0Logo variant="banner-white" size="lg" />
    </template>

    <!-- Error Card -->
    <Hook0Card v-if="alert.visible" variant="glow">
      <Hook0CardContent>
        <Hook0Alert :type="alert.type" :title="alert.title" :description="alert.description" />

        <Hook0Button variant="ghost" size="lg" :to="{ name: routes.Login }" full-width>
          <template #left>
            <ArrowLeft :size="16" aria-hidden="true" />
          </template>
          {{ t('auth.resetPassword.backToLogin') }}
        </Hook0Button>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Form Card. Deliberately not the `v-else` of the card above: a refused
         password is a mistake the user can fix in place, and unmounting the
         form would cost them their reset link for it. It goes away only when
         retrying cannot help — a missing or expired link. -->
    <Hook0Card v-if="linkIsUsable" variant="glow">
      <Hook0CardHeader
        variant="centered"
        :title="t('auth.resetPassword.title')"
        :subtitle="t('auth.resetPassword.subtitle')"
      />

      <Hook0CardContent>
        <Hook0Form data-test="reset-password-form" :loading="isLoading" @submit="submit">
          <Hook0Input
            id="new_password"
            v-model="new_password"
            type="password"
            required
            show-password-toggle
            :label="t('auth.resetPassword.password')"
            :placeholder="t('auth.resetPassword.passwordPlaceholder')"
            autocomplete="new-password"
            :disabled="isLoading"
            :error="passwordError"
            data-test="reset-password-new-password-input"
          >
            <template #helpText>{{
              t('validation.passwordRequirements', { count: passwordMinimumLength })
            }}</template>
          </Hook0Input>

          <Hook0Input
            id="confirm_password"
            v-model="confirm_new_password"
            type="password"
            required
            show-password-toggle
            :label="t('auth.resetPassword.confirmPassword')"
            :placeholder="t('auth.resetPassword.confirmPasswordPlaceholder')"
            autocomplete="new-password"
            :disabled="isLoading"
            data-test="reset-password-confirm-password-input"
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
            {{ isLoading ? t('auth.resetPassword.submitting') : t('auth.resetPassword.submit') }}
          </Hook0Button>
        </Hook0Form>
      </Hook0CardContent>
    </Hook0Card>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0PageLayout variant="fullscreen" handles card width (28rem max) */
</style>
