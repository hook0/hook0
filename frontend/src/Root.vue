<script setup lang="ts">
import { RouteLocationNamedRaw, useRoute, RouterView } from 'vue-router';
import { computed } from 'vue';

import Hook0Logo from '@/components/Hook0Logo.vue';
import MenuItem from '@/components/MenuItem.vue';
import OrganizationSelector from '@/pages/OrganizationAndApplicationSelector.vue';
import { routes } from '@/routes';
import Hook0Footer from '@/components/Hook0Footer.vue';
import Hook0LoginMenu from '@/components/Hook0LoginMenu.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import { Notivue, Notification, NotificationProgress } from 'notivue';

const route = useRoute();

interface Route {
  name: string;
  icon: string;
  route?: RouteLocationNamedRaw;
  href?: string;
}

const items = computed<Route[]>(() => {
  if (route.params.organization_id && route.params.application_id) {
    return [
      {
        name: 'API Keys',
        icon: 'key',
        route: {
          name: routes.ApplicationSecretsList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      },
      {
        name: 'Event Types',
        icon: 'folder-tree',
        route: {
          name: routes.EventTypesList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      },
      {
        name: 'Events',
        icon: 'file-lines',
        route: {
          name: routes.EventsList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      },
      {
        name: 'Subscriptions',
        icon: 'link',
        route: {
          name: routes.SubscriptionsList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      },
      {
        name: 'Request Attempts',
        icon: 'file-contract',
        route: {
          name: routes.LogsList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      },
      {
        name: 'Settings',
        icon: 'sliders',
        route: {
          name: routes.ApplicationsDashboard,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      },
      {
        name: 'API Documentation',
        icon: 'gear',
        href: 'https://documentation.hook0.com/',
      },
    ];
  } else {
    return [
      {
        name: 'API Documentation',
        icon: 'book',
        href: 'https://documentation.hook0.com/',
      },
    ];
  }
});
</script>

<template>
  <Notivue v-slot="item">
    <Notification :item="item">
      <NotificationProgress :item="item" />
    </Notification>
  </Notivue>
  <div class="h-screen flex overflow-hidden bg-gray-100">
    <div class="hidden md:flex md:flex-shrink-0">
      <div class="flex flex-col w-64 bg-gray-800">
        <div class="flex flex-col h-0 flex-1">
          <div class="flex items-center h-16 flex-shrink-0 px-4">
            <Hook0Logo></Hook0Logo>
          </div>
          <div class="flex flex-shrink-0 bg-gray-100">
            <OrganizationSelector></OrganizationSelector>
          </div>
          <div class="flex-1 flex flex-col overflow-y-auto">
            <nav class="flex-1 px-2 py-4 space-y-1">
              <MenuItem
                v-for="(item, index) in items"
                :key="index"
                :active="item.route ? item.route.name === $route.name : false"
                :name="item.name"
                :href="item.href"
                :to="item.route"
              >
                <Hook0Icon class="mr-1" :name="item.icon"></Hook0Icon>
              </MenuItem>
            </nav>
          </div>
        </div>
      </div>
    </div>
    <div class="flex flex-col w-0 flex-1 overflow-hidden">
      <div class="relative z-10 flex-shrink-0 flex h-16 bg-white shadow">
        <button
          class="px-4 border-r border-gray-200 text-gray-500 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500 md:hidden"
        >
          <span class="sr-only">Open sidebar</span>
          <!-- Heroicon name: menu-alt-2 -->
          <svg
            class="h-6 w-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 12h16M4 18h7"
            />
          </svg>
        </button>
        <div class="flex-1 px-4 flex justify-between">
          <div class="flex-1 flex">
            <!---
            <form class="w-full flex md:ml-0" action="#" method="GET">
              <label for="search_field" class="sr-only">Search</label>
              <div class="relative w-full text-gray-400 focus-within:text-gray-600">
                <div class="absolute inset-y-0 left-0 flex items-center pointer-events-none">
                  < !-- Heroicon name: search -- >
                  <svg
                    class="h-5 w-5"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                    aria-hidden="true"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
                <input
                  id="search_field"
                  class="block w-full h-full pl-8 pr-3 py-2 border-transparent text-gray-900 placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-0 focus:border-transparent sm:text-sm"
                  placeholder="Search"
                  type="search"
                  name="search"
                />
              </div>
            </form>
          --></div>
          <div class="ml-4 flex items-center md:ml-6">
            <!-- Profile dropdown -->
            <div class="ml-3">
              <Hook0LoginMenu></Hook0LoginMenu>
            </div>
          </div>
        </div>
      </div>

      <main class="flex-1 relative overflow-y-auto focus:outline-none" tabindex="0">
        <div class="py-6 max-w-7xl mx-auto px-4 sm:px-6 md:px-8 h-96">
          <RouterView></RouterView>
          <Hook0Footer></Hook0Footer>
        </div>
      </main>
    </div>
  </div>
</template>

<style>
/* shared */
.ease-enter-active {
  @apply transition ease-out duration-100 z-50;
}

.ease-enter {
  @apply transform opacity-0 scale-95 duration-75;
}

.ease-enter-to {
  @apply transform opacity-100 scale-100;
}

.ease-leave-active {
  @apply transition ease-in duration-75;
}

.ease-leave {
  @apply transition ease-in duration-75;
}

.ease-leave-to {
  @apply transform opacity-0 scale-95;
}
</style>
