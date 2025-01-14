// options for k6
const DEFAULTS = {
  vus: 1, // Number of virtual users
  iterations: 1, // Number of iterations
  maxDuration: '1m', // Duration before timeout the job
  keepTestApplication: false, // Delete the application after the test
};

export function getEnvironmentVariables() {
  const vus = __ENV.VUS || DEFAULTS.vus;
  const iterations = __ENV.ITERATIONS || DEFAULTS.iterations;
  const maxDuration = __ENV.MAX_DURATION || DEFAULTS.maxDuration;
  const keepTestApplication = __ENV.KEEP_TEST_APPLICATION
    ? __ENV.KEEP_TEST_APPLICATION === "true"
    : DEFAULTS.keepTestApplication;

  const apiOrigin = __ENV.API_ORIGIN ? __ENV.API_ORIGIN : null;
  const targetUrl = __ENV.TARGET_URL ? __ENV.TARGET_URL : null;
  const serviceToken = __ENV.SERVICE_TOKEN ? __ENV.SERVICE_TOKEN : null;
  const organizationId = __ENV.ORGANIZATION_ID ? __ENV.ORGANIZATION_ID : null;

  if (!apiOrigin || !targetUrl || !serviceToken || !organizationId) {
    throw new Error(
      'Missing environment variables API_ORIGIN, TARGET_URL, SERVICE_TOKEN, ORGANIZATION_ID'
    );
  }

  return {
    vus,
    iterations,
    maxDuration,
    apiOrigin: apiOrigin.endsWith('/') ? apiOrigin : `${apiOrigin}/`,
    targetUrl,
    serviceToken,
    organizationId,
    keepTestApplication,
  };
}
