import { z } from 'zod';

export const subscriptionSchema = z.object({
  description: z.string().min(1, 'Subscription description is required'),
  target: z.object({
    method: z.string().min(1, 'HTTP method is required'),
    url: z.string().url('A valid URL is required'),
  }),
});

export type SubscriptionFormValues = z.infer<typeof subscriptionSchema>;
