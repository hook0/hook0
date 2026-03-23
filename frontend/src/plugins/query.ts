import { VueQueryPlugin, type VueQueryPluginOptions } from '@tanstack/vue-query';
import type { App } from 'vue';

const queryPluginOptions: VueQueryPluginOptions = {
  queryClientConfig: {
    defaultOptions: {
      queries: {
        staleTime: 30_000, // 30 seconds
        gcTime: 5 * 60_000, // 5 minutes garbage collection
        retry: 1,
        refetchOnWindowFocus: true,
        refetchOnReconnect: true,
      },
      mutations: {
        retry: 0,
      },
    },
  },
};

export function setupQueryPlugin(app: App): void {
  app.use(VueQueryPlugin, queryPluginOptions);
}
