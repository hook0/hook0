import * as crypto from 'crypto';
import { Hook0ClientError } from './index';

/**
 * Decode a hex-encoded signature field, refusing anything that is not entirely valid hex.
 *
 * `Buffer.from(value, 'hex')` stops at the first character it cannot decode and returns what it
 * managed to read, so a malformed field would otherwise become a shorter, plausible-looking
 * signature instead of an error. Every byte accounts for exactly two input characters, so a
 * decoded length that does not cover the whole input means the input was not valid hex.
 * @param value - The hex-encoded value
 * @param signature - The whole signature header, reported when the value cannot be decoded
 * @returns The decoded bytes
 * @throws Hook0ClientError if the value is not entirely valid hex
 */
function decodeHexField(value: string, signature: string): Buffer {
  const decoded = Buffer.from(value, 'hex');
  if (decoded.length * 2 !== value.length) {
    throw Hook0ClientError.SignatureParsing(signature);
  }
  return decoded;
}

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
    const parts = new Map<string, string>();
    for (const part of signature.split(Signature.SIGNATURE_PART_SEPARATOR)) {
      // Only the first assignator separates the key from the value; the value keeps everything
      // that follows so that a stray `=` inside it makes the field invalid rather than truncating
      // it silently.
      const assignatorIndex = part.indexOf(Signature.SIGNATURE_PART_ASSIGNATOR);
      if (assignatorIndex >= 0) {
        parts.set(part.slice(0, assignatorIndex).trim(), part.slice(assignatorIndex + 1).trim());
      }
    }

    const tStr = parts.get('t');
    if (typeof tStr !== 'string') {
      throw Hook0ClientError.SignatureParsing(signature);
    }
    const t = parseInt(tStr, 10);
    if (isNaN(t)) {
      throw Hook0ClientError.TimestampParsingInSignature(tStr);
    }

    const v0Str = parts.get('v0');
    const v0 = typeof v0Str === 'string' ? decodeHexField(v0Str, signature) : null;

    const hStr = parts.get('h');
    const h =
      typeof hStr === 'string' && hStr.length > 0
        ? hStr.toLowerCase().split(Signature.SIGNATURE_PART_HEADER_NAMES_SEPARATOR)
        : [];

    const v1Str = parts.get('v1');
    const v1 = typeof v1Str === 'string' ? decodeHexField(v1Str, signature) : null;

    if (v0 === null && v1 === null) {
      throw Hook0ClientError.SignatureParsing(signature);
    }

    return new Signature(t, v0, h, v1);
  }

  /**
   * Verify the signature against a payload and secret (HMAC)
   * @param payload - The payload to verify the signature against
   * @param secret - The secret key used to generate the HMAC signature
   * @returns true if the signature is valid, false otherwise
   * @throws Hook0ClientError if a header the signature covers is absent from the request
   */
  verify(payload: Buffer, headers: Headers, secret: string): boolean {
    // Resolved before any HMAC work, so that a header the signature covers but the request does
    // not carry is reported as such instead of quietly shortening the signed string and surfacing
    // as an invalid signature.
    const resolvedHeaderValues = this.h.map((name) => {
      const value = headers.get(name);
      if (typeof value !== 'string') {
        throw Hook0ClientError.MissingHeader(name);
      }
      return value;
    });

    const timestampStr = this.timestamp.toString();

    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(Buffer.from(timestampStr));
    hmac.update(Signature.PAYLOAD_SEPARATOR_BYTES);

    if (this.v1 !== null) {
      const header_names = this.h.join(Signature.SIGNATURE_PART_HEADER_NAMES_SEPARATOR);
      const header_values = resolvedHeaderValues.join(Signature.PAYLOAD_SEPARATOR);

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
