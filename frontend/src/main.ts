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
import Hook0Alert from "@/components/Hook0Alert.vue";
import Hook0Button from "@/components/Hook0Button.vue";
import Hook0Input from "@/components/Hook0Input.vue";
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";

app.component('Hook0Alert', Hook0Alert);
app.component('Hook0Button', Hook0Button);
app.component('Hook0Input', Hook0Input);
app.component('Hook0Card', Hook0Card);
app.component('Hook0CardHeader', Hook0CardHeader);
app.component('Hook0CardFooter', Hook0CardFooter);
app.component('Hook0CardContent', Hook0CardContent);
app.component('Hook0CardContentLine', Hook0CardContentLine);

// Mount the app
app.mount('#app');
