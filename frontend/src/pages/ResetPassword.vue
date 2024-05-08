<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import { onMounted, ref } from 'vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import { push } from 'notivue';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { resetPassword } from '@/pages/users/UsersServices.ts';
import { routes } from '@/routes.ts';
import router from '@/router.ts';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

const token = ref('');
const new_password = ref('');
const confirm_new_password = ref('');

async function submit() {
  if (new_password.value !== confirm_new_password.value) {
    push.warning({
      title: 'Warning',
      message: 'Passwords do not match.',
      duration: 5000,
    });
    return;
  }
  await resetPassword(token.value, new_password.value)
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Your password has been reset successfully. Please login.',
        duration: 5000,
      });
      return router.push(routes.Login);
    })
    .catch((err: AxiosError<AxiosResponse<Problem>>) => {
      let problem = handleError(err);
      displayError(problem);
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
  token.value = router.currentRoute.value.query.token as string;
  if (!token.value) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: 'Invalid token',
      detail: 'Token is required to reset password',
    });
    return;
  }
}

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0CardContent v-if="alert.visible">
    <Hook0Alert
      :type="alert.type"
      :title="alert.title"
      :description="alert.description"
    ></Hook0Alert>
  </Hook0CardContent>
  <form v-else @submit="submit">
    <Hook0Card>
      <Hook0CardHeader>
        <template #header> Change account password </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine>
          <template #label> New password </template>
          <template #content>
            <div class="flex flex-row">
              <Hook0Input
                v-model="new_password"
                type="password"
                class="w-full"
                placeholder="New password"
                required
              >
              </Hook0Input>
            </div>
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine>
          <template #label> Confirm new password </template>
          <template #content>
            <div class="flex flex-row">
              <Hook0Input
                v-model="confirm_new_password"
                type="password"
                class="w-full"
                placeholder="Confirm new password"
                required
              >
              </Hook0Input>
            </div>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button class="primary" type="submit" @click="submit">Submit</Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </form>
</template>
