<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { push } from 'notivue';
import { User, Lock, AlertTriangle, Trash2, Palette } from 'lucide-vue-next';

import * as UserService from '@/pages/user/UserService';
import { passwordChangeSchema } from '@/pages/user/passwordChange.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { useAuthStore } from '@/stores/auth';
import { useUiStore } from '@/stores/ui';
import type { ColorMode } from '@/stores/ui';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';

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

// VeeValidate form with Zod schema for password change
const { errors, defineField, handleSubmit, resetForm } = useForm({
  validationSchema: toTypedSchema(passwordChangeSchema),
});

const [newPassword, newPasswordAttrs] = defineField('new_password');
const [confirmNewPassword, confirmNewPasswordAttrs] = defineField('confirm_new_password');

const onChangePassword = handleSubmit((values) => {
  UserService.changePassword(values.new_password)
    .then(() => {
      push.success({
        title: t('common.success'),
        message: t('userSettings.passwordChanged'),
        duration: 3000,
      });
      resetForm();
    })
    .catch((err) => displayError(err as Problem));
});

const showDeleteAccountDialog = ref(false);

function deleteAccount() {
  showDeleteAccountDialog.value = true;
}

function confirmDeleteAccount() {
  showDeleteAccountDialog.value = false;

  UserService.deleteUser()
    .then(() => {
      push.success({
        title: t('common.success'),
        message: t('userSettings.accountDeleted'),
        duration: 3000,
      });
      setTimeout(() => {
        void authStore.logout();
      }, 3000);
    })
    .catch((err) => displayError(err as Problem));
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
              :options="[{ label: 'English', value: 'en' }]"
              disabled
              data-test="language-select"
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Change Password -->
    <Hook0Card v-if="currentUser" data-test="change-password-card">
      <Hook0Form data-test="change-password-form" @submit="onChangePassword">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="warning">
                <Lock :size="18" aria-hidden="true" />
              </Hook0IconBadge>
              <span>{{ t('userSettings.changePassword') }}</span>
            </Hook0Stack>
          </template>
          <template #subtitle>{{ t('userSettings.changePasswordSubtitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.newPassword') }}</template>
            <template #content>
              <Hook0Input
                v-model="newPassword"
                v-bind="newPasswordAttrs"
                type="password"
                :placeholder="t('userSettings.newPasswordPlaceholder')"
                :error="errors.new_password"
                data-test="new-password-input"
              />
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.confirmPassword') }}</template>
            <template #content>
              <Hook0Input
                v-model="confirmNewPassword"
                v-bind="confirmNewPasswordAttrs"
                type="password"
                :placeholder="t('userSettings.confirmPasswordPlaceholder')"
                :error="errors.confirm_new_password"
                data-test="confirm-password-input"
              />
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Stack direction="row" align="center" justify="between" gap="md">
            <Hook0Alert type="warning" :title="t('common.warning')">
              <template #description>{{ t('userSettings.changePasswordWarning') }}</template>
            </Hook0Alert>
            <Hook0Button
              variant="primary"
              submit
              :aria-label="t('userSettings.changePassword')"
              :title="t('userSettings.changePassword')"
              data-test="change-password-button"
            >
              <Lock :size="16" aria-hidden="true" />
              {{ t('userSettings.changePassword') }}
            </Hook0Button>
          </Hook0Stack>
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
            {{ t('common.delete') }}
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

<style scoped>
/* No custom CSS - using Hook0* components only */
</style>
