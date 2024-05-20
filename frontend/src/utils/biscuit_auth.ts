import { Biscuit, block, PublicKey } from '@biscuit-auth/biscuit-wasm';
import http, { handleError, Problem, UUID } from '@/http.ts';
import { parse } from 'uuid';
import { push } from 'notivue';
import { App, Plugin } from 'vue';
import { AxiosError, AxiosResponse } from 'axios';
import { components } from '@/types.ts';

type definitions = components['schemas'];
export type InstanceConfig = definitions['InstanceConfig'];

export function getInstanceConfig(): Promise<InstanceConfig> {
  return http.get('/instance', {}).then(
    (res: AxiosResponse<InstanceConfig>) => res.data,
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

export const checkWebAssembly: Plugin = {
  install(_app: App) {
    try {
      if (typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function') {
        const module = new WebAssembly.Module(
          Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00)
        );
        if (
          !(module instanceof WebAssembly.Module) ||
          !(new WebAssembly.Instance(module) instanceof WebAssembly.Instance)
        ) {
          throw new Error();
        }
      } else {
        throw new Error();
      }
    } catch (e) {
      push.error({
        title: 'WebAssembly Unsupported',
        message:
          'Your browser does not support WebAssembly. You need to enable it on your browser or install a browser that supports it.',
      });
    }
  },
};
