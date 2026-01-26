<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import * as UserService from '@/pages/user/UserService';
import { Problem } from '@/http.ts';
import { push } from 'notivue';
import { getUserInfo, logout } from '@/iam.ts';
import { ref } from 'vue';

const currentUser = getUserInfo();

const new_password = ref<string>('');
const confirm_new_password = ref<string>('');

async function changePassword() {
  if (new_password.value !== confirm_new_password.value) {
    push.error({
      title: 'Error',
      message: 'Passwords do not match.',
      duration: 5000,
    });
    return;
  }

  await UserService.changePassword(new_password.value).catch(displayError);

  push.success({
    title: 'Success',
    message: 'Your password was successfully changed.',
    duration: 3000,
  });
}

async function deleteAccount() {
  if (!confirm(`Are you sure to delete your account?`)) {
    return;
  }

  await UserService.deleteUser()
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Your account has been deleted. You will be logged out in 3 seconds.',
        duration: 3000,
      });
      setTimeout(() => {
        void logout();
      }, 3000);
    })
    .catch(displayError);
}

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}
</script>

<template>
  <div>
    <Hook0Card v-if="currentUser" data-test="user-info-card">
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
              data-test="user-email-input"
            >
            </Hook0Input>
          </template>
        </Hook0CardContentLine>

        <Hook0CardContentLine>
          <template #label> First Name </template>
          <template #content>
            <Hook0Input
              v-model="currentUser.firstName"
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
              v-model="currentUser.lastName"
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

    <Hook0Card v-if="currentUser" data-test="change-password-card">
      <form data-test="change-password-form" @submit.prevent="changePassword">
        <Hook0CardHeader>
          <template #header>Change password</template>
          <template #subtitle>Set a new password to your user account.</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label> New password </template>
            <template #content>
              <Hook0Input
                v-model="new_password"
                type="password"
                placeholder="New password"
                required
                class="w-full"
                data-test="new-password-input"
              >
              </Hook0Input>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> Confirm new password </template>
            <template #content>
              <Hook0Input
                v-model="confirm_new_password"
                type="password"
                placeholder="Confirm new password"
                required
                class="w-full"
                data-test="confirm-password-input"
              >
              </Hook0Input>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
        <Hook0CardFooter class="!justify-between">
          <div
            class="flex items-center bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 p-4 rounded"
          >
            <div>
              <p class="font-bold">Warning</p>
              <p>You will be disconnected from all browsers/devices including the current one.</p>
            </div>
          </div>
          <Hook0Button
            class="primary"
            submit
            aria-label="Change Password"
            title="Change Password"
            data-test="change-password-button"
            >Change password</Hook0Button
          >
        </Hook0CardFooter>
      </form>
    </Hook0Card>

    <Hook0Card v-if="currentUser" data-test="delete-account-card">
      <form data-test="delete-account-form" @submit.prevent="deleteAccount">
        <Hook0CardHeader>
          <template #header> Delete my account </template>
          <template #subtitle>
            This action <strong>delete your account</strong> and all your data linked to it.
            <strong>This action irreversible.</strong>
          </template>
        </Hook0CardHeader>
        <Hook0CardFooter>
          <Hook0Button class="danger" submit data-test="delete-account-button">Delete</Hook0Button>
        </Hook0CardFooter>
      </form>
    </Hook0Card>

    <!-- If the user is not logged in, show a message -->
    <Hook0Card v-else>
      <Hook0CardHeader>
        <template #header>Not logged in</template>
        <template #subtitle>You are not logged in. Please log in to view your settings.</template>
      </Hook0CardHeader>
    </Hook0Card>
  </div>
</template>
