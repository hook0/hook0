import { z } from 'zod';
import i18n from '@/plugins/i18n';

// Mirrors the server-side allow-list (`crate::validators::event_type_segment`).
// The three segments are concatenated into a dot-delimited name that gets
// interpolated into a URL path, so restricting them to safe characters here is
// immediate feedback for the same rule the API enforces authoritatively.
const EVENT_TYPE_SEGMENT_PATTERN = /^[A-Za-z0-9_-]+$/;
const EVENT_TYPE_SEGMENT_MAX_LENGTH = 50;

export function createEventTypeSchema() {
  const t = i18n.global.t;
  const segment = (field: string) =>
    z
      .string()
      .min(1, t('validation.required', { field }))
      .max(EVENT_TYPE_SEGMENT_MAX_LENGTH, t('validation.eventTypeSegment', { field }))
      .regex(EVENT_TYPE_SEGMENT_PATTERN, t('validation.eventTypeSegment', { field }));
  return z.object({
    service: segment(t('fields.service')),
    resource_type: segment(t('fields.resourceType')),
    verb: segment(t('fields.verb')),
  });
}

export type EventTypeFormValues = z.infer<ReturnType<typeof createEventTypeSchema>>;
