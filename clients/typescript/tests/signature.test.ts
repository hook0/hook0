import { describe, expect, test } from '@jest/globals';

import {
  Hook0ClientError,
  Signature,
  verifyWebhookSignature,
  verifyWebhookSignatureWithCurrentTime,
} from '../src/index';

describe('Signature', () => {
  test('should successfully parse a valid v0 signature', () => {
    const signature = Signature.parse('t=123,v0=abcd');
    expect(signature).toBeInstanceOf(Signature);
    expect(signature.timestamp).toStrictEqual(123);
    expect(signature.v0?.toString('hex')).toStrictEqual('abcd');
    expect(signature.h).toStrictEqual([]);
    expect(signature.v1).toStrictEqual(null);
  });

  test('should fail to parse a signature with invalid timestamp', () => {
    expect(() => Signature.parse('t=error,v0=defg')).toThrow(
      Hook0ClientError.TimestampParsingInSignature('error')
    );
  });

  test('should successfully parse a valid v1 signature', () => {
    const signature = Signature.parse('t=123,h=test1 test2,v1=abcd');
    expect(signature).toBeInstanceOf(Signature);
    expect(signature.timestamp).toStrictEqual(123);
    expect(signature.v0).toStrictEqual(null);
    expect(signature.h).toStrictEqual(['test1', 'test2']);
    expect(signature.v1?.toString('hex')).toStrictEqual('abcd');
  });

  test('should successfully parse a valid v0 and v1 signature', () => {
    const signature = Signature.parse('t=123,v0=4567,h=test1 test2,v1=abcd');
    expect(signature).toBeInstanceOf(Signature);
    expect(signature.timestamp).toStrictEqual(123);
    expect(signature.v0?.toString('hex')).toStrictEqual('4567');
    expect(signature.h).toStrictEqual(['test1', 'test2']);
    expect(signature.v1?.toString('hex')).toStrictEqual('abcd');
  });

  test('should verify a valid v0 signature', () => {
    const signature = new Signature(
      1636936200,
      Buffer.from('1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98', 'hex'),
      [],
      null
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(signature.verify(payload, new Headers(), secret)).toStrictEqual(true);
  });

  test('should fail to verify an invalid v0 signature', () => {
    const signature = new Signature(
      1636936200,
      Buffer.from('1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98', 'hex'),
      [],
      null
    );
    const payload = Buffer.from('hello !');
    const secret = 'another secret';

    expect(signature.verify(payload, new Headers(), secret)).toStrictEqual(false);
  });

  test('should parse and verify a valid v0 signature', () => {
    const signature = Signature.parse(
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(signature.verify(payload, new Headers(), secret)).toStrictEqual(true);
  });

  test('should parse a valid v0 signature but fail verification with incorrect secret', () => {
    const signature = Signature.parse(
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'another secret';

    expect(signature.verify(payload, new Headers(), secret)).toStrictEqual(false);
  });

  test('should verify a valid v0 signature with current time', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    const currentTime = new Date(1636936200 * 1000);

    expect(
      verifyWebhookSignatureWithCurrentTime(
        signature,
        payload,
        new Headers(),
        secret,
        300,
        currentTime
      )
    ).toStrictEqual(true);
  });

  test('should fail to verify a v0 signature with an expired timestamp', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    const currentTime = new Date(1636936501 * 1000);

    expect(() =>
      verifyWebhookSignatureWithCurrentTime(
        signature,
        payload,
        new Headers(),
        secret,
        300,
        currentTime
      )
    ).toThrow();
  });

  test('should fail to verify a v0 signature without a timestamp', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(() => verifyWebhookSignature(signature, payload, new Headers(), secret, 300)).toThrow();
  });

  test('should refuse a signature header carrying no signature at all', () => {
    for (const header of ['t=123', 't=123,h=x-test', 't=123,foo=bar']) {
      expect(() => Signature.parse(header)).toThrow(Hook0ClientError.SignatureParsing(header));
    }
  });

  test('should refuse a signature whose hex is not decodable', () => {
    // `zz` is not hex at all, `abcz` and `abc` would decode to a shorter value than announced,
    // and `ab=cd` is what an extra assignator inside a value looks like.
    for (const header of [
      't=123,v0=zz',
      't=123,v0=abcz',
      't=123,v0=abc',
      't=123,v0=ab=cd',
      't=123,h=x-test,v1=zz',
      't=123,h=x-test,v1=abcz',
    ]) {
      expect(() => Signature.parse(header)).toThrow(Hook0ClientError.SignatureParsing(header));
    }
  });

  test('should report a header named in the signature but absent from the request as missing', () => {
    const signature = new Signature(
      1636936200,
      null,
      ['x-test', 'x-test2'],
      Buffer.from('493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a', 'hex')
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(() => signature.verify(payload, new Headers([['x-test', 'val1']]), secret)).toThrow(
      Hook0ClientError.MissingHeader('x-test2')
    );
  });

  test('should require a header named in the signature even when only v0 is signed', () => {
    const signature = Signature.parse(
      't=1636936200,h=x-test,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(() => signature.verify(payload, new Headers(), secret)).toThrow(
      Hook0ClientError.MissingHeader('x-test')
    );
  });

  test('should refuse a timestamp later than the tolerance', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    // The webhook claims to have been signed one hour after the moment it is received.
    const currentTime = new Date((1636936200 - 3600) * 1000);

    expect(() =>
      verifyWebhookSignatureWithCurrentTime(
        signature,
        payload,
        new Headers(),
        secret,
        300,
        currentTime
      )
    ).toThrow(Hook0ClientError.ExpiredWebhook(new Date(1636936200 * 1000), 300, currentTime));
  });

  test('should accept a timestamp slightly ahead but within the tolerance', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    // A minute of clock drift between the producer and the consumer.
    const currentTime = new Date((1636936200 - 60) * 1000);

    expect(
      verifyWebhookSignatureWithCurrentTime(
        signature,
        payload,
        new Headers(),
        secret,
        300,
        currentTime
      )
    ).toStrictEqual(true);
  });

  test('should report the moment a webhook outside the tolerance was actually signed', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    const currentTime = new Date((1636936200 + 3600) * 1000);

    expect(() =>
      verifyWebhookSignatureWithCurrentTime(
        signature,
        payload,
        new Headers(),
        secret,
        300,
        currentTime
      )
    ).toThrow(Hook0ClientError.ExpiredWebhook(new Date(1636936200 * 1000), 300, currentTime));
  });

  test('should verify a valid v1 signature', () => {
    const signature = new Signature(
      1636936200,
      null,
      ['x-test', 'x-test2'],
      Buffer.from('493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a', 'hex')
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(
      signature.verify(
        payload,
        new Headers([
          ['x-test', 'val1'],
          ['x-test2', 'val2'],
        ]),
        secret
      )
    ).toStrictEqual(true);

    // Uppercase in header names should not change signature
    expect(
      signature.verify(
        payload,
        new Headers([
          ['X-Test', 'val1'],
          ['X-TEST2', 'val2'],
        ]),
        secret
      )
    ).toStrictEqual(true);
  });
});
