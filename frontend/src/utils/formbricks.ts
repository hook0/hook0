import { getInstanceConfig } from '@/utils/biscuit_auth';
import { State } from '@/iam';
import formbricks from '@formbricks/js';

export async function initializeFormbricks(storedState: State) {
  if (typeof window !== 'undefined') {
    const instanceConfig = await getInstanceConfig();
    if (
      instanceConfig &&
      instanceConfig.formbricks_api_host &&
      instanceConfig.formbricks_environment_id
    ) {
      if (storedState && storedState.userId) {
        //await formbricks.reset();
        await formbricks
          .init({
            apiHost: instanceConfig.formbricks_api_host,
            environmentId: instanceConfig.formbricks_environment_id,
            userId: storedState.userId,
          })
          .catch((e) => {
            console.warn(`Formbricks initialization failed: ${e}`);
          });
      }
    }
  }
}
