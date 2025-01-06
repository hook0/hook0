// options for k6
const VUS = 1; // Number of virtual users
const ITERATIONS = 1; // Number of iterations
const MAX_DURATION= '1m'; // Duration before timeout the job

export function getEnvironmentVariables() {
    const vus = __ENV.VUS || VUS;
    const iterations = __ENV.ITERATIONS || ITERATIONS;
    const maxDuration = __ENV.MAX_DURATION || MAX_DURATION;

    const hostname = __ENV.HOSTNAME ? __ENV.HOSTNAME : null;
    const targetUrl = __ENV.TARGET_URL ? __ENV.TARGET_URL : null;
    const serviceToken = __ENV.SERVICE_TOKEN ? __ENV.SERVICE_TOKEN : null;
    const organizationId = __ENV.ORGANIZATION_ID ? __ENV.ORGANIZATION_ID : null;

    if (!hostname || !targetUrl || !serviceToken || !organizationId) {
        console.log(hostname);
        console.log(targetUrl);
        console.log(serviceToken);
        console.log(organizationId);
        throw new Error('Missing environment variables HOSTNAME, TARGET_URL, SERVICE_TOKEN, ORGANIZATION_ID');
    }


    return {
        vus,
        iterations,
        maxDuration,
        hostname,
        targetUrl,
        serviceToken,
        organizationId,
    };
}
