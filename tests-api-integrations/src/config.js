// options for k6
const DEFAULTS = {
  vus: 1, // Number of virtual users
  iterations: 1, // Number of iterations
  maxDuration: '1m', // Duration before timeout the job
  keepTestApplication: false, // Delete the application after the test
};

// Generate a random UUID v4
function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

// Generate random test user credentials
function generateTestUserDefaults() {
  const uuid1 = generateUUID();
  const uuid2 = generateUUID();
  return {
    testUserEmail: `e2e-test-${uuid1}@hook0.test`,
    testUserPassword: `TestPass!${uuid1}`,
    testUserEmail2: `e2e-test-${uuid2}@hook0.test`,
    testUserPassword2: `TestPass!${uuid2}`,
  };
}

const testUserDefaults = generateTestUserDefaults();

export function getEnvironmentVariables() {
  const vus = __ENV.VUS || DEFAULTS.vus;
  const iterations = __ENV.ITERATIONS || DEFAULTS.iterations;
  const maxDuration = __ENV.MAX_DURATION || DEFAULTS.maxDuration;
  const keepTestApplication = __ENV.KEEP_TEST_APPLICATION
    ? __ENV.KEEP_TEST_APPLICATION === 'true'
    : DEFAULTS.keepTestApplication;

  const apiOrigin = __ENV.API_ORIGIN ? __ENV.API_ORIGIN : null;
  const targetUrl = __ENV.TARGET_URL ? __ENV.TARGET_URL : null;
  const serviceToken = __ENV.SERVICE_TOKEN ? __ENV.SERVICE_TOKEN : null;
  const organizationId = __ENV.ORGANIZATION_ID ? __ENV.ORGANIZATION_ID : null;

  // User credentials for account deletion tests (random defaults, can be overridden)
  const testUserEmail = __ENV.TEST_USER_EMAIL || testUserDefaults.testUserEmail;
  const testUserPassword = __ENV.TEST_USER_PASSWORD || testUserDefaults.testUserPassword;

  // Second user credentials for account isolation tests (random defaults, can be overridden)
  const testUserEmail2 = __ENV.TEST_USER_EMAIL_2 || testUserDefaults.testUserEmail2;
  const testUserPassword2 = __ENV.TEST_USER_PASSWORD_2 || testUserDefaults.testUserPassword2;

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
    testUserEmail,
    testUserPassword,
    testUserEmail2,
    testUserPassword2,
  };
}
