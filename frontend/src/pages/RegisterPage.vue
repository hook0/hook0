<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';

import { register, getAccessToken } from '@/iam';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
//import { handleError, Problem } from '@/http.ts';
//import { AxiosError, AxiosResponse } from 'axios';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';

const router = useRouter();

const email = ref<string>('');
const firstName = ref<string>('');
const lastName = ref<string>('');
const password = ref<string>('');

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

let intervalId: number | null = null;

async function submit() {
  await register(email.value, firstName.value, lastName.value, password.value)
    .then(() => {
      let seconds = 3;
      intervalId = window.setInterval(() => {
        if (seconds >= 0) {
          displaySucess(seconds);
          seconds--;
        } else {
          if (intervalId !== null) {
            window.clearInterval(intervalId);
            intervalId = null;
            //ça ne veut pas fonctionner avec return mais void fonctionne
            void router.push('/login');
          }
        }
      }, 1000);
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

function displaySucess(seconds: number = 3) {
  alert.value.visible = true;
  alert.value.type = 'success';
  alert.value.title = 'Success';
  alert.value.description = 'You will be redirected in ' + seconds + ' seconds';
}

function _onLoad() {
  alert.value.visible = false;
  alert.value.type = 'alert';
  alert.value.title = '';
  alert.value.description = '';
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
          <template #header> Register </template>
          <template #subtitle>
            <div class="text-sm text-gray-500">
              Welcome to Hook0. Please enter your information to register and start using our
              services.
            </div>
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
            <template #label> First Name </template>
            <template #content>
              <div class="flex flex-row">
                <Hook0Input
                  v-model="firstName"
                  type="text"
                  class="w-full"
                  placeholder="First Name"
                  required
                >
                </Hook0Input>
              </div>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> Last Name </template>
            <template #content>
              <div class="flex flex-row">
                <Hook0Input
                  v-model="lastName"
                  type="text"
                  class="w-full"
                  placeholder="Last Name"
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
          <Hook0Button class="primary" type="button" @click="submit">Register</Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>
  </div>
</template>