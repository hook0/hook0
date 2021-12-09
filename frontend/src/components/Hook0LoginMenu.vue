<template>
  <div class="ml-3 relative" v-if="currentUser">
    <div>
      <button
        class="bg-white rounded-full flex text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        id="user-menu"
        aria-haspopup="true"
        @click="open = !open"
      >
        <span class="sr-only">Open user menu</span>
        <b-icon class="h-8 w-8 rounded-full" size="fa-2x" pack="fas" icon="user-circle" />
      </button>
    </div>
    <!--
  Profile dropdown panel, show/hide based on dropdown state.

  Entering: "transition ease-out duration-200"
    From: "transform opacity-0 scale-95"
    To: "transform opacity-100 scale-100"
  Leaving: "transition ease-in duration-75"
    From: "transform opacity-100 scale-100"
    To: "transform opacity-0 scale-95"
-->
    <div
      class="origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white ring-1 ring-black ring-opacity-5 z-10"
      role="menu"
      aria-orientation="vertical"
      aria-labelledby="user-menu"
      v-show="open"
    >
      <span class="menu-item-text " role="menuitem">{{ currentUser.email }}</span>
      <span class="menu-item-text" role="menuitem">{{ kcFullName }}</span>
      <a @click="logout()" class="menu-item-link" role="menuitem">{{ $t('logout') }}</a>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import router from '@/router';
import { KeycloakProfile } from 'keycloak-js';

interface AccessToken {
  given_name?: string,
  preferred_username: string,
}

export default defineComponent({
  name: 'hook0-login-menu',
  async mounted() {
    this.currentUser = await this.$keycloak.loadUserProfile();
  },
  data() {
    return {
      currentUser: null as KeycloakProfile | null,
      open: false,
    };
  },
  computed: {
    kcGivenName(): string {
      if (this.isKeycloakFeatureEnabled && this.$keycloak.tokenParsed) {
        const tokenParsed = this.$keycloak.tokenParsed as AccessToken;
        return tokenParsed.given_name ? tokenParsed.given_name : tokenParsed.preferred_username;
      }
      return '';
    },
    kcUserName(): string {
      if (this.isKeycloakFeatureEnabled && this.$keycloak.tokenParsed) {
        const tokenParsed = this.$keycloak.tokenParsed as AccessToken;
        return tokenParsed.preferred_username;
      }
      return '';
    },
    kcFullName(): string | undefined {
      const firstName = this.currentUser?.firstName;
      const lastName = this.currentUser?.lastName;
      if (!firstName || !lastName) return;

      return `${firstName} ${lastName}`;
    },
    isKeycloakFeatureEnabled(): boolean {
      return process.env.VUE_APP_FEATURES_KEYCLOAK  === 'true';
    },
  },
  methods: {
    isAuthenticated(): boolean {
      return !!this.currentUser;
    },
    async logout() {
      await this.$keycloak.logout();
      await router.push('/');
    },
  },
});
</script>

<style scoped lang="scss">
.given-name {
  padding-right: 5px;
}

.menu-item-text {
  @apply block px-4 py-2 text-sm text-gray-500;
}

.menu-item-link {
  @apply block px-4 py-2 text-sm font-medium text-indigo-600 hover:text-indigo-900 hover:bg-gray-100;
}
</style>
