// Tailwind <3
import './tailwind.css';

// Setup main app
import {createApp, Plugin} from 'vue';
import router from './router';
import {Promised} from 'vue-promised';
// import components from './components';
import {KeycloakPlugin} from './iam';
import vClickOutside from "click-outside-vue3";

import Root from './Root.vue';
import {RouteLocation, Router} from 'vue-router';

// @ts-ignore
import VueFormJsonSchema from 'vue-form-json-schema/src/index';

// FontAwesome
import {library} from '@fortawesome/fontawesome-svg-core';

import {
  faFolderTree,
  faLink,
  faFileLines,
  faBook,
  faGear,
  faChevronDown,
  faChevronLeft,
  faChevronRight, faCircleNotch, faPlus,
  faRocket,
  faSitemap,
  faSpinner, faTrash, faUserCircle
} from '@fortawesome/free-solid-svg-icons';
import {FontAwesomeIcon} from '@fortawesome/vue-fontawesome';


// Create and mount the root instance.
const app = createApp(Root);

// Vue - UIRouter
app.use(router);

// Expose Keycloak JS
app.use(KeycloakPlugin);


// font-awesome
// Add here
library.add(faFolderTree,
  faLink,
  faBook,
  faFileLines,
  faGear, faChevronDown, faRocket, faUserCircle, faPlus, faTrash, faSitemap, faSpinner, faCircleNotch, faChevronLeft, faChevronRight);
app.component('font-awesome-icon', FontAwesomeIcon);

// @ts-ignore
app.use(vClickOutside);


// Vue - promised
app.component('Promised', Promised);

// eslint-disable-next-line
app.component('vue-form-json-schema', VueFormJsonSchema);


// Vue - Hook0 own components
import Hook0Alert from "@/components/Hook0Alert.vue";
import Hook0Button from "@/components/Hook0Button.vue";
import Hook0Input from "@/components/Hook0Input.vue";
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";
import Hook0Dropdown from "@/components/Hook0Dropdown.vue";
import Hook0DropdownMenuItemLink from "@/components/Hook0DropdownMenuItemLink.vue";
import Hook0DropdownMenuItems from "@/components/Hook0DropdownMenuItems.vue";
import Hook0DropdownMenuItemText from "@/components/Hook0DropdownMenuItemText.vue";
import Hook0Text from "@/components/Hook0Text.vue";
import Hook0Icon from "@/components/Hook0Icon.vue";
import Hook0Loader from "@/components/Hook0Loader.vue";

app.component('Hook0Alert', Hook0Alert);
app.component('Hook0Loader', Hook0Loader);
app.component('Hook0Button', Hook0Button);
app.component('Hook0Text', Hook0Text);
app.component('Hook0Input', Hook0Input);
app.component('Hook0Icon', Hook0Icon);
app.component('Hook0Card', Hook0Card);
app.component('Hook0CardHeader', Hook0CardHeader);
app.component('Hook0CardFooter', Hook0CardFooter);
app.component('Hook0CardContent', Hook0CardContent);
app.component('Hook0CardContentLine', Hook0CardContentLine);

app.component('Hook0Dropdown', Hook0Dropdown);
app.component('Hook0DropdownMenuItems', Hook0DropdownMenuItems);
app.component('Hook0DropdownMenuItemLink', Hook0DropdownMenuItemLink);
app.component('Hook0DropdownMenuItemText', Hook0DropdownMenuItemText);

// Mount the app
app.mount('#app');
