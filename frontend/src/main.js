import { createApp } from 'vue';
import { Promised } from 'vue-promised';
import App from './App.vue';
import './tailwind.css';

const Vue = createApp(App);

// Import components
require('./components')(Vue);
Vue.component('Promised', Promised);

Vue.mount('#app');
