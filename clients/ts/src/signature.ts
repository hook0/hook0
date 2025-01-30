import crypto from 'crypto';
import { Hook0ClientError } from './lib';

class Signature {
  static PAYLOAD_SEPARATOR = Buffer.from('.');

  timestamp: number;
  v0: string;

  constructor(timestamp: number, v0: string) {
    this.timestamp = timestamp;
    this.v0 = v0;
  }

  static parse(signature: string): Signature {
    const match = signature.match(/^t=(\d+),v0=([a-f0-9]+)$/i);
    if (match) {
      const [, timestamp, v0] = match;
      const parsedTimestamp = parseInt(timestamp, 10);
      if (isNaN(parsedTimestamp)) {
        throw Hook0ClientError.TimestampParsingInSignature(new Error(timestamp));
      }
      return new Signature(parsedTimestamp, v0);
    }
    throw Hook0ClientError.SignatureParsing(signature);
  }

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

// Tests
function testParsingSuccessfulSignature() {
  const signature = Signature.parse('t=123,v0=abc');
  console.assert(signature.timestamp === 123, 'Timestamp should be 123');
  console.assert(signature.v0 === 'abc', "v0 should be 'abc'");
}

function testParsingInvalidSignature() {
  try {
    Signature.parse('t=error,v0=def');
    console.assert(false, 'Parsing should have failed');
  } catch (error) {
    console.assert(error instanceof Hook0ClientError, 'Error should be Hook0ClientError');
  }
}

function testVerificationSuccessful() {
  const signature = new Signature(
    1636936200,
    '1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
  );
  const payload = Buffer.from('hello !');
  const secret = 'secret';
  console.assert(signature.verify(payload, secret), 'Verification should succeed');
}

function testVerificationFailed() {
  const signature = new Signature(
    1636936200,
    '1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
  );
  const payload = Buffer.from('hello !');
  const secret = 'another secret';
  console.assert(!signature.verify(payload, secret), 'Verification should fail');
}

testParsingSuccessfulSignature();
testParsingInvalidSignature();
testVerificationSuccessful();
testVerificationFailed();

console.log('All tests passed!');
