<script setup lang="ts">
import * as VerifyEmailService from './VerifyEmailService';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import { Problem } from '@/http.ts';
import { Alert } from '@/components/Hook0Alert.ts';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import Hook0Alert from '@/components/Hook0Alert.vue';

const route = useRoute();

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function displaySuccess(sec: number) {
  alert.value.visible = true;
  alert.value.type = 'success';
  alert.value.title = 'Success';
  alert.value.description =
    'Email verified successfully, redirect to login page in ' + sec + ' seconds';
}

function _onLoad() {
  const token = route.query.token as string;
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: 'Invalid token',
      detail: 'Token is required to verify email',
    });
    return;
  }
  VerifyEmailService.verifyEmail(token)
    .then(() => {
      setInterval(() => {
        displaySuccess(5);
        setTimeout(() => {
          void router.push(routes.Login);
        }, 5000);
      }, 1000);
    })
    .catch(displayError);
}

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Verify email</template>
      <template #subtitle>If request failed, retry in a few minutes or contact support</template>
    </Hook0CardHeader>
    <Hook0CardContent v-if="alert.visible">
      <Hook0Alert
        :type="alert.type"
        :title="alert.title"
        :description="alert.description"
      ></Hook0Alert>
    </Hook0CardContent>
  </Hook0Card>
</template>
