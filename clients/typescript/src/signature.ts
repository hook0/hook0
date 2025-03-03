import * as crypto from 'crypto';
import { Hook0ClientError } from './index';

/**
 * Signature class to parse and verify signatures
 */
class Signature {
  static PAYLOAD_SEPARATOR = '.';
  static PAYLOAD_SEPARATOR_BYTES = Buffer.from(Signature.PAYLOAD_SEPARATOR);
  static SIGNATURE_PART_ASSIGNATOR = '=';
  static SIGNATURE_PART_SEPARATOR = ',';
  static SIGNATURE_PART_HEADER_NAMES_SEPARATOR = ' ';

  timestamp: number;
  v0: Buffer | null;
  h: string[];
  v1: Buffer | null;

  /**
   * Constructor for the Signature class
   * @param timestamp - The timestamp of the signature
   * @param v0 - The version 0 hex-encoded signature
   * @param h - Name of the headers included in the signature
   * @param v1 - The version 1 hex-encoded signature
   */
  constructor(timestamp: number, v0: Buffer | null, h: string[], v1: Buffer | null) {
    this.timestamp = timestamp;
    this.v0 = v0;
    this.h = h;
    this.v1 = v1;
  }

  /**
   * Parse a signature string into a Signature object
   * @param signature - Signature string to parse
   * @returns A Signature instance
   * @throws Hook0ClientError if parsing fails
   */
  static parse(signature: string): Signature {
    const parts = new Map(
      signature.split(Signature.SIGNATURE_PART_SEPARATOR).map((part) => {
        const terms = part.split(Signature.SIGNATURE_PART_ASSIGNATOR, 2).map((term) => term.trim());
        return [terms[0], terms[1]] as [string, string];
      })
    );

    const tStr = parts.get('t');
    if (typeof tStr !== 'string') {
      throw Hook0ClientError.SignatureParsing(signature);
    }
    const t = parseInt(tStr, 10);
    if (isNaN(t)) {
      throw Hook0ClientError.TimestampParsingInSignature(tStr);
    }

    const v0Str = parts.get('v0') ?? null;
    const v0 = typeof v0Str === 'string' ? (Buffer.from(v0Str, 'hex') ?? null) : null;

    const hStr = (parts.get('h') ?? '').trim().toLowerCase();
    const h = hStr.length > 0 ? hStr.split(Signature.SIGNATURE_PART_HEADER_NAMES_SEPARATOR) : [];

    const v1Str = parts.get('v1') ?? null;
    const v1 = typeof v1Str === 'string' ? (Buffer.from(v1Str, 'hex') ?? null) : null;

    if (typeof v0 !== 'object' && typeof v1 !== 'object') {
      throw Hook0ClientError.SignatureParsing(signature);
    }

    return new Signature(t, v0, h, v1);
  }

  /**
   * Verify the signature against a payload and secret (HMAC)
   * @param payload - The payload to verify the signature against
   * @param secret - The secret key used to generate the HMAC signature
   * @returns true if the signature is valid, false otherwise
   */
  verify(payload: Buffer, headers: Headers, secret: string): boolean {
    const timestampStr = this.timestamp.toString();

    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(Buffer.from(timestampStr));
    hmac.update(Signature.PAYLOAD_SEPARATOR_BYTES);

    if (this.v1 !== null) {
      const header_names = this.h.join(Signature.SIGNATURE_PART_HEADER_NAMES_SEPARATOR);
      const header_values = this.h
        .map((name) => headers.get(name))
        .filter((v) => typeof v === 'string')
        .join(Signature.PAYLOAD_SEPARATOR);

      hmac.update(Buffer.from(header_names));
      hmac.update(Signature.PAYLOAD_SEPARATOR_BYTES);
      hmac.update(Buffer.from(header_values));
      hmac.update(Signature.PAYLOAD_SEPARATOR_BYTES);
      hmac.update(Buffer.from(payload));

      const expectedSignature = hmac.digest();
      const actualSignature = this.v1;

      return (
        expectedSignature.length === actualSignature.length &&
        crypto.timingSafeEqual(expectedSignature, actualSignature)
      );
    } else if (this.v0 !== null) {
      hmac.update(Buffer.from(payload));

      const expectedSignature = hmac.digest();
      const actualSignature = this.v0;

      return (
        expectedSignature.length === actualSignature.length &&
        crypto.timingSafeEqual(expectedSignature, actualSignature)
      );
    } else {
      // This cannot happen because this error would be raised while parsing the signature
      console.error('Failed to decode signature: no v0 nor v1 field');
      return false;
    }
  }
}

export { Signature };
