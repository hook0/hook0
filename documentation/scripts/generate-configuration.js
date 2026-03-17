#!/usr/bin/env node

/**
 * Generates configuration.md from Hook0 API /environment_variables endpoint
 *
 * Usage:
 *   node scripts/generate-configuration.js [API_URL]
 *
 * Default API_URL: http://localhost:8081/api/v1
 */

const fs = require('fs');
const path = require('path');

const API_URL = process.argv[2] || 'http://localhost:8081/api/v1';
const OUTPUT_FILE = path.join(__dirname, '..', 'reference', 'configuration.md');

async function fetchEnvVars() {
  const response = await fetch(`${API_URL}/environment_variables`);
  if (!response.ok) {
    throw new Error(`Failed to fetch environment variables: ${response.status}`);
  }
  return response.json();
}

function groupByGroup(envVars) {
  const groups = {};
  for (const envVar of envVars) {
    const group = envVar.group || 'Other';
    if (!groups[group]) {
      groups[group] = [];
    }
    groups[group].push(envVar);
  }
  return groups;
}

function generateMarkdown(envVars) {
  const grouped = groupByGroup(envVars);

  // Define preferred group order
  const groupOrder = [
    'Web Server',
    'Reverse Proxy',
    'Database',
    'Auth',
    'Email',
    'Frontend',
    'Rate Limiting',
    'Quotas',
    'Housekeeping',
    'Monitoring',
    'Hook0 Client',
    'Object Storage',
    'Pulsar',
    'Deprecated',
  ];

  // Sort groups: preferred order first, then alphabetically
  const sortedGroups = Object.keys(grouped).sort((a, b) => {
    const aIndex = groupOrder.indexOf(a);
    const bIndex = groupOrder.indexOf(b);
    if (aIndex !== -1 && bIndex !== -1) return aIndex - bIndex;
    if (aIndex !== -1) return -1;
    if (bIndex !== -1) return 1;
    return a.localeCompare(b);
  });

  let md = `# Configuration Reference

<!--
  ⚠️  AUTO-GENERATED FILE - DO NOT EDIT MANUALLY

  This file is generated from the Hook0 API /environment_variables endpoint.
  To regenerate, run: npm run generate:config
-->

Environment variables for configuring Hook0.

:::tip Source of Truth
The authoritative reference for all configuration options is running the executable with \`--help\`:

\`\`\`bash
hook0-api --help
hook0-output-worker --help
\`\`\`

This documentation may not cover all options or reflect recent changes.
:::

## API

`;

  for (const group of sortedGroups) {
    const vars = grouped[group].sort((a, b) => a.env_var.localeCompare(b.env_var));

    md += `### ${group}\n\n`;
    md += `| Variable | Description | Default | Required |\n`;
    md += `|----------|-------------|---------|----------|\n`;

    for (const v of vars) {
      const desc = (v.description || '').replace(/\|/g, '\\|').replace(/\n/g, ' ');
      const defaultVal = v.default !== null ? `\`${v.default}\`` : '-';
      const required = v.required ? '✓' : '';
      const sensitive = v.sensitive ? ' 🔒' : '';

      md += `| \`${v.env_var}\`${sensitive} | ${desc} | ${defaultVal} | ${required} |\n`;
    }

    md += '\n';
  }

  // Add output-worker section (not auto-generated)
  md += `## Output Worker

The output-worker is a separate binary with its own configuration. Run \`hook0-output-worker --help\` for the authoritative reference.

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| \`SENTRY_DSN\` | Optional Sentry DSN for error reporting | - |  |
| \`SENTRY_DEBUG\` | Enable Sentry SDK debug mode | \`false\` |  |
| \`SENTRY_SEND_DEFAULT_PII\` | Send default PII (IP addresses, cookies, etc.) to Sentry | \`false\` |  |
| \`OTLP_METRICS_ENDPOINT\` | Optional OTLP endpoint that will receive metrics | - |  |
| \`OTLP_TRACES_ENDPOINT\` | Optional OTLP endpoint that will receive traces | - |  |
| \`OTLP_AUTHORIZATION\` 🔒 | Optional value for OTLP \`Authorization\` header (for example: \`Bearer mytoken\`) | - |  |
| \`DATABASE_URL\` 🔒 | Database URL (with credentials) | - | ✓ |
| \`MAX_DB_CONNECTIONS\` | Maximum number of connections to database (for a worker with pg queue type, it should be equal to CONCURRENT) | \`5\` |  |
| \`PULSAR_BINARY_URL\` | Pulsar binary URL | - |  |
| \`PULSAR_TOKEN\` 🔒 | Pulsar token | - |  |
| \`PULSAR_TENANT\` | Pulsar tenant | - |  |
| \`PULSAR_NAMESPACE\` | Pulsar namespace | - |  |
| \`OBJECT_STORAGE_HOST\` | Host of the S3-like object storage (without https://) | - |  |
| \`OBJECT_STORAGE_FORCE_HTTP_SCHEME\` | Force endpoint scheme to be HTTP (by default it is HTTPS) | \`false\` |  |
| \`OBJECT_STORAGE_KEY_ID\` | Key ID of the S3-like object storage | - |  |
| \`OBJECT_STORAGE_KEY_SECRET\` 🔒 | Key secret of the S3-like object storage | - |  |
| \`OBJECT_STORAGE_MAX_ATTEMPTS\` | Maximum number of attempts for object storage operations | \`3\` |  |
| \`OBJECT_STORAGE_CONNECT_TIMEOUT\` | Connect timeout for object storage operations | \`3s\` |  |
| \`OBJECT_STORAGE_READ_TIMEOUT\` | Read timeout for object storage operations | \`5s\` |  |
| \`OBJECT_STORAGE_OPERATION_ATTEMPT_TIMEOUT\` | Operation attempt timeout for object storage | \`10s\` |  |
| \`OBJECT_STORAGE_OPERATION_TIMEOUT\` | Operation timeout for object storage operations | \`30s\` |  |
| \`OBJECT_STORAGE_BUCKET_NAME\` | Bucket name of the S3-like object storage | - |  |
| \`STORE_RESPONSE_BODY_AND_HEADERS_IN_OBJECT_STORAGE\` | If true, new response bodies and headers will be stored in object storage instead of database | \`false\` |  |
| \`STORE_RESPONSE_BODY_AND_HEADERS_IN_OBJECT_STORAGE_ONLY_FOR\` | A comma-separated list of applications ID whose response bodies and headers should be stored in object storage | - |  |
| \`WORKER_NAME\` | Worker name (as defined in the infrastructure.worker table) | - | ✓ |
| \`WORKER_VERSION\` | Worker version (if empty, will use version from Cargo.toml) | - |  |
| \`CONCURRENT\` | Number of request attempts to handle concurrently | \`1\` |  |
| \`MAX_FAST_RETRIES\` | Maximum number of fast retries (before doing slow retries) | \`30\` |  |
| \`MAX_SLOW_RETRIES\` | Maximum number of slow retries (before giving up) | \`30\` |  |
| \`MONITORING_HEARTBEAT_URL\` | Heartbeat URL that should be called regularly | - |  |
| \`MONITORING_HEARTBEAT_MIN_PERIOD_IN_S\` | Minimal duration (in second) to wait between sending two heartbeats | \`60\` |  |
| \`DISABLE_TARGET_IP_CHECK\` | If set to false (default), webhooks targeting non-globally-reachable IPs will fail | \`false\` |  |
| \`CONNECT_TIMEOUT\` | Timeout for establishing a connection to the target | \`5s\` |  |
| \`TIMEOUT\` | Timeout for obtaining a HTTP response from the target, including connect phase | \`15s\` |  |
| \`SIGNATURE_HEADER_NAME\` | Name of the header containing webhook's signature | \`X-Hook0-Signature\` |  |
| \`ENABLED_SIGNATURE_VERSIONS\` | A comma-separated list of enabled signature versions | \`v1\` |  |
| \`LOAD_WAITING_REQUEST_ATTEMPTS_INTO_PULSAR\` | If true, will load waiting request attempts from DB into Pulsar before starting | \`false\` |  |
| \`REQUEST_ATTEMPT_DB_COMMIT_GRACE_PERIOD\` | Grace period to wait for DB commit before dropping unfound request attempts (Pulsar workers only) | \`5s\` |  |
| \`PULSAR_CONSUMER_STATS_INTERVAL\` | Period of Pulsar consumer stats collection (set to "0s" to disable) (only for Pulsar workers) | \`15s\` |  |
| \`THROUGHPUT_LOG_INTERVAL\` | Interval between periodic throughput log lines (set to "0s" to disable) | \`60s\` |  |

`;

  md += `## Notes

- 🔒 indicates sensitive values (hidden in logs)
- Boolean values: \`true\`, \`false\` (case-insensitive)
- Durations: Use humantime format (\`1h\`, \`30m\`, \`7d\`) where supported, otherwise seconds
- Lists: Comma-separated
- URLs: Must be valid URLs with scheme
`;

  return md;
}

async function main() {
  console.log(`Fetching environment variables from ${API_URL}/environment_variables...`);

  try {
    const envVars = await fetchEnvVars();
    console.log(`Found ${envVars.length} environment variables`);

    const markdown = generateMarkdown(envVars);

    fs.writeFileSync(OUTPUT_FILE, markdown, 'utf8');
    console.log(`Generated ${OUTPUT_FILE}`);
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

main();
