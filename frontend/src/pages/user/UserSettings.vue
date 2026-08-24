<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { toast } from 'vue-sonner';
import { User, Lock, AlertTriangle, Trash2, Palette } from 'lucide-vue-next';

import * as UserService from '@/pages/user/UserService';
import { createPasswordChangeSchema } from '@/pages/user/passwordChange.schema';
import { changePasswordFailure } from '@/pages/user/changePasswordFailure';
import { DEFAULT_PASSWORD_MINIMUM_LENGTH, type UserIdentity } from '@/utils/passwordPolicy';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { toTypedSchema } from '@/utils/zod-adapter';
import { useAuthStore } from '@/stores/auth';
import { useUiStore } from '@/stores/ui';
import type { ColorMode } from '@/stores/ui';
import { handleMutationError } from '@/utils/handleMutationError';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';

const { t } = useI18n();
const authStore = useAuthStore();
const uiStore = useUiStore();
const currentUser = computed(() => authStore.userInfo);

// Appearance preferences
const colorModeValue = computed({
  get: () => uiStore.colorMode,
  set: (value: string) => uiStore.setColorMode(value as ColorMode),
});

const colorModeOptions = computed(() => [
  { label: t('userSettings.themeSystem'), value: 'system' },
  { label: t('userSettings.themeLight'), value: 'light' },
  { label: t('userSettings.themeDark'), value: 'dark' },
]);

// Who the account belongs to, so the new password can be checked against it.
// The schema is recomputed when the user finishes loading; until then there is
// nothing to compare against, and the API runs the whole policy on submit
// anyway.
const passwordIdentity = computed<UserIdentity>(() => {
  const user = currentUser.value;
  if (user === null) {
    return { email: '', firstName: '', lastName: '' };
  }
  return { email: user.email, firstName: user.firstName, lastName: user.lastName };
});

// The length floor is operator configuration; read it rather than guess it.
const { data: instanceConfig } = useInstanceConfig();
const passwordMinimumLength = computed(() => {
  const config = instanceConfig.value;
  if (config === undefined) {
    return DEFAULT_PASSWORD_MINIMUM_LENGTH;
  }
  return config.password_minimum_length;
});

// VeeValidate form with Zod schema for password change
const { errors, meta, defineField, handleSubmit, resetForm, setFieldError } = useForm({
  validationSchema: computed(() =>
    toTypedSchema(createPasswordChangeSchema(passwordIdentity.value, passwordMinimumLength.value))
  ),
});

const [currentPassword, currentPasswordAttrs] = defineField('current_password');
const [newPassword, newPasswordAttrs] = defineField('new_password');
const [confirmNewPassword, confirmNewPasswordAttrs] = defineField('confirm_new_password');

// The request runs two Argon2 hashes in series — one to check the password
// being replaced, one to store the password replacing it — and both are slow
// on purpose. Without a state to show for it the form looks untouched for the
// best part of a second, and a second click really does leave: serialized
// behind the first, it arrives after the change and presents a current
// password that no longer opens the account, so the user reads the success and
// the refusal at once.
const isSubmitting = ref(false);

const onChangePassword = handleSubmit((values) => {
  if (isSubmitting.value) {
    return;
  }
  isSubmitting.value = true;

  // Both outcomes of the request are handled here rather than in a trailing
  // `.catch`, which would also catch the navigation below: a router hiccup
  // after the password was changed would then be read as the change failing,
  // and put a refusal on a form the user has already left behind.
  UserService.changePassword(values.current_password, values.new_password)
    .then(
      () => {
        toast.success(t('common.success'), {
          description: t('userSettings.passwordChanged'),
          duration: 5000,
        });
        resetForm();
        // The change ends every session the account had, this one included
        // (`store_new_password`, api/src/handlers/auth.rs). Left where they
        // are, the user learns that from whichever request happens to fail
        // next; clearing the tokens takes them straight to the form they now
        // have to use.
        return authStore.clearTokens();
      },
      (err: unknown) => {
        // Which field a refusal belongs under is decided in one place, because
        // the errors this endpoint answers with do not all name a field and
        // the two passwords it carries are not interchangeable.
        const failure = changePasswordFailure(err, t('userSettings.currentPasswordRejected'));
        if (failure.currentPassword.shown) {
          setFieldError('current_password', failure.currentPassword.message);
        }
        if (failure.newPassword.shown) {
          setFieldError('new_password', failure.newPassword.message);
        }
        // Only what no field on the form can account for: a reason already
        // sitting under the field the user has to fix does not need repeating
        // over the top of it.
        if (failure.unexplained) {
          handleMutationError(err);
        }
      }
    )
    .finally(() => {
      isSubmitting.value = false;
    });
});

// The button is gated on the form being valid and touched, and vee-validate
// only writes a message for fields the user has already visited: someone who
// fills in the two new passwords and never touches the current one sees a grey
// button and no error anywhere. A native `<button disabled>` swallows the
// click too, so there is not even a late explanation — hence the reason on
// hover and on focus, which is what the tooltip branch of Hook0Button exists
// for.
const isSubmitBlocked = computed(
  () => isSubmitting.value || !meta.value.valid || !meta.value.dirty
);
const submitHint = computed(() =>
  isSubmitBlocked.value ? t('userSettings.changePasswordBlocked') : t('userSettings.changePassword')
);

const showDeleteAccountDialog = ref(false);

function deleteAccount() {
  showDeleteAccountDialog.value = true;
}

function confirmDeleteAccount() {
  showDeleteAccountDialog.value = false;

  UserService.deleteUser()
    .then(() => {
      toast.success(t('common.success'), {
        description: t('userSettings.accountDeleted'),
        duration: 3000,
      });
      setTimeout(() => {
        void authStore.logout();
      }, 3000);
    })
    .catch(handleMutationError);
}
</script>

<template>
  <Hook0PageLayout :title="t('userSettings.title')">
    <!-- Personal Information -->
    <Hook0Card v-if="currentUser" data-test="user-info-card">
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0IconBadge variant="info">
              <User :size="18" aria-hidden="true" />
            </Hook0IconBadge>
            <span>{{ t('userSettings.personalInfo') }}</span>
          </Hook0Stack>
        </template>
        <template #subtitle>{{ t('userSettings.personalInfoSubtitle') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine>
          <template #label>{{ t('userSettings.email') }}</template>
          <template #content>
            <Hook0Input
              v-model="currentUser.email"
              type="text"
              :placeholder="t('userSettings.email')"
              disabled
              data-test="user-email-input"
            />
          </template>
        </Hook0CardContentLine>

        <Hook0CardContentLine>
          <template #label>{{ t('userSettings.firstName') }}</template>
          <template #content>
            <Hook0Input
              v-model="currentUser.firstName"
              type="text"
              :placeholder="t('userSettings.firstName')"
              disabled
            />
          </template>
        </Hook0CardContentLine>

        <Hook0CardContentLine>
          <template #label>{{ t('userSettings.lastName') }}</template>
          <template #content>
            <Hook0Input
              v-model="currentUser.lastName"
              type="text"
              :placeholder="t('userSettings.lastName')"
              disabled
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Appearance Preferences -->
    <Hook0Card data-test="appearance-card">
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0IconBadge variant="primary">
              <Palette :size="18" aria-hidden="true" />
            </Hook0IconBadge>
            <span>{{ t('userSettings.appearance') }}</span>
          </Hook0Stack>
        </template>
        <template #subtitle>{{ t('userSettings.appearanceSubtitle') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine>
          <template #label>{{ t('userSettings.theme') }}</template>
          <template #content>
            <Hook0Select
              v-model="colorModeValue"
              :options="colorModeOptions"
              data-test="theme-select"
            />
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine>
          <template #label>{{ t('userSettings.language') }}</template>
          <template #content>
            <Hook0Select
              model-value="en"
              :options="[{ label: t('userSettings.languageEnglish'), value: 'en' }]"
              disabled
              data-test="language-select"
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Change Password -->
    <Hook0Card v-if="currentUser" data-test="change-password-card">
      <Hook0Form
        data-test="change-password-form"
        :loading="isSubmitting"
        @submit="onChangePassword"
      >
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="warning">
                <Lock :size="18" aria-hidden="true" />
              </Hook0IconBadge>
              <span>{{ t('userSettings.changePassword') }}</span>
            </Hook0Stack>
          </template>
          <template #subtitle>
            {{ t('userSettings.changePasswordWarning') }}
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.currentPassword') }}</template>
            <template #content>
              <Hook0Input
                id="current-password-input"
                v-model="currentPassword"
                v-bind="currentPasswordAttrs"
                type="password"
                required
                show-password-toggle
                autocomplete="current-password"
                :aria-label="t('userSettings.currentPassword')"
                :placeholder="t('userSettings.currentPasswordPlaceholder')"
                :error="errors.current_password"
                :disabled="isSubmitting"
                data-test="current-password-input"
              />
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.newPassword') }}</template>
            <template #content>
              <Hook0Input
                v-model="newPassword"
                v-bind="newPasswordAttrs"
                type="password"
                required
                show-password-toggle
                autocomplete="new-password"
                :aria-label="t('userSettings.newPassword')"
                :placeholder="t('userSettings.newPasswordPlaceholder')"
                :error="errors.new_password"
                :disabled="isSubmitting"
                data-test="new-password-input"
              >
                <template #helpText>{{
                  t('validation.passwordRequirements', { count: passwordMinimumLength })
                }}</template>
              </Hook0Input>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.confirmPassword') }}</template>
            <template #content>
              <Hook0Input
                v-model="confirmNewPassword"
                v-bind="confirmNewPasswordAttrs"
                type="password"
                required
                :aria-label="t('userSettings.confirmPassword')"
                :placeholder="t('userSettings.confirmPasswordPlaceholder')"
                :error="errors.confirm_new_password"
                :disabled="isSubmitting"
                data-test="confirm-password-input"
              />
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button
            variant="primary"
            submit
            :disabled="isSubmitBlocked"
            :loading="isSubmitting"
            :tooltip="submitHint"
            :aria-label="t('userSettings.changePassword')"
            data-test="change-password-button"
          >
            <Lock :size="16" aria-hidden="true" />
            {{ t('userSettings.changePassword') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Form>
    </Hook0Card>

    <!-- Delete Account (Danger Zone) -->
    <Hook0Card v-if="currentUser" data-test="delete-account-card">
      <Hook0Form data-test="delete-account-form" @submit="deleteAccount">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="danger">
                <AlertTriangle :size="18" aria-hidden="true" />
              </Hook0IconBadge>
              <span>{{ t('userSettings.deleteAccount') }}</span>
            </Hook0Stack>
          </template>
          <template #subtitle>{{ t('userSettings.deleteAccountSubtitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0Alert type="alert">
            <template #description>
              {{
                t('userSettings.deleteAccountWarningDetail') ||
                t('userSettings.deleteAccountSubtitle')
              }}
            </template>
          </Hook0Alert>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button variant="danger" submit data-test="delete-account-button">
            <Trash2 :size="16" aria-hidden="true" />
            {{ t('userSettings.deleteAccount') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Form>
    </Hook0Card>

    <!-- Not logged in -->
    <Hook0Card v-if="!currentUser">
      <Hook0CardHeader>
        <template #header>{{ t('userSettings.notLoggedIn') }}</template>
        <template #subtitle>{{ t('userSettings.notLoggedInSubtitle') }}</template>
      </Hook0CardHeader>
    </Hook0Card>

    <Hook0Dialog
      :open="showDeleteAccountDialog"
      variant="danger"
      :title="t('userSettings.deleteAccount')"
      @close="showDeleteAccountDialog = false"
      @confirm="confirmDeleteAccount()"
    >
      <p>{{ t('userSettings.deleteAccountConfirm') }}</p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>
