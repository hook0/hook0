import { App, Component } from 'vue';

export { default as Logo } from './Logo.vue';
export { default as MenuItem } from './MenuItem.vue';
export { default as Icon } from './Icon.vue';

export default {
  install: (app: App) => {
    Object.entries(module.exports as Component[]).forEach(([key, value]) =>
      key !== 'init' ? app.component(key, value) : null
    );
  },
};
