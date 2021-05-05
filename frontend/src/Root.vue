<template>
  <!--  <UIRouter :router="router" :states="states" :plugins="plugins">-->
  <div class="h-screen flex overflow-hidden bg-gray-100">
    <!-- Static sidebar for desktop -->
    <div class="hidden md:flex md:flex-shrink-0">
      <div class="flex flex-col w-64">
        <!-- Sidebar component, swap this element with another sidebar if you like -->
        <div class="flex flex-col h-0 flex-1">
          <div class="flex items-center h-16 flex-shrink-0 px-4 bg-gray-900">
            <logo></logo>
          </div>
          <div class="flex h-16 flex-shrink-0 bg-gray-900">
            <organization-selector></organization-selector>
          </div>
          <div class="flex-1 flex flex-col overflow-y-auto">
            <nav class="flex-1 px-2 py-4 bg-gray-800 space-y-1">
              <menu-item v-for="(item, index) in items" :key="index"
                         :active="item.route.name === $route.name"
                         :name="item.name"
                         @click="$router.push(item.route)">
                <icon :kind="item.icon"></icon>
              </menu-item>
            </nav>
          </div>
        </div>
      </div>
    </div>
    <div class="flex flex-col w-0 flex-1 overflow-hidden">
      <div class="relative z-10 flex-shrink-0 flex h-16 bg-white shadow">
        <button
          class="px-4 border-r border-gray-200 text-gray-500 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500 md:hidden">
          <span class="sr-only">Open sidebar</span>
          <!-- Heroicon name: menu-alt-2 -->
          <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
               stroke="currentColor"
               aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7"/>
          </svg>
        </button>
        <div class="flex-1 px-4 flex justify-between">
          <div class="flex-1 flex">
            <form class="w-full flex md:ml-0" action="#" method="GET">
              <label for="search_field" class="sr-only">Search</label>
              <div class="relative w-full text-gray-400 focus-within:text-gray-600">
                <div class="absolute inset-y-0 left-0 flex items-center pointer-events-none">
                  <!-- Heroicon name: search -->
                  <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor"
                       aria-hidden="true">
                    <path fill-rule="evenodd"
                          d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                          clip-rule="evenodd"/>
                  </svg>
                </div>
                <input id="search_field"
                       class="block w-full h-full pl-8 pr-3 py-2 border-transparent text-gray-900 placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-0 focus:border-transparent sm:text-sm"
                       placeholder="Search" type="search" name="search">
              </div>
            </form>
          </div>
          <div class="ml-4 flex items-center md:ml-6">
            <button
              class="bg-white p-1 rounded-full text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
              <span class="sr-only">View notifications</span>
              <!-- Heroicon name: bell -->
              <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                   stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
              </svg>
            </button>

            <!-- Profile dropdown -->
            <div class="ml-3 relative">
              <div>
                <button
                  id="user-menu"
                  class="max-w-xs bg-white flex items-center text-sm rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                  aria-haspopup="true">
                  <span class="sr-only">Open user menu</span>
                  <img class="h-8 w-8 rounded-full"
                       src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                       alt="">
                </button>
              </div>
              <!--
                Profile dropdown panel, show/hide based on dropdown state.

                Entering: "transition ease-out duration-100"
                  From: "transform opacity-0 scale-95"
                  To: "transform opacity-100 scale-100"
                Leaving: "transition ease-in duration-75"
                  From: "transform opacity-100 scale-100"
                  To: "transform opacity-0 scale-95"
              -->
              <div
                class="origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white ring-1 ring-black ring-opacity-5"
                role="menu" aria-orientation="vertical" aria-labelledby="user-menu">
                <a href="#" class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100" role="menuitem">Your
                  Profile</a>

                <a href="#" class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                   role="menuitem">Settings</a>

                <a href="#" class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100" role="menuitem">Sign
                  out</a>
              </div>
            </div>
          </div>
        </div>
      </div>

      <main class="flex-1 relative overflow-y-auto focus:outline-none" tabindex="0">
        <div class="py-6">
          <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
            <h1 class="text-2xl font-semibold text-gray-900">Dashboard</h1>
          </div>
          <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
            <div class="py-4">
              <div class="border-4 border-dashed border-gray-200 rounded-lg h-96">
                <router-view></router-view>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script lang="ts">
import {Icon, Logo, MenuItem} from './components';
import {Vue, Options} from 'vue-class-component';
import OrganizationSelector from "@/pages/organizations/OrganizationSelector.vue";
import {list} from "@/pages/organizations/OrganizationService";
import {routes} from "@/routes";

@Options({
  components: {Logo, OrganizationSelector, Icon, MenuItem},

  data: () => {
    return {

      items: [
        {
          name: "Dashboard",
          icon: 'home',
          route: {
            name: routes.ApplicationsList
          }
        },
        {
          name: "Organization",
          icon: 'team',
          route: {
            name: routes.OrganizationsDetail
          }
        },
        {
          name: "Applications",
          icon: 'document',
          route: {
            name: routes.ApplicationsList
          }
        },
        {
          name: "Event Types",
          icon: 'document',
          route: {
            name: routes.EventTypesList
          }
        },
        {
          name: "Webhooks",
          icon: 'folder',
          route: {
            name: routes.WebhooksList
          }
        },
        {
          name: "Logs",
          icon: 'calendar',
          route: {
            name: routes.LogsList
          }
        },
        {
          name: "Settings",
          icon: 'document',
          route: {
            name: routes.Settings
          }
        },
        {
          name: "API Documentation",
          icon: 'document',
          route: {
            name: routes.APIDocumentation
          }
        },
      ]
    };
  },
  mounted() {
    if (!this.$route.params.organization_id) {
      list().then(organizations => {
        this.$router.push({
          name: routes.OrganizationsDetail,
          params: {
            organization_id: organizations[0].organization_id
          }
        })
      });
    }
  }
})
export default class Root extends Vue {

};
</script>

<style>
</style>
