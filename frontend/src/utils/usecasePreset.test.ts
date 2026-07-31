import {
  USE_CASE_OPTIONS,
  isUseCaseId,
  getUseCasePreset,
  formatEventTypeName,
  type UseCaseId,
} from './usecasePreset';

describe('usecasePreset', () => {
  describe('isUseCaseId', () => {
    it('accepts every advertised option id', () => {
      for (const option of USE_CASE_OPTIONS) {
        expect(isUseCaseId(option.id)).toBe(true);
      }
    });

    it('rejects an unknown id', () => {
      expect(isUseCaseId('crypto-mining')).toBe(false);
      expect(isUseCaseId('')).toBe(false);
    });
  });

  describe('USE_CASE_OPTIONS', () => {
    it('exposes a distinct, non-empty i18n label key per option', () => {
      const keys = USE_CASE_OPTIONS.map((o) => o.labelKey);
      expect(new Set(keys).size).toBe(keys.length);
      for (const key of keys) {
        expect(key.startsWith('tutorial.intro.useCase')).toBe(true);
      }
    });

    it('offers a non-personalized "other" escape hatch', () => {
      expect(USE_CASE_OPTIONS.some((o) => o.id === 'other')).toBe(true);
      expect(getUseCasePreset('other')).toBeUndefined();
    });
  });

  describe('getUseCasePreset', () => {
    it('maps e-commerce to an order.created example', () => {
      const preset = getUseCasePreset('ecommerce');
      expect(preset).toBeDefined();
      expect(preset && formatEventTypeName(preset.eventType)).toBe('store.order.created');
    });

    it('maps SaaS B2B to a user.updated example', () => {
      const preset = getUseCasePreset('saas-b2b');
      expect(preset && formatEventTypeName(preset.eventType)).toBe('iam.user.updated');
    });

    it('maps internal microservices to an invoice.paid example', () => {
      const preset = getUseCasePreset('microservices');
      expect(preset && formatEventTypeName(preset.eventType)).toBe('billing.invoice.paid');
    });

    it('produces a valid, non-empty JSON payload and at least one label for each preset', () => {
      const personalized: UseCaseId[] = ['saas-b2b', 'ecommerce', 'microservices'];
      for (const id of personalized) {
        const preset = getUseCasePreset(id);
        expect(preset).toBeDefined();
        if (!preset) continue;
        expect(preset.labels.length).toBeGreaterThan(0);
        for (const label of preset.labels) {
          expect(label.key.length).toBeGreaterThan(0);
          expect(label.value.length).toBeGreaterThan(0);
        }
        expect(() => {
          JSON.parse(preset.payload);
        }).not.toThrow();
        const parsed = JSON.parse(preset.payload) as Record<string, unknown>;
        expect(Object.keys(parsed).length).toBeGreaterThan(0);
      }
    });
  });

  describe('formatEventTypeName', () => {
    it('joins the three segments with dots', () => {
      expect(formatEventTypeName({ service: 'a', resourceType: 'b', verb: 'c' })).toBe('a.b.c');
    });
  });
});
