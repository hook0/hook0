import { getInstanceConfig } from './utils/biscuit_auth';

export async function isPricingEnabled(): Promise<boolean> {
  const config = await getInstanceConfig();
  return config.quota_enforcement;
}

export async function getSupportEmailAddress(): Promise<string> {
  const config = await getInstanceConfig();
  return config.support_email_address;
}
