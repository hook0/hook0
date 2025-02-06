// Tailwind <3
import './tailwind.css';

// Setup main app
import { createApp } from 'vue';
import router from './router';
import { Promised } from 'vue-promised';
import VueMatomo from 'vue-matomo';
import * as Sentry from '@sentry/vue';

import { AuthPlugin } from './iam';
import Root from './Root.vue';

// FontAwesome
import { library } from '@fortawesome/fontawesome-svg-core';

// https://fontawesome.com/v6/search?o=r&m=free&s=solid
import {
  faCheck,
  faFolderTree,
  faLink,
  faFileLines,
  faBook,
  faGear,
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faCircleNotch,
  faPlus,
  faRocket,
  faSitemap,
  faSpinner,
  faTrash,
  faUserCircle,
  faArrowsRotate,
  faArrowUpRightFromSquare,
  faMinus,
  faQuestion,
  faXmark,
  faPause,
  faCalendar,
  faKey,
  faFileContract,
  faSliders,
  faMoneyCheckDollar,
  faUsers,
  faFolder,
  faDatabase,
  faEye,
  faPen,
  faCopy,
  faCircle,
} from '@fortawesome/free-solid-svg-icons';
import { faToggleOn } from '@fortawesome/free-solid-svg-icons/faToggleOn';
import { faToggleOff } from '@fortawesome/free-solid-svg-icons/faToggleOff';
import { createNotivue } from 'notivue';

// Create and mount the root instance.
const app = createApp(Root);

// Vue Router
app.use(router);

// Authentication & authorization
app.use(AuthPlugin);

if (import.meta.env.VITE_SENTRY_DSN) {
  Sentry.init({
    app,
    dsn: import.meta.env.VITE_SENTRY_DSN,
    integrations: [Sentry.browserTracingIntegration({ router }), Sentry.replayIntegration()],
    // Tracing
    tracesSampleRate: 1.0, //  Capture 100% of the transactions
    // Set 'tracePropagationTargets' to control for which URLs distributed tracing should be enabled
    tracePropagationTargets: ['http://localhost:5173', /^http:\/\/localhost:8080\/api/],
    // Session Replay
    replaysSessionSampleRate: 0.1, // This sets the sample rate at 10%. You may want to change it to 100% while in development and then sample at a lower rate in production.
    replaysOnErrorSampleRate: 1.0, // If you're not already sampling the entire session, change the sample rate to 100% when sampling sessions where errors occur.
  });
}

// Notivue
import 'notivue/notification.css'; // Only needed if using built-in notifications
import 'notivue/animations.css'; // Only needed if using built-in animations
import 'notivue/notification-progress.css';

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

import { getInstanceConfig } from './utils/biscuit_auth';
void getInstanceConfig().then((config) => {
  if (config.matomo) {
    app.use(VueMatomo, {
      host: config.matomo.url,
      siteId: config.matomo.site_id,
      router,
      disableCookies: true,
      enableHeartBeatTimer: true,
      heartBeatTimerInterval: 15,
    });
  }
});

// font-awesome
// Add here
library.add(
  faFolderTree,
  faLink,
  faBook,
  faFileLines,
  faArrowsRotate,
  faArrowUpRightFromSquare,
  faMinus,
  faPlus,
  faToggleOn,
  faToggleOff,
  faGear,
  faChevronDown,
  faRocket,
  faUserCircle,
  faTrash,
  faSitemap,
  faSpinner,
  faCircleNotch,
  faChevronLeft,
  faChevronRight,
  faMoneyCheckDollar,
  faUsers,
  faFolder,
  faDatabase,
  faEye,
  faPen,
  faCopy,
  faCircle,

  //RequestAttemptStatus
  faCheck,
  faQuestion,
  faXmark,
  faPause,
  faCalendar,
  // faSpinner

  // Navigation
  faKey,
  faFileContract,
  faSliders
);

// Vue - promised
app.component('Promised', Promised);

// Mount the app
app.mount('#app');
