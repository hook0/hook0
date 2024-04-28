<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import { inject, onMounted, onUpdated, ref } from 'vue';
import { getToken, keycloakKey, KeycloakTokenParsedAttributes } from '@/iam.ts';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Keycloak from 'keycloak-js';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import * as UsersServices from '@/pages/users/UsersService.ts';
import { Problem } from '@/http.ts';

const $keycloak = inject(keycloakKey) as Keycloak;

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function _load() {
  return getToken().then(() => {
    currentUser.value = $keycloak.idTokenParsed as KeycloakTokenParsedAttributes;
  });
}

onMounted(() => {
  void _load();
});

onUpdated(() => {
  void _load();
});

const currentUser = ref<(Keycloak.KeycloakTokenParsed & KeycloakTokenParsedAttributes) | null>(
  null
);

function deleteAccount(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!confirm(`Are you sure to delete your account?`)) {
    return;
  }

  alert.value.visible = false; // reset alert

  UsersServices.deleteUser()
    .then(() => {
      alert.value.visible = true;
      alert.value.type = 'success';
      alert.value.title = 'Account deleted';
      alert.value.description =
        'Your account has been deleted. You will be redirected to the login page.';
      setTimeout(() => {
        void $keycloak.logout();
      }, 3000);
    })
    .catch(displayError);
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}
</script>

<template>
  <div>
    <form>
      <Hook0Card v-if="currentUser">
        <Hook0CardHeader>
          <template #header> Personal information </template>
          <template #subtitle>
            This is your personal information. Contact support to change it.
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label> Email </template>
            <template #content>
              <Hook0Input
                v-model="currentUser.email"
                type="text"
                placeholder="Email"
                disabled
                class="w-full disabled:bg-slate-50 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none"
              >
              </Hook0Input>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> First Name </template>
            <template #content>
              <Hook0Input
                v-model="currentUser.given_name"
                type="text"
                placeholder="First Name"
                disabled
                class="w-full disabled:bg-slate-50 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none"
              >
              </Hook0Input>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> Last Name </template>
            <template #content>
              <Hook0Input
                v-model="currentUser.family_name"
                type="text"
                placeholder="Last Name"
                disabled
                class="w-full disabled:bg-slate-50 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none"
              >
              </Hook0Input>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
      </Hook0Card>

      <Hook0Card v-if="currentUser">
        <Hook0CardHeader>
          <template #header> Delete my account </template>
          <template #subtitle>
            This action <strong>delete your account</strong> and all your data linked to it.
            <strong>This action irreversible.</strong>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent v-if="alert.visible">
          <Hook0Alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></Hook0Alert>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button class="danger" type="button" @click="deleteAccount($event)"
            >Delete</Hook0Button
          >
        </Hook0CardFooter>
      </Hook0Card>

      <!-- If the user is not logged in, show a message -->
      <Hook0Card v-else>
        <Hook0CardHeader>
          <template #header>Not logged in</template>
          <template #subtitle
            >You are not logged in. Please log in to view your settings.
          </template>
        </Hook0CardHeader>
      </Hook0Card>
    </form>
  </div>
</template>
