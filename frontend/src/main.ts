// Tailwind <3
import './tailwind.css';

// Setup main app
import { createApp } from 'vue';
import router from './router';
import { Promised } from 'vue-promised';
import components from './components';

import Root from './Root.vue';
import { RouteLocation } from 'vue-router';

// Create and mount the root instance.
const app = createApp(Root);

// Vue - UIRouter
app.use(router);

// wrap router in order to keep our organization_id when present
app.config.globalProperties.$router.push = (function(push) {
  return function(to: RouteLocation) {
    return push({
      ...to,
      params: {
        ...to.params,
        organization_id: app.config.globalProperties.$route.params.organization_id || null,
      },
    });
  };
})(app.config.globalProperties.$router.push);

// Vue - promised
app.component('Promised', Promised);

// Vue - Hook0 own components
//app.use(components);

// Mount the app
app.mount('#app');
