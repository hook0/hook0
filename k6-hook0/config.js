// options for k6
const VUS = 1; // Number of virtual users
const ITERATIONS = 1; // Number of iterations
const MAX_DURATION= '1m'; // Duration before timeout the job

const DEFAULT_HOSTNAME = 'http://localhost:8081/'; // Hostname for the application
const DEFAULT_TARGET_URL = 'https://webhook.site/2af5e206-b9fd-4f83-8e96-202a80c862b1'; // Target URL for the application

const DEFAULT_MASTER_API_KEY = '10e27acc-662e-48d2-bee3-38a4fa956449';

export function getEnvironmentVariables() {
    const vus = __ENV.VUS || VUS;
    const iterations = __ENV.ITERATIONS || ITERATIONS;
    const maxDuration = __ENV.MAX_DURATION || MAX_DURATION;

    const hostname = __ENV.HOSTNAME || DEFAULT_HOSTNAME;
    const targetUrl = __ENV.TARGET_URL || DEFAULT_TARGET_URL;
    const masterApiKey = __ENV.MASTER_API_KEY || DEFAULT_MASTER_API_KEY;


    return {
        vus,
        iterations,
        maxDuration,
        hostname,
        targetUrl,
        masterApiKey
    };
}