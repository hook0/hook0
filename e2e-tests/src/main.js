import { getEnvironmentVariables } from './config.js';
import { scenario_1 } from './scenarios/scenario1.js';
import { scenario_2 } from './scenarios/scenario2.js';

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
  scenario_1(config);
  scenario_2(config);
}
