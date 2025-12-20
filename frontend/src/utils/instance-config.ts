import http, { handleError, Problem } from '@/http.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { components } from '@/types.ts';

type definitions = components['schemas'];
export type InstanceConfig = definitions['InstanceConfig'];

let instanceConfigCache: Promise<InstanceConfig> | null = null;

export function getInstanceConfig(): Promise<InstanceConfig> {
  if (instanceConfigCache) {
    return instanceConfigCache;
  } else {
    const promise = http.get('/instance', {}).then(
      (res: AxiosResponse<InstanceConfig>) => {
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
