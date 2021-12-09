// Tailwind <3
import './tailwind.css';

// Setup main app
import { createApp, Plugin } from 'vue';
import router from './router';
import { Promised } from 'vue-promised';
// import components from './components';
import { KeycloakPlugin } from './iam';

import Root from './Root.vue';
import { RouteLocation, Router } from 'vue-router';

// Create and mount the root instance.
const app = createApp(Root);

// Vue - UIRouter
app.use(router);

// Expose Keyloack JS
app.use(KeycloakPlugin);

// wrap router in order to keep our organization_id when present
const RouterOrgPlugin: Plugin = {
  install: (app, _options) => {
    const router = app.config.globalProperties.$router as Router;
    router.push = ((push) => {
      return (to: RouteLocation) => {
        const route = app.config.globalProperties.$route as RouteLocation;
        return push({
          ...to,
          params: {
            ...to.params,
            organization_id: route.params.organization_id || null,
          },
        });
      };
    })(router.push);// eslint-disable-line @typescript-eslint/unbound-method
  }
};
app.use(RouterOrgPlugin);

// Vue - promised
app.component('Promised', Promised);

// Vue - Hook0 own components
//app.use(components); // TODO: finish or clean up

// Mount the app
app.mount('#app');
