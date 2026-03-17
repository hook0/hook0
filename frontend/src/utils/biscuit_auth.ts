import {
  Biscuit,
  BlockBuilder,
  block,
  PublicKey,
  SignatureAlgorithm,
} from '@biscuit-auth/biscuit-wasm';
import { UUID } from '@/http.ts';

function uuidParse(uuid: string): Uint8Array {
  const hex = uuid.replace(/-/g, '');
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

// Re-export from instance-config for backward compatibility
export { getInstanceConfig } from './instance-config';
export type { InstanceConfig } from './instance-config';

export function parseBiscuitFromBase64(biscuit_token: string, biscuitPublicKey: string): Biscuit {
  const public_key: PublicKey = PublicKey.fromString(biscuitPublicKey, SignatureAlgorithm.Ed25519);
  try {
    return Biscuit.fromBase64(biscuit_token, public_key);
  } catch (e) {
    throw new Error(
      'An error occurred while parsing the service token. Your biscuit may be invalid. If the error persists, please contact support.'
    );
  }
}

export interface BiscuitBlockInfo {
  index: number;
  source: string;
}

export function getBiscuitBlocks(biscuit: Biscuit): Array<BiscuitBlockInfo> {
  const count = biscuit.countBlocks();
  const blocks: Array<BiscuitBlockInfo> = [];
  for (let i = 0; i < count; i++) {
    blocks.push({ index: i, source: biscuit.getBlockSource(i) });
  }
  return blocks;
}

export function attenuateBiscuit(
  biscuit_token: string,
  application_id: UUID | null,
  expired_at: Date | null,
  biscuitPublicKey: string
): Biscuit {
  const biscuit = parseBiscuitFromBase64(biscuit_token, biscuitPublicKey);
  const application_id_bytes = application_id ? uuidParse(application_id) : null;

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

export function attenuateBiscuitWithDatalog(
  biscuit_token: string,
  application_id: UUID | null,
  expired_at: Date | null,
  customDatalog: string,
  biscuitPublicKey: string
): Biscuit {
  const biscuit = parseBiscuitFromBase64(biscuit_token, biscuitPublicKey);
  const application_id_bytes = application_id ? uuidParse(application_id) : null;
  const builder = new BlockBuilder();

  if (application_id_bytes) {
    builder.addCode(`check if application_id(hex:${bytesToHex(application_id_bytes)});`);
  }

  if (expired_at) {
    builder.addCode(`check if time($t), $t < ${expired_at.toISOString()};`);
  }

  if (customDatalog.trim().length > 0) {
    builder.addCode(customDatalog.trim());
  }

  return biscuit.appendBlock(builder);
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}
