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

// Initialize auth store and router guards
const authStore = useAuthStore();
authStore.initialize();
authStore.setupRouterGuard();

// Matomo (must resolve before mount to avoid Vue plugin warning)
void setupMatomo(app, router).then(() => {
  app.mount('#app');
});
