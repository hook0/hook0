export function isPricingEnabled(): boolean {
  return (process.env.VUE_APP_ENABLE_QUOTA_ENFORCEMENT ?? 'false').toLowerCase() === 'true';
}
