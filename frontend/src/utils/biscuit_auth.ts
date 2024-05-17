import featureFlags from '@/feature-flags.ts';
import { Biscuit, block, PublicKey } from '@biscuit-auth/biscuit-wasm';
import { UUID } from '@/http.ts';
import { parse } from 'uuid';

const biscuitPublicKey = verifyBiscuitPublicKey();

export function verifyBiscuitPublicKey(): PublicKey {
  try {
    return PublicKey.fromString(
      featureFlags.getOrElse('BISCUIT_PUBLIC_KEY', import.meta.env.VITE_BISCUIT_PUBLIC_KEY ?? '')
    );
  } catch (e) {
    throw new Error('Invalid BISCUIT_PUBLIC_KEY');
  }
}

export function attenuateBiscuit(
  biscuit_string: string,
  application_id: UUID | null,
  expired_at: Date | null
): Biscuit {
  let biscuit: Biscuit;
  try {
    biscuit = Biscuit.fromBase64(biscuit_string, biscuitPublicKey);
  } catch (e) {
    console.log('Failed to parse biscuit', e);
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
  install(app) {
    try {
      if (typeof WebAssembly === "object" && typeof WebAssembly.instantiate === "function") {
        const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
        if (!(module instanceof WebAssembly.Module) || !(new WebAssembly.Instance(module) instanceof WebAssembly.Instance)) {
          throw new Error();
        }
      } else {
        throw new Error();
      }
    } catch (e) {
      Notify.error({
        title: 'WebAssembly Unsupported',
        message: 'Your browser does not support WebAssembly. You need to enable it on your browser or install a browser that supports it.',
      });
    }
  }
};