import VueMatomo from 'vue-matomo';
import type { App } from 'vue';
import type { Router } from 'vue-router';
import { getInstanceConfig } from '@/utils/instance-config';

export async function setupMatomo(app: App, router: Router): Promise<void> {
  const config = await getInstanceConfig();
  if (config.matomo) {
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
