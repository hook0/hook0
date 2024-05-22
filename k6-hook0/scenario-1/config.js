// options for k6
const VUS = 1; // Number of virtual users
const ITERATIONS = 1; // Number of iterations
const MAX_DURATION= '1m'; // Duration before timeout the job

const DEFAULT_HOSTNAME = 'http://localhost:8081/'; // Hostname for the application
const DEFAULT_TARGET_URL = 'https://webhook.site/2af5e206-b9fd-4f83-8e96-202a80c862b1'; // Target URL for the application

const DEFAULT_SERVICE_TOKEN = 'EoUCCpoBCgR0eXBlCg5zZXJ2aWNlX2FjY2VzcwoIdG9rZW5faWQKCmNyZWF0ZWRfYXQKD29yZ2FuaXphdGlvbl9pZBgDIgoKCAiACBIDGIEIIggKBggWEgIQASIZChcIgggSEioQid2xfYQ2RY2OBTOEk1gfKyINCgsIgwgSBiCf0bayBiIZChcIhAgSEioQ34yq-k0hTo-Uy_gyJ-7I3BIkCAASIESnww10dhfs4tOIK2JXLF4-hJFgHjraOlpgZQWvgp0DGkBJi0-X-8EFPXbkABXxUJwppzxkDNe1fS2SwH2qT3BVREXH2eLBZOOwrWQrjDx_z6zFxJluMnMVwJzhEvNg3scFIiIKINUTKiMzF8zKGEk4lDXCyTpWM4FBDoCuK-VzWvcHzeLY';
const DEFAULT_ORGANIZATION_ID = 'df8caafa-4d21-4e8f-94cb-f83227eec8dc';

export function getEnvironmentVariables() {
    const vus = __ENV.VUS || VUS;
    const iterations = __ENV.ITERATIONS || ITERATIONS;
    const maxDuration = __ENV.MAX_DURATION || MAX_DURATION;

    const hostname = __ENV.HOSTNAME || DEFAULT_HOSTNAME;
    const targetUrl = __ENV.TARGET_URL || DEFAULT_TARGET_URL;
    const serviceToken = __ENV.SERVICE_TOKEN || DEFAULT_SERVICE_TOKEN;
    const organizationId = __ENV.ORGANIZATION_ID || DEFAULT_ORGANIZATION_ID;


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