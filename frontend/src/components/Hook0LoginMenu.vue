<script setup lang="ts">
import { inject, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import Keycloak from 'keycloak-js';

import { KeycloakTokenParsedAttributes, getToken, keycloakKey } from '@/iam';
import { routes } from '@/routes';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Dropdown from '@/components/Hook0Dropdown.vue';
import Hook0DropdownMenuItems from '@/components/Hook0DropdownMenuItems.vue';
import Hook0DropdownMenuItemText from '@/components/Hook0DropdownMenuItemText.vue';
import Hook0DropdownMenuItemLink from '@/components/Hook0DropdownMenuItemLink.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';

const $keycloak = inject(keycloakKey) as Keycloak;

const router = useRouter();
defineSlots<{
  default(): unknown;
}>();

const currentUser = ref<(Keycloak.KeycloakTokenParsed & KeycloakTokenParsedAttributes) | null>(
  null
);

async function logout() {
  await $keycloak.logout();
  await router.push('/');
}

onMounted(() => {
  return getToken().then(() => {
    currentUser.value = $keycloak.idTokenParsed as KeycloakTokenParsedAttributes;
  });
});
</script>

<template>
  <div>
    <Hook0Dropdown v-if="currentUser" orientation="left">
      <template #menu="parent">
        <Hook0Button
          class="bg-white rounded-full flex text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          @click="parent.toggle"
        >
          <Hook0Icon class="h-8 w-8 rounded-full fa-2x" name="user-circle"></Hook0Icon>
        </Hook0Button>
      </template>

      <template #dropdown="parent">
        <Hook0DropdownMenuItems>
          <Hook0DropdownMenuItemText>
            <Hook0Text>Hi 👋</Hook0Text>
            <Hook0Text class="block bold text-gray-900 truncate">
              {{ currentUser.email }}</Hook0Text
            >
          </Hook0DropdownMenuItemText>
        </Hook0DropdownMenuItems>
        <Hook0DropdownMenuItems>
          <Hook0DropdownMenuItemLink :to="{ name: routes.Settings }" @click="parent.close()">
            <Hook0Text>Settings</Hook0Text>
          </Hook0DropdownMenuItemLink>
        </Hook0DropdownMenuItems>
        <Hook0DropdownMenuItems>
          <Hook0DropdownMenuItemLink
            data-e2e="logout"
            @click="
              logout();
              parent.close();
            "
          >
            Logout
          </Hook0DropdownMenuItemLink>
        </Hook0DropdownMenuItems>
      </template>
    </Hook0Dropdown>
    <Hook0Loader v-else></Hook0Loader>
  </div>
</template>
