<script setup lang="ts">
import { ColDef, ValueFormatterParams } from '@ag-grid-community/core';
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import { getUserInfo } from '@/iam';
import { Invitation, Members, User } from './MemberService';
import * as MemberService from './MemberService';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import { Problem, UUID } from '@/http';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import { Hook0SelectSingleOption } from '@/components/Hook0Select';
import Hook0Select from '@/components/Hook0Select.vue';
import { Alert } from '@/components/Hook0Alert';
import Hook0Alert from '@/components/Hook0Alert.vue';

const route = useRoute();

interface Props {
  // cache-burst
  burst?: string | string[];
}

defineProps<Props>();
const columnDefs: ColDef[] = [
  {
    field: 'name',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Name',
    valueFormatter: (params: ValueFormatterParams<User, string[]>) => {
      const isCurrentUser = currentUser.value && params.data?.email == currentUser.value.email;
      const you = isCurrentUser ? ' (current user)' : '';
      return `${params.data?.first_name} ${params.data?.last_name}${you}`;
    },
  },
  {
    field: 'email',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    headerName: 'Email',
  },
  {
    field: 'role',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    width: 60,
    headerName: 'Role',
  },
  {
    width: 105,
    suppressMovable: true,
    headerName: 'Options',
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Delete',
      icon: 'trash',
      disabled(row: User) {
        return currentUser.value && row.email == currentUser.value.email;
      },
      onClick(row: User) {
        if (
          organization_id.value &&
          confirm(`Are you sure to revoke access of ${row.email} from this organization?`)
        ) {
          MemberService.revoke(organization_id.value, row.user_id)
            .then(() => {
              // @TODO notify user of success
              _forceLoad();
            })
            .catch(displayError);
        }
      },
    },
  },
];

const currentUser = getUserInfo();
const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});
const members$ = ref<Promise<Members>>();
const organization_id = ref<null | UUID>(null);

const invitation$ = ref<Invitation>(emptyInvitation());

function emptyInvitation(): Invitation {
  return {
    email: '',
    role: '',
  };
}

const roles: Hook0SelectSingleOption[] = [
  { label: '', value: '' },
  { label: 'Editor', value: 'editor' },
  { label: 'Viewer', value: 'viewer' },
];

function invite(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (organization_id.value && invitation$.value.email !== '' && invitation$.value.role != '') {
    MemberService.invite(organization_id.value, invitation$.value)
      .then(() => {
        // @TODO notify user of success
        invitation$.value = emptyInvitation();
        _forceLoad();
      })
      .catch(displayError);
  }
}

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  members$.value = MemberService.get(route.params.organization_id as string);
}

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    _forceLoad();
  }
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <Promised :promise="members$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="members">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>Members</template>
          <template #subtitle>Your organization can be used by multiple users.</template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="members.members.length > 0">
          <transition name="ease">
            <Hook0Table
              :context="{ members$, columnDefs }"
              :column-defs="columnDefs"
              :row-data="members.members"
            >
            </Hook0Table>
          </transition>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text
                  >Start your journey by creating a Hook0 application. This application will have
                  API keys that will be required to send events to Hook0 API so it can dispatch
                  these events to your customers through webhooks.</Hook0Text
                >
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardContent v-if="alert.visible">
          <Hook0Alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></Hook0Alert>
        </Hook0CardContent>
        <form @submit="invite">
          <Hook0CardFooter>
            <Hook0Input
              v-model="invitation$.email"
              type="email"
              placeholder="Email address"
              required
              class="flex-grow-1"
            />
            <Hook0Select
              v-model="invitation$.role"
              class="flex-none width-small"
              :options="roles"
            />
            <Hook0Button
              class="primary"
              type="submit"
              :disabled="invitation$.email === '' || invitation$.role === ''"
              @click="invite($event)"
              >Invite a user
            </Hook0Button>
          </Hook0CardFooter>
        </form>
      </Hook0Card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
</template>
