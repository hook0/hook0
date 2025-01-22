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
        await formbricks.reset().catch((e) => {
          console.warn(`Formbricks reset failed: ${e}`);
        });
        await formbricks
          .init({
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            apiHost: instanceConfig.formbricks_api_host,
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            environmentId: instanceConfig.formbricks_environment_id,
            userId: storedState.userId,
          })
          .catch((e) => {
            console.warn(`Formbricks initialization failed: ${e}`);
          });
      } else {
        console.error('Formbricks initialization failed: storedState.userId is missing');
      }
    }
  }
}
