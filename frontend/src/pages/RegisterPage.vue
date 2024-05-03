<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';

import { register, getAccessToken } from '@/iam';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { push } from 'notivue';

const router = useRouter();

const email = ref<string>('');
const firstName = ref<string>('');
const lastName = ref<string>('');
const password = ref<string>('');

async function submit() {
  await register(email.value, firstName.value, lastName.value, password.value)
    .then(() => {
      push.success({
        title: 'Success',
        message:
          "You're successfully registered. You need to confirm your email address before using Hook0. Check your mailbox!",
        duration: 5000,
      });
    })
    .catch((err: AxiosError<AxiosResponse<Problem>>) => {
      let problem = handleError(err);
      let options = {
        title: problem.title,
        message: problem.detail,
        duration: 5000,
      };
      problem.status >= 500 ? push.error(options) : push.warning(options);
    });
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
        <Hook0CardFooter>
          <Hook0Button class="primary" type="button" @click="submit">Register</Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>
  </div>
</template>