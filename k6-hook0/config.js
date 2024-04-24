// options for k6
const VUS = 1; // Number of virtual users
const ITERATIONS = 1; // Number of iterations
const MAX_DURATION= '1m'; // Duration before timeout the job

const DEFAULT_HOSTNAME = 'http://localhost:8081/'; // Hostname for the application
const DEFAULT_TARGET_URL = 'https://webhook.site/2af5e206-b9fd-4f83-8e96-202a80c862b1'; // Target URL for the application

const DEFAULT_AUTH_TOKEN = 'Bearer 08f34020-566f-40fe-bb78-94b68cd24540'; // Auth token for the application
const DEFAULT_MASTER_API_KEY = '10e27acc-662e-48d2-bee3-38a4fa956449';

const TIME_BEFORE_EACH_REQUEST = 1; // Time in seconds before each request per VU
const TIME_BEFORE_EACH_VERIFICATION = 1; // Time in seconds before each verification request per virtual users
const TIME_BEFORE_EACH_DELETE = 1; // Time in seconds before each delete request per virtual users

const RETRY_COUNT = 3; // Number of retries before giving up

export function getEnvironmentVariables() {
    const vus = __ENV.VUS || VUS;
    const iterations = __ENV.ITERATIONS || ITERATIONS;
    const maxDuration = __ENV.MAX_DURATION || MAX_DURATION;

    const hostname = __ENV.HOSTNAME || DEFAULT_HOSTNAME;
    const targetUrl = __ENV.TARGET_URL || DEFAULT_TARGET_URL;
    const authToken = __ENV.AUTH_TOKEN || DEFAULT_AUTH_TOKEN;
    const masterApiKey = __ENV.MASTER_API_KEY || DEFAULT_MASTER_API_KEY;

    const timeBeforeEachRequest = __ENV.TIME_BEFORE_EACH_REQUEST || TIME_BEFORE_EACH_REQUEST;
    const timeBeforeEachVerification = __ENV.TIME_BEFORE_EACH_VERIFICATION || TIME_BEFORE_EACH_VERIFICATION;
    const timeBeforeEachDelete = __ENV.TIME_BEFORE_EACH_DELETE || TIME_BEFORE_EACH_DELETE;

    const retryCount = __ENV.RETRY_COUNT || RETRY_COUNT;


    return {
        vus,
        iterations,
        maxDuration,
        hostname,
        targetUrl,
        authToken,
        masterApiKey,
        timeBeforeEachRequest,
        timeBeforeEachVerification,
        timeBeforeEachDelete,
        retryCount
    };
}