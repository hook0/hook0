<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router';
import { computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Notivue, Notification, NotificationProgress } from 'notivue';
import { useAuthStore } from '@/stores/auth';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useEntityContext } from '@/composables/useEntityContext';
import Hook0TopNav from '@/components/Hook0TopNav.vue';
import Hook0MobileTabBar from '@/components/Hook0MobileTabBar.vue';
import Hook0MobileDrawer from '@/components/Hook0MobileDrawer.vue';
import Hook0CommandPalette from '@/components/Hook0CommandPalette.vue';
import Hook0ShortcutsCheatSheet from '@/components/Hook0ShortcutsCheatSheet.vue';
import Hook0RefetchBar from '@/components/Hook0RefetchBar.vue';

const { t } = useI18n();
const route = useRoute();
const authStore = useAuthStore();
const contextStore = useContextStore();
const uiStore = useUiStore();

// Keyboard shortcuts
useKeyboardShortcuts();

// Entity context - syncs org/app names from route params to context store
useEntityContext();

// Track context from route params
watch(
  () => route.params,
  (params) => {
    contextStore.updateFromRoute(params as Record<string, string>);
  },
  { immediate: true }
);

// Track recent pages
watch(
  () => route.fullPath,
  (path) => {
    if (authStore.isAuthenticated && route.name) {
      uiStore.addRecentPage(path, String(route.name));
    }
  }
);

const showFullScreen = computed(() => {
  return !authStore.isAuthenticated || route.meta.fullScreen === true;
});
</script>

<template>
  <!-- Notifications -->
  <Notivue v-slot="item">
    <Notification :item="item" data-test="toast-notification">
      <NotificationProgress :item="item" />
    </Notification>
  </Notivue>

  <!-- Command Palette (global overlay) -->
  <Hook0CommandPalette v-if="authStore.isAuthenticated" />

  <!-- Keyboard Shortcuts Cheat Sheet -->
  <Hook0ShortcutsCheatSheet />

  <!-- Background refetch progress bar -->
  <Hook0RefetchBar />

  <!-- Authenticated layout -->
  <div v-if="!showFullScreen" class="hook0-app">
    <a href="#main-content" class="skip-link">{{ t('nav.skipToContent') }}</a>

    <!-- Top Navigation (Clerk-style 2-level navigation) -->
    <Hook0TopNav />

    <!-- Main content area -->
    <main id="main-content" class="hook0-app__main">
      <div class="hook0-app__container">
        <!-- Page content -->
        <!-- Wrapper div required: with mode="out-in", Vue listens for transitionend
             on the leaving component's root element. During route change, reactive deps
             (contextStore params) are cleared immediately, causing the old component to
             re-render and swap its root DOM node. This orphans the transitionend listener,
             blocking the enter transition forever. The stable div ensures the transition
             target never changes regardless of internal re-renders. -->
        <RouterView v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <div :key="route.fullPath">
              <component :is="Component" />
            </div>
          </Transition>
        </RouterView>
      </div>
    </main>

    <!-- Mobile bottom tab bar -->
    <Hook0MobileTabBar />

    <!-- Mobile drawer navigation -->
    <Hook0MobileDrawer />
  </div>

  <!-- Unauthenticated / fullscreen layout -->
  <div v-else class="min-h-screen" style="background-color: var(--color-bg-secondary)">
    <RouterView />
  </div>
</template>

<style scoped>
.skip-link {
  position: absolute;
  top: -100%;
  left: 0;
  padding: 0.5rem 1rem;
  background: var(--color-primary);
  color: var(--color-bg-primary);
  z-index: 100;
  border-radius: var(--radius-md);
}

.skip-link:focus {
  top: 0.5rem;
  left: 0.5rem;
}

.hook0-app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background-color: var(--color-bg-secondary);
}

.hook0-app__main {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 5rem; /* Space for mobile tab bar */
}

@media (min-width: 768px) {
  .hook0-app__main {
    padding-bottom: 0;
  }
}

.hook0-app__container {
  max-width: 80rem;
  margin: 0 auto;
  padding: 1.5rem 1rem;
}

@media (min-width: 640px) {
  .hook0-app__container {
    padding: 1.5rem 1.5rem;
  }
}

@media (min-width: 1024px) {
  .hook0-app__container {
    padding: 2rem 2rem;
  }
}
</style>
