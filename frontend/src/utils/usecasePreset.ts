// Onboarding use-case personalization.
//
// The tutorial opens with a single, skippable "what are you building?" question.
// The answer pre-fills a realistic example event type + payload in the later steps,
// so the first webhook a user sends looks like their own domain instead of a
// generic `{"test": true}`. Kept free of Vue/i18n imports so it stays unit-testable
// under ts-jest (node env) — see usecasePreset.test.ts.

export type UseCaseId = 'saas-b2b' | 'ecommerce' | 'microservices' | 'other';

// `other` (and the skipped/default state) intentionally maps to no preset: the
// existing generic defaults are kept untouched.
export type PersonalizableUseCaseId = Exclude<UseCaseId, 'other'>;

export type EventTypePreset = {
  service: string;
  resourceType: string;
  verb: string;
};

export type EventLabel = {
  key: string;
  value: string;
};

export type UseCasePreset = {
  eventType: EventTypePreset;
  labels: EventLabel[];
  payload: string;
};

// Options rendered in the intro. A fixed business enumeration (not a dynamically
// discoverable set), paired with its i18n label key so the component never
// hardcodes the labels themselves.
export const USE_CASE_OPTIONS: ReadonlyArray<{ id: UseCaseId; labelKey: string }> = [
  { id: 'saas-b2b', labelKey: 'tutorial.intro.useCaseSaasB2b' },
  { id: 'ecommerce', labelKey: 'tutorial.intro.useCaseEcommerce' },
  { id: 'microservices', labelKey: 'tutorial.intro.useCaseMicroservices' },
  { id: 'other', labelKey: 'tutorial.intro.useCaseOther' },
];

function pretty(payload: Record<string, unknown>): string {
  return JSON.stringify(payload, null, 2);
}

const PRESETS: Record<PersonalizableUseCaseId, UseCasePreset> = {
  'saas-b2b': {
    eventType: { service: 'iam', resourceType: 'user', verb: 'updated' },
    labels: [{ key: 'tenant_id', value: 'acme-inc' }],
    payload: pretty({
      user_id: 'usr_1a2b3c',
      email: 'jane@acme.com',
      role: 'admin',
      plan: 'business',
    }),
  },
  ecommerce: {
    eventType: { service: 'store', resourceType: 'order', verb: 'created' },
    labels: [{ key: 'customer_id', value: 'cus_42' }],
    payload: pretty({
      order_id: 'ord_10245',
      currency: 'EUR',
      amount: 8990,
      items: 3,
      status: 'paid',
    }),
  },
  microservices: {
    eventType: { service: 'billing', resourceType: 'invoice', verb: 'paid' },
    labels: [{ key: 'service', value: 'billing' }],
    payload: pretty({
      invoice_id: 'inv_2024_0091',
      account_id: 'acc_7781',
      amount_due: 12000,
      currency: 'USD',
    }),
  },
};

// Returns the preset for a personalizable use-case, or `undefined` for `other`
// (and any unknown id) so callers keep their existing generic defaults.
export function getUseCasePreset(useCase: UseCaseId): UseCasePreset | undefined {
  return useCase === 'other' ? undefined : PRESETS[useCase];
}

// Joins an event type preset into the canonical `service.resourceType.verb` name.
export function formatEventTypeName(eventType: EventTypePreset): string {
  return `${eventType.service}.${eventType.resourceType}.${eventType.verb}`;
}
