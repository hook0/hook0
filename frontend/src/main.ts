// Tailwind <3
import './tailwind.css';

// Setup main app
import { createApp } from 'vue';
import router from './router';
import { Promised } from 'vue-promised';
import { AuthPlugin } from './iam';
import { createNotivue } from 'notivue';

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
} from '@fortawesome/free-solid-svg-icons';
import { faToggleOn } from '@fortawesome/free-solid-svg-icons/faToggleOn';
import { faToggleOff } from '@fortawesome/free-solid-svg-icons/faToggleOff';

import 'notivue/notification.css'; // Only needed if using built-in notifications
import 'notivue/animations.css'; // Only needed if using built-in animations
import 'notivue/notification-progress.css';

const notivue = createNotivue({
  position: 'top-right',
  limit: 4,
  enqueue: true,
  avoidDuplicates: true,
  notifications: {
    global: {
      duration: 10000,
    },
  },
  pauseOnHover: true,
});

// Create and mount the root instance.
const app = createApp(Root);

// Vue Router
app.use(router);

// Authentication & authorization
app.use(AuthPlugin);

// Notifications with notivue
app.use(notivue);

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
