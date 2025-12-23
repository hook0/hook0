#!/usr/bin/env node

/**
 * Generates error-codes.md from Hook0 API /errors endpoint
 *
 * Usage:
 *   node scripts/generate-error-codes.js [API_URL]
 *
 * Default API_URL: https://app.hook0.com/api/v1
 */

const fs = require('fs');
const path = require('path');

const API_URL = process.argv[2] || 'https://app.hook0.com/api/v1';
const OUTPUT_FILE = path.join(__dirname, '..', 'reference', 'error-codes.md');

async function fetchErrors() {
  const response = await fetch(`${API_URL}/errors`);
  if (!response.ok) {
    throw new Error(`Failed to fetch errors: ${response.status}`);
  }
  return response.json();
}

function groupByStatus(errors) {
  const groups = {};
  for (const error of errors) {
    const status = error.status;
    if (!groups[status]) {
      groups[status] = [];
    }
    groups[status].push(error);
  }
  return groups;
}

function getStatusCategory(status) {
  if (status === 400) return { name: 'Bad Request', desc: 'Invalid request format or parameters' };
  if (status === 401) return { name: 'Unauthorized', desc: 'Authentication required' };
  if (status === 403) return { name: 'Forbidden', desc: 'Insufficient permissions or invalid credentials' };
  if (status === 404) return { name: 'Not Found', desc: 'Resource does not exist' };
  if (status === 409) return { name: 'Conflict', desc: 'Resource already exists or state conflict' };
  if (status === 410) return { name: 'Gone', desc: 'Feature disabled' };
  if (status === 422) return { name: 'Unprocessable Entity', desc: 'Validation errors' };
  if (status === 429) return { name: 'Too Many Requests', desc: 'Rate limit or quota exceeded' };
  if (status === 500) return { name: 'Internal Server Error', desc: 'Unexpected error' };
  if (status === 503) return { name: 'Service Unavailable', desc: 'Service temporarily unavailable' };
  return { name: `Status ${status}`, desc: '' };
}

function generateMarkdown(errors) {
  const grouped = groupByStatus(errors);
  const statuses = Object.keys(grouped).map(Number).sort((a, b) => a - b);

  let md = `# Error Codes Reference

<!--
  ⚠️  AUTO-GENERATED FILE - DO NOT EDIT MANUALLY

  This file is generated from the Hook0 API /errors endpoint.
  To regenerate, run: npm run generate:errors
-->

Hook0 uses RFC 7807 Problem Details for HTTP APIs format for structured error responses.

## Error Response Format

All API errors follow this structure (RFC 7807):

- **type**: URL to error documentation
- **id**: Error identifier (enum variant name)
- **title**: Short human-readable summary
- **detail**: Explanation of the error
- **status**: HTTP status code

`;

  for (const status of statuses) {
    const cat = getStatusCategory(status);
    const statusErrors = grouped[status].sort((a, b) => a.id.localeCompare(b.id));

    md += `## ${status} ${cat.name}\n\n`;

    for (const error of statusErrors) {
      const errorObj = {
        type: `https://hook0.com/documentation/errors/${error.id}`,
        id: error.id,
        title: error.title,
        detail: error.detail,
        status: error.status
      };

      md += `### ${error.id}\n\n`;
      md += `\`\`\`json\n${JSON.stringify(errorObj, null, 2)}\n\`\`\`\n\n`;
    }
  }

  md += `## Handling Errors

For implementation guidance on error handling in your client code, see [Client-side Error Handling Best Practices](/how-to-guides/client-error-handling).
`;

  return md;
}

async function main() {
  console.log(`Fetching errors from ${API_URL}/errors...`);

  try {
    const errors = await fetchErrors();
    console.log(`Found ${errors.length} error codes`);

    const markdown = generateMarkdown(errors);

    fs.writeFileSync(OUTPUT_FILE, markdown, 'utf8');
    console.log(`Generated ${OUTPUT_FILE}`);
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

main();
