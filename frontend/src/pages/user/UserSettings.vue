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
import { getUserInfo } from '@/iam.ts';
import { ref, onMounted } from 'vue';

const currentUser = getUserInfo();

const new_password = ref<string>('');
const confirm_new_password = ref<string>('');
const deletionRequested = ref<boolean>(false);
const loadingDeletionStatus = ref<boolean>(true);

onMounted(() => {
  if (currentUser) {
    UserService.getAccountDeletionStatus()
      .then((status) => {
        deletionRequested.value = status.deletion_requested;
        loadingDeletionStatus.value = false;
      })
      .catch((err: Problem) => {
        displayError(err);
        loadingDeletionStatus.value = false;
      });
  } else {
    loadingDeletionStatus.value = false;
  }
});

function changePassword() {
  if (new_password.value !== confirm_new_password.value) {
    push.error({
      title: 'Error',
      message: 'Passwords do not match.',
      duration: 5000,
    });
    return;
  }

  UserService.changePassword(new_password.value)
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Your password was successfully changed.',
        duration: 3000,
      });
    })
    .catch(displayError);
}

function requestDeletion() {
  if (
    !confirm(
      'Are you sure you want to request account deletion?\n\nYour account will be permanently deleted after 30 days. You can cancel this request during this period.'
    )
  ) {
    return;
  }

  UserService.requestAccountDeletion()
    .then(() => {
      deletionRequested.value = true;
      push.success({
        title: 'Deletion requested',
        message:
          'Your account deletion has been requested. It will be permanently deleted in 30 days.',
        duration: 5000,
      });
    })
    .catch(displayError);
}

function cancelDeletion() {
  if (!confirm('Are you sure you want to cancel your account deletion request?')) {
    return;
  }

  UserService.cancelAccountDeletion()
    .then(() => {
      deletionRequested.value = false;
      push.success({
        title: 'Deletion cancelled',
        message: 'Your account deletion request has been cancelled.',
        duration: 3000,
      });
    })
    .catch(displayError);
}

function displayError(err: Problem) {
  console.error(err);
  const options = {
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

    <Hook0Card v-if="currentUser && !loadingDeletionStatus" data-test="delete-account-card">
      <!-- Deletion pending state -->
      <template v-if="deletionRequested">
        <form @submit.prevent="cancelDeletion">
          <Hook0CardHeader>
            <template #header> Account deletion pending </template>
            <template #subtitle>
              Your account is scheduled for deletion. It will be permanently deleted in 30 days. You
              can cancel this request during this period.
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <div
              class="flex items-center bg-orange-100 border-l-4 border-orange-500 text-orange-700 p-4 rounded"
            >
              <div>
                <p class="font-bold">Deletion scheduled</p>
                <p>
                  Your account and all associated data will be permanently deleted after the 30-day
                  grace period.
                </p>
              </div>
            </div>
          </Hook0CardContent>
          <Hook0CardFooter>
            <Hook0Button class="secondary" submit>Cancel deletion request</Hook0Button>
          </Hook0CardFooter>
        </form>
      </template>

      <!-- Normal state - request deletion -->
      <template v-else>
        <form data-test="delete-account-form" @submit.prevent="requestDeletion">
          <Hook0CardHeader>
            <template #header> Delete my account </template>
            <template #subtitle>
              Request the deletion of your account and all your data linked to it. You will have 30
              days to cancel this request.
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <div
              class="flex items-center bg-red-100 border-l-4 border-red-500 text-red-700 p-4 rounded"
            >
              <div>
                <p class="font-bold">Warning</p>
                <p>
                  After 30 days, this action is <strong>irreversible</strong>. All your data will be
                  permanently deleted.
                </p>
              </div>
            </div>
          </Hook0CardContent>
          <Hook0CardFooter>
            <Hook0Button data-test="delete-account-button" class="danger" submit>Request account deletion</Hook0Button>
          </Hook0CardFooter>
        </form>
      </template>
    </Hook0Card>

    <!-- If the user is not logged in, show a message -->
    <Hook0Card v-if="!currentUser">
      <Hook0CardHeader>
        <template #header>Not logged in</template>
        <template #subtitle>You are not logged in. Please log in to view your settings.</template>
      </Hook0CardHeader>
    </Hook0Card>
  </div>
</template>
