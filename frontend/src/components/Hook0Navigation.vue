<template>
  <nav class="bg-white shadow">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex justify-between h-16">
        <div class="flex">
          <div class="flex-shrink-0 flex items-center">
            <router-link class="navbar-item" :to="{ path: '/' }">
              <img src="../assets/logo.png" width="28" height="28" />
            </router-link>
          </div>
          <div class="navigation-items">
            <a href="/" class="navigation-item navigation-item--active">
              {{ $t('menu.deploymentList') }}
            </a>
            <a class="navigation-item">
              {{ $t('menu.teamList') }}
            </a>
          </div>
        </div>
        <div class="hidden sm:ml-6 sm:flex sm:items-center">
          <slot name="right"></slot>
        </div>
      </div>
    </div>
  </nav>
</template>

<script lang="ts">
export default {
  name: 'hook0-navigation',
  data() {
    return {};
  },
};
</script>

<style lang="scss" scoped>
.footer-container {
  @apply max-w-7xl mx-auto py-12 px-4 overflow-hidden sm:px-6 lg:px-8;
}
.navigation-items {
  @apply hidden sm:ml-6 sm:flex sm:space-x-8;
}
.navigation-item {
  @apply border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700 inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium;
}

.navigation-item--active {
  @apply border-indigo-500 text-gray-900 inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium;
}
</style>
