import { z } from 'zod';

export const sendEventSchema = z.object({
  eventType: z.string().min(1, 'Event type is required'),
  labels: z.record(z.string(), z.string()).refine((val) => Object.keys(val).length > 0, {
    message: 'At least one label is required',
  }),
  occurredAt: z.coerce.date({ message: 'Occurred at date is required' }),
  payload: z
    .string()
    .min(1, 'Payload is required')
    .refine(
      (val) => {
        try {
          JSON.parse(val);
          return true;
        } catch {
          return false;
        }
      },
      { message: 'Payload must be valid JSON' }
    ),
});

export type SendEventFormValues = z.infer<typeof sendEventSchema>;
