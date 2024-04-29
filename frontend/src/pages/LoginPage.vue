<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';

import { login, getAccessToken } from '@/iam';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { handleError, Problem } from '@/http.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';

const router = useRouter();

const email = ref<string>('');
const password = ref<string>('');

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

async function submit() {
  await login(email.value, password.value)
    .then(() => {
      return router.push(routes.Home);
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
  let token = getAccessToken();
  if (token.value !== null) {
    return router.push(routes.Home);
  }
}

onMounted(_onLoad);
onUpdated(_onLoad);
</script>

<template>
  <div>
    <form @submit="submit">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Login </template>
          <template #subtitle>
            <div class="text-sm text-gray-500">Please enter your email and password to login.</div>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label> Email </template>
            <template #content>
              <div class="flex flex-row">
                <Hook0Input
                  v-model="email"
                  type="email"
                  class="w-full"
                  placeholder="Email"
                  required
                >
                </Hook0Input>
              </div>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> Password </template>
            <template #content>
              <div class="flex flex-row">
                <Hook0Input
                  v-model="password"
                  type="password"
                  class="w-full"
                  placeholder="Password"
                  required
                >
                </Hook0Input>
              </div>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
        <Hook0CardContent v-if="alert.visible">
          <Hook0Alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></Hook0Alert>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button class="primary" type="button" @click="submit">Login</Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>
  </div>
</template>