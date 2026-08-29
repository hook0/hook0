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
import {
  browserSessionStorage,
  currentPageSource,
  rememberSignupChannel,
} from './utils/signupChannel';

// Apply color mode from localStorage before app renders to prevent flash
{
  const theme = window.localStorage.getItem(LOCAL_STORAGE_KEY_THEME);
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  document.documentElement.classList.toggle('dark', resolveIsDark(theme, prefersDark));
}

// Remember where this tab came from while the referrer still points outside
// Hook0: once the router has navigated once, the browser reports the previous
// in-app page and the origin of the signup is lost.
rememberSignupChannel(currentPageSource(), browserSessionStorage());

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

// Matomo (settled before mount to avoid a Vue plugin warning). Matomo reads the instance config,
// and that call can fail — a failure that must not keep the app from mounting, since analytics is
// optional and the dashboard is not. Guarded so the app comes up either way rather than leaving a
// blank page behind a refused `/instance`.
void setupMatomo(app, router)
  .catch((error: unknown) => {
    console.error(error);
  })
  .then(() => {
    app.mount('#app');
  });
