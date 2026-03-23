<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { toast } from 'vue-sonner';
import { Problem } from '@/http';
import { resetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes';
import router from '@/router';
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

  if (new_password.value !== confirm_new_password.value) {
    toast.warning(t('common.warning'), {
      description: t('auth.resetPassword.passwordsMismatch'),
      duration: 5000,
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
    })
    .finally(() => {
      isLoading.value = false;
    });
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
  if (!token) {
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
      <Hook0Logo variant="image" size="lg" />
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

    <!-- Form Card -->
    <Hook0Card v-else variant="glow">
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
            data-test="reset-password-new-password-input"
          />

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
