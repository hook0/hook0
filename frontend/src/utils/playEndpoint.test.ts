import {
  generatePlayToken,
  isValidPlayToken,
  normalizePlayBaseUrl,
  buildPlayReceiveUrl,
  buildPlayViewUrl,
  buildPlayInspectUrl,
} from './playEndpoint';

describe('playEndpoint', () => {
  describe('generatePlayToken', () => {
    it('produces a server-valid token (c_ + 27 chars)', () => {
      const token = generatePlayToken();
      expect(token.startsWith('c_')).toBe(true);
      expect(token).toHaveLength(29);
      expect(isValidPlayToken(token)).toBe(true);
    });

    it('only uses base62 characters', () => {
      expect(generatePlayToken().slice(2)).toMatch(/^[0-9A-Za-z]{27}$/);
    });

    it('is unique across many calls', () => {
      const tokens = Array.from({ length: 200 }, () => generatePlayToken());
      expect(new Set(tokens).size).toBe(tokens.length);
    });
  });

  describe('isValidPlayToken', () => {
    it('rejects a missing prefix', () => {
      expect(isValidPlayToken('x_123456789012345678901234567')).toBe(false);
    });

    it('rejects a wrong length', () => {
      expect(isValidPlayToken('c_tooshort')).toBe(false);
    });

    it('rejects invalid characters', () => {
      expect(isValidPlayToken('c_12345678901234567890123456!')).toBe(false);
    });

    it('accepts a freshly generated token', () => {
      expect(isValidPlayToken(generatePlayToken())).toBe(true);
    });
  });

  describe('url builders', () => {
    it('normalizes trailing slashes', () => {
      expect(normalizePlayBaseUrl('https://play.hook0.com/')).toBe('https://play.hook0.com');
      expect(normalizePlayBaseUrl('https://play.hook0.com///')).toBe('https://play.hook0.com');
      expect(normalizePlayBaseUrl('https://play.hook0.com')).toBe('https://play.hook0.com');
    });

    it('builds the receive URL', () => {
      expect(buildPlayReceiveUrl('https://play.hook0.com/', 'c_abc')).toBe(
        'https://play.hook0.com/in/c_abc/'
      );
    });

    it('builds the view URL pointing at the inspector page fragment', () => {
      expect(buildPlayViewUrl('https://play.hook0.com', 'c_abc')).toBe(
        'https://play.hook0.com/#c_abc'
      );
      expect(buildPlayViewUrl('https://play.hook0.com/', 'c_abc')).toBe(
        'https://play.hook0.com/#c_abc'
      );
    });

    it('builds the inspect API URL', () => {
      expect(buildPlayInspectUrl('https://play.hook0.com', 'c_abc')).toBe(
        'https://play.hook0.com/api/tokens/c_abc/webhooks'
      );
    });
  });
});
