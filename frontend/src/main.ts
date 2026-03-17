// Styles
import './assets/styles/tailwind.css';
import './assets/styles/transitions.css';

// Self-hosted fonts
import '@fontsource-variable/inter';
import '@fontsource/jetbrains-mono/400.css';
import '@fontsource/jetbrains-mono/500.css';
import '@fontsource/jetbrains-mono/700.css';

// Core
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import router from './router';

// Plugins
import { setupQueryPlugin } from './plugins/query';
import { setupI18n } from './plugins/i18n';
import { setupMatomo } from './plugins/matomo';

// Stores
import { useAuthStore } from './stores/auth';

// Notivue
import { createNotivue } from 'notivue';
import 'notivue/notification.css';
import 'notivue/animations.css';
import 'notivue/notification-progress.css';

// Root component
import App from './App.vue';

import { LOCAL_STORAGE_KEY_THEME, resolveIsDark } from './constants/theme';

// Apply color mode from localStorage before app renders to prevent flash
{
  const theme = window.localStorage.getItem(LOCAL_STORAGE_KEY_THEME);
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  document.documentElement.classList.toggle('dark', resolveIsDark(theme, prefersDark));
}

// Create app
const app = createApp(App);

// Pinia (must be first for stores to work)
const pinia = createPinia();
app.use(pinia);

// Vue Router
app.use(router);

// TanStack Query
setupQueryPlugin(app);

// vue-i18n
setupI18n(app);

// Notivue
const notivue = createNotivue({
  position: 'top-right',
  limit: 4,
  enqueue: true,
  avoidDuplicates: true,
  animations: {
    enter: 'Notivue__enter',
    leave: 'Notivue__leave',
    clearAll: 'Notivue__clearAll',
  },
  pauseOnHover: true,
  transition: 'transform 0.35s cubic-bezier(0.5, 1, 0.25, 1)',
});
app.use(notivue);

// Matomo
setupMatomo(app, router);

// Initialize auth store and router guards
const authStore = useAuthStore();
authStore.initialize();
authStore.setupRouterGuard();

// Mount
app.mount('#app');
