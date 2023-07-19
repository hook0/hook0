export function isPricingEnabled(): boolean {
  return (import.meta.env.VITE_ENABLE_QUOTA_ENFORCEMENT ?? 'false').toLowerCase() === 'true';
}
