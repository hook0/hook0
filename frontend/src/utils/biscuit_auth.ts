import featureFlags from '@/feature-flags.ts';
import { Biscuit, block, PublicKey } from '@biscuit-auth/biscuit-wasm';
import { UUID } from '@/http.ts';

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

export function getDeserializedBiscuit(biscuit: string) {
  try {
    return Biscuit.fromBase64(biscuit, biscuitPublicKey).toString();
  } catch (e) {
    return {
      title: 'Invalid Biscuit',
      message: 'The biscuit is invalid',
    };
  }
}

export function attenuateBiscuitToApplicationOnly(biscuit: Biscuit, application_id: UUID): Biscuit {
  return biscuit.appendBlock(block`check application_id = ${application_id};`);
}
