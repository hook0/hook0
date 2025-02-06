import { getEnvironmentVariables } from './config.js';
import { basic_usage } from './scenarios/basic_usage.js';
import { dashboard_api } from './scenarios/dashboard_api.js';

const config = getEnvironmentVariables();

export const options = {
  vus: config.vus,
  iterations: config.iterations,
  duration: config.maxDuration,
  thresholds: {
    // the rate of successful checks should be at 100%
    checks: ['rate>=1.0'],
  },
};

export default function () {
  basic_usage(config);
  dashboard_api(config);
}
