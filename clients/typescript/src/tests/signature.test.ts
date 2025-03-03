import { describe, expect, test } from '@jest/globals';

import {
  Hook0ClientError,
  Signature,
  verifyWebhookSignature,
  verifyWebhookSignatureWithCurrentTime,
} from '../index';

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
