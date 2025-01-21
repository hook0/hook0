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
        await formbricks
          .init({
            apiHost: instanceConfig.formbricks_api_host,
            environmentId: 'cm669w0ca0002l703bc2n76qc',
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
