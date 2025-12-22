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

`;

  for (const group of sortedGroups) {
    const vars = grouped[group].sort((a, b) => a.env_var.localeCompare(b.env_var));

    md += `## ${group}\n\n`;
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
