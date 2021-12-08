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
import { Vue, Options } from 'vue-class-component';
import router from '@/router';

@Options({
  name: 'hook0-login-menu',
  data() {
    return {
      currentUser: null,
      open: false,
    };
  },
  async mounted() {
    this.currentUser = await this.$keycloak.loadUserProfile();
  },
  methods: {
    isAuthenticated() {
      return !!this.currentUser;
    },
    logout() {
      this.$keycloak.logout();
      router.push('/');
    },
  },
  computed: {
    kcGivenName() {
      if (this.isKeycloakFeatureEnabled && this.$keycloak.tokenParsed) {
        const tokenParsed = this.$keycloak.tokenParsed;
        return tokenParsed.given_name ? tokenParsed.given_name : tokenParsed.preferred_username;
      }
      return '';
    },
    kcUserName() {
      if (this.isKeycloakFeatureEnabled && this.$keycloak.tokenParsed) {
        const tokenParsed = this.$keycloak.tokenParsed;
        return tokenParsed.preferred_username;
      }
      return '';
    },
    kcFullName() {
      const { firstName, lastName } = this.currentUser;
      if (!firstName && !lastName) return;

      return `${firstName} ${lastName}`;
    },
    isKeycloakFeatureEnabled() {
      return process.env.VUE_APP_FEATURES_KEYCLOAK === 'true';
    },
  },
})
export default class Hook0LoginMenu extends Vue {};
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
