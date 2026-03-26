import { z } from 'zod';

export const sendEventSchema = z.object({
  eventType: z.string().min(1, 'Event type is required'),
  labels: z
    .array(
      z.object({
        key: z.string().min(1, 'Label key is required'),
        value: z.string().min(1, 'Label value is required'),
      })
    )
    .min(1, 'At least one label is required'),
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

export type SendEventForm = z.infer<typeof sendEventSchema>;
export type SendEventFormValues = z.infer<typeof sendEventSchema>;
