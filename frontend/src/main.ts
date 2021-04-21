// Tailwind <3
import './tailwind.css';

// Setup main app
import { createApp } from 'vue';
import router from './router';
import { Promised } from 'vue-promised';
import components from './components';

import Root from './Root.vue';

// Create and mount the root instance.
const app = createApp(Root);

// Vue - UIRouter
app.use(router);

// Vue - promised
app.component('Promised', Promised);

// Vue - Hook0 own components
//app.use(components);

// Mount the app
app.mount('#app');
