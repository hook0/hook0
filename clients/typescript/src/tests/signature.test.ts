import { describe, expect, test } from '@jest/globals';

import { Signature, verifyWebhookSignature, verifyWebhookSignatureWithCurrentTime } from '../index';

describe('Signature', () => {
  test('should successfully parse a valid signature', () => {
    const signature = Signature.parse('t=123,v0=abc');
    expect(signature).toBeInstanceOf(Signature);
    expect(signature.timestamp).toBe(123);
    expect(signature.v0).toBe('abc');
  });

  test('should fail to parse an invalid signature', () => {
    expect(() => Signature.parse('t=error,v0=def')).toThrow();
  });

  test('should verify a valid signature', () => {
    const signature = new Signature(
      1636936200,
      '1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(signature.verify(payload, secret)).toBe(true);
  });

  test('should fail to verify an incorrect signature', () => {
    const signature = new Signature(
      1636936200,
      '1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'another secret';

    expect(signature.verify(payload, secret)).toBe(false);
  });

  test('should parse and verify a valid signature', () => {
    const signature = Signature.parse(
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(signature.verify(payload, secret)).toBe(true);
  });

  test('should parse a valid signature but fail verification with incorrect secret', () => {
    const signature = Signature.parse(
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98'
    );
    const payload = Buffer.from('hello !');
    const secret = 'another secret';

    expect(signature.verify(payload, secret)).toBe(false);
  });

  test('should verify a valid signature with current time', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    const currentTime = new Date(1636936200 * 1000);

    expect(
      verifyWebhookSignatureWithCurrentTime(signature, payload, secret, 300, currentTime)
    ).toBe(true);
  });

  test('should fail to verify a signature with an expired timestamp', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';
    const currentTime = new Date(1636936501 * 1000);

    expect(() =>
      verifyWebhookSignatureWithCurrentTime(signature, payload, secret, 300, currentTime)
    ).toThrow();
  });

  test('should fail to verify a signature without a timestamp', () => {
    const signature =
      't=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98';
    const payload = Buffer.from('hello !');
    const secret = 'secret';

    expect(() => verifyWebhookSignature(signature, payload, secret, 300)).toThrow();
  });
});
