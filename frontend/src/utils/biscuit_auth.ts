import { Biscuit, block, PublicKey } from '@biscuit-auth/biscuit-wasm';
import http, { handleError, Problem, UUID } from '@/http.ts';
import { parse } from 'uuid';
import { AxiosError, AxiosResponse } from 'axios';
import { components } from '@/types.ts';

type definitions = components['schemas'];
export type InstanceConfig = definitions['InstanceConfig'];

let instanceConfigCache: InstanceConfig | null = null;

export function getInstanceConfig(): Promise<InstanceConfig> {
  console.log(!instanceConfigCache);
  if (instanceConfigCache) {
    return Promise.resolve(instanceConfigCache);
  }

  return http.get('/instance', {}).then(
    (res: AxiosResponse<InstanceConfig>) => {
      instanceConfigCache = res.data;
      return res.data;
    },
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function attenuateBiscuit(
  biscuit_token: string,
  application_id: UUID | null,
  expired_at: Date | null,
  biscuitPublicKey: string
): Biscuit {
  const public_key: PublicKey = PublicKey.fromString(biscuitPublicKey);
  let biscuit: Biscuit;
  try {
    biscuit = Biscuit.fromBase64(biscuit_token, public_key);
  } catch (e) {
    throw new Error(
      'An error occurred while generating the service token. Your biscuit may be invalid. If the error persists, please contact support.'
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const application_id_bytes = application_id ? parse(application_id) : null;

  if (application_id_bytes && expired_at) {
    return biscuit.appendBlock(
      block`check if application_id(${application_id_bytes}); check if time($t), $t < ${expired_at};`
    );
  } else if (application_id_bytes) {
    return biscuit.appendBlock(block`check if application_id(${application_id_bytes});`);
  } else if (expired_at) {
    return biscuit.appendBlock(block`check if time($t), $t < ${expired_at};`);
  }
  return biscuit;
}
