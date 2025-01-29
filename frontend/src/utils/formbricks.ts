import { getInstanceConfig } from '@/utils/biscuit_auth';
import formbricks from '@formbricks/js';
import { UUID } from '@/http';

export async function initializeFormbricks(userId: UUID) {
  if (typeof window !== 'undefined') {
    const instanceConfig = await getInstanceConfig();
    if (
      instanceConfig &&
      instanceConfig.formbricks &&
      instanceConfig.formbricks.api_host &&
      instanceConfig.formbricks.environment_id
    ) {
      await formbricks.reset().catch((e) => {
        console.warn(`Formbricks reset failed: ${e}`);
      });
      await formbricks
        .init({
          apiHost: instanceConfig.formbricks.api_host,
          environmentId: instanceConfig.formbricks.environment_id,
          userId,
        })
        .catch((e) => {
          console.warn(`Formbricks initialization failed: ${e}`);
        });
    }
  }
}
