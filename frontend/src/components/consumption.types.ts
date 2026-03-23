import type { Component } from 'vue';

export type ConsumptionQuota = {
  icon?: Component;
  name: string;
  description?: string;
  consumption: number;
  quota: number;
  /** Override display (e.g. "7" for retention) */
  displayValue?: string;
  /** Unit suffix (e.g. "days") shown after displayValue */
  displayUnit?: string;
};
