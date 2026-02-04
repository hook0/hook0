<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { push } from 'notivue';
import { User, Lock, AlertTriangle, Trash2 } from 'lucide-vue-next';

import * as UserService from '@/pages/user/UserService';
import { useAuthStore } from '@/stores/auth';
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
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();
const authStore = useAuthStore();
const currentUser = computed(() => authStore.userInfo);

const new_password = ref('');
const confirm_new_password = ref('');

function changePassword() {
  if (new_password.value !== confirm_new_password.value) {
    push.error({
      title: t('common.error'),
      message: t('userSettings.passwordMismatch'),
      duration: 5000,
    });
    return;
  }

  UserService.changePassword(new_password.value)
    .then(() => {
      push.success({
        title: t('common.success'),
        message: t('userSettings.passwordChanged'),
        duration: 3000,
      });
    })
    .catch((err) => displayError(err as Problem));
}

function deleteAccount() {
  if (!confirm(t('userSettings.deleteAccountConfirm'))) {
    return;
  }

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
            <Hook0Text>{{ t('userSettings.personalInfo') }}</Hook0Text>
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

    <!-- Change Password -->
    <Hook0Card v-if="currentUser" data-test="change-password-card">
      <Hook0Form data-test="change-password-form" @submit="changePassword">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <Hook0IconBadge variant="warning">
                <Lock :size="18" aria-hidden="true" />
              </Hook0IconBadge>
              <Hook0Text>{{ t('userSettings.changePassword') }}</Hook0Text>
            </Hook0Stack>
          </template>
          <template #subtitle>{{ t('userSettings.changePasswordSubtitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.newPassword') }}</template>
            <template #content>
              <Hook0Input
                v-model="new_password"
                type="password"
                :placeholder="t('userSettings.newPasswordPlaceholder')"
                required
                data-test="new-password-input"
              />
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label>{{ t('userSettings.confirmPassword') }}</template>
            <template #content>
              <Hook0Input
                v-model="confirm_new_password"
                type="password"
                :placeholder="t('userSettings.confirmPasswordPlaceholder')"
                required
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
              <Hook0Text>{{ t('userSettings.deleteAccount') }}</Hook0Text>
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
  </Hook0PageLayout>
</template>

<style scoped>
/* No custom CSS - using Hook0* components only */
</style>
