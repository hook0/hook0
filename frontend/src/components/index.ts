import { App, Component } from 'vue';

export { default as Logo } from './Logo.vue';
export { default as MenuItem } from './MenuItem.vue';
export { default as Hook0Button } from '@/components/Hook0Button.vue';
export { default as Hook0Input } from '@/components/Hook0Input.vue';
export { default as Hook0Card } from '@/components/Hook0Card.vue';
export { default as Hook0CardHeader } from '@/components/Hook0CardHeader.vue';
export { default as Hook0CardFooter } from '@/components/Hook0CardFooter.vue';
export { default as Hook0CardContent } from '@/components/Hook0CardContent.vue';
export { default as Hook0CardContentLine } from '@/components/Hook0CardContentLine.vue';

export default {
  install: (app: App) => {
    Object.entries(module.exports as Component[]).forEach(([key, value]) =>
      key !== 'init' ? app.component(key, value) : null
    );
  },
};
