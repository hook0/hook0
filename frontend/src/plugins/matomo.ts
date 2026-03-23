import MatomoModule from 'vue-matomo';
import type { App, Plugin } from 'vue';
import type { Router } from 'vue-router';
import { getInstanceConfig } from '@/utils/instance-config';

// vue-matomo UMD export may resolve to a namespace object under Vite ESM interop
const VueMatomo: Plugin =
  typeof MatomoModule === 'function'
    ? (MatomoModule as Plugin)
    : ((MatomoModule as Record<string, unknown>).default as Plugin);

export async function setupMatomo(app: App, router: Router): Promise<void> {
  const config = await getInstanceConfig();
  if (config.matomo && VueMatomo) {
    app.use(VueMatomo, {
      host: config.matomo.url,
      siteId: config.matomo.site_id,
      router,
      disableCookies: true,
      enableHeartBeatTimer: true,
      heartBeatTimerInterval: 15,
    });
  }
}
