<template>
  <div>
    <hook0-dropdown v-if="currentUser" orientation="left">
      <template v-slot:menu="parent">
        <hook0-button
          class="bg-white rounded-full flex text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          @click="parent.toggle"
        >
          <hook0-icon class="h-8 w-8 rounded-full fa-2x" name="user-circle"></hook0-icon>
        </hook0-button>
      </template>

      <template v-slot:dropdown="parent">
        <hook0-dropdown-menu-items>
          <hook0-dropdown-menu-item-text>
            <hook0-text>Hi 👋</hook0-text>
            <hook0-text class="block bold text-gray-900 truncate">
              {{ currentUser.email }}</hook0-text
            >
          </hook0-dropdown-menu-item-text>
        </hook0-dropdown-menu-items>
        <hook0-dropdown-menu-items>
          <hook0-dropdown-menu-item-link :to="{ name: routes.Settings }" @click="parent.close()">
            <hook0-text>Settings</hook0-text>
          </hook0-dropdown-menu-item-link>
        </hook0-dropdown-menu-items>
        <hook0-dropdown-menu-items>
          <hook0-dropdown-menu-item-link
            data-e2e="logout"
            @click="
              logout();
              parent.close();
            "
          >
            Logout
          </hook0-dropdown-menu-item-link>
        </hook0-dropdown-menu-items>
      </template>
    </hook0-dropdown>
    <hook0-loader v-else></hook0-loader>
  </div>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import iam, { KeycloakTokenParsedAttributes } from '@/iam';
import { routes } from '@/routes';

export default class Hook0LoginMenu extends Vue {
  currentUser: (Keycloak.KeycloakTokenParsed & KeycloakTokenParsedAttributes) | null = null;
  open = false;

  routes = routes;

  mounted() {
    return iam.getToken().then(() => {
      this.currentUser = this.$keycloak.idTokenParsed as KeycloakTokenParsedAttributes;
    });
  }

  async logout() {
    await this.$keycloak.logout();
    await this.$router.push('/');
  }
}
</script>

<style scoped lang="scss"></style>
