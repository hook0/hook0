import * as crypto from 'crypto';
import { Hook0ClientError } from './lib';

/**
 * Signature class to parse and verify signatures
 */
class Signature {
  // Separator used between different parts of the payload
  static PAYLOAD_SEPARATOR = Buffer.from('.');

  timestamp: number;
  v0: string;

  /**
   * Constructor for the Signature class
   * @param timestamp - The timestamp of the signature
   * @param v0 - The version 0 hash of the signature
   */
  constructor(timestamp: number, v0: string) {
    this.timestamp = timestamp;
    this.v0 = v0;
  }

  /**
   * Parse a signature string into a Signature object
   * @param signature - Signature string to parse
   * @returns A Signature instance
   * @throws Hook0ClientError if parsing fails
   */
  static parse(signature: string): Signature {
    const match = signature.match(/^t=(\d+),v0=([a-f0-9]+)$/i);
    if (match) {
      const [, timestamp, v0] = match;
      const parsedTimestamp = parseInt(timestamp, 10);
      if (isNaN(parsedTimestamp)) {
        throw Hook0ClientError.TimestampParsingInSignature(timestamp);
      }
      return new Signature(parsedTimestamp, v0);
    }
    throw Hook0ClientError.SignatureParsing(signature);
  }

  /**
   * Verify the signature against a payload and secret
   * @param payload - The payload to verify the signature against
   * @param secret - The secret used to generate the signature
   * @returns true if the signature is valid, false otherwise
   */
  verify(payload: Buffer, secret: string): boolean {
    const timestampStr = this.timestamp.toString();
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(Buffer.from(timestampStr));
    hmac.update(Signature.PAYLOAD_SEPARATOR);
    hmac.update(payload);

    const expectedSignature = hmac.digest('hex');
    return expectedSignature === this.v0;
  }
}

export default Signature;
