export { default as Logo } from './Logo.vue';
export { default as MenuItem } from './MenuItem.vue';
export { default as Icon } from './Icon.vue';

export default {
  install: (app: any) => {
    Object.entries(module.exports).forEach(([key, value]) =>
      key !== 'init' ? app.component(key, value) : null
    );
  },
};
