import { z } from 'zod';

export const eventTypeSchema = z.object({
  service: z.string().min(1, 'Service is required'),
  resource_type: z.string().min(1, 'Resource type is required'),
  verb: z.string().min(1, 'Verb is required'),
});

export type EventTypeFormValues = z.infer<typeof eventTypeSchema>;
