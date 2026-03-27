import type { Component } from 'vue';
import type { RouteLocationRaw } from 'vue-router';

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
  /** If set, the row renders as a <router-link> to this route. */
  to?: RouteLocationRaw;
  /** If set, the row is clickable and calls this handler. */
  onClick?: () => void;
};
