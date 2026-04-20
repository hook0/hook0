import http, { handleError, Problem } from '@/http.ts';
import { AxiosError, type AxiosResponse } from 'axios';
import { components } from '@/types.ts';
import type { RetryScheduleLimits } from '@/pages/organizations/retry_schedules/retrySchedule.types';

type definitions = components['schemas'];
// TODO: drop the intersection once types.ts is regenerated against the branch API;
// `retry_schedule` will then appear natively on InstanceConfig.
export type InstanceConfig = definitions['InstanceConfig'] & {
  retry_schedule?: RetryScheduleLimits;
};

let instanceConfigCache: Promise<InstanceConfig> | null = null;

export function getInstanceConfig(): Promise<InstanceConfig> {
  if (instanceConfigCache) {
    return instanceConfigCache;
  } else {
    const promise = http.get<InstanceConfig>('/instance', {}).then(
      (res) => {
        return res.data;
      },
      (err: AxiosError<AxiosResponse<Problem>>) => {
        instanceConfigCache = null;
        return Promise.reject(handleError(err));
      }
    );
    instanceConfigCache = promise;
    return promise;
  }
}
