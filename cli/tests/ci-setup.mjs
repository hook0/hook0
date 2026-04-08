// Provisions test data for CLI E2E tests in CI.
// Outputs shell `export` statements to stdout for eval.
//
// Required env vars: DATABASE_URL, MASTER_API_KEY
// Optional env vars: API_URL (defaults to http://localhost:8080)

import pg from "pg";
const { Pool } = pg;

const DATABASE_URL = process.env.DATABASE_URL;
const MASTER_API_KEY = process.env.MASTER_API_KEY;
const API_BASE = process.env.API_URL || "http://localhost:8080";

if (!DATABASE_URL || !MASTER_API_KEY) {
  console.error("Missing DATABASE_URL or MASTER_API_KEY");
  process.exit(1);
}

const ORG_ID = "10000000-0000-0000-0000-000000000001";
const ORG_NAME = `cli-e2e-${Date.now()}`;

// Step 1: Insert organization directly into the database
const pool = new Pool({ connectionString: DATABASE_URL });
await pool.query(
  "INSERT INTO iam.organization (organization__id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
  [ORG_ID, ORG_NAME]
);
await pool.end();
console.error(`Created organization ${ORG_ID}`);

// Step 2: Create application via API using MASTER_API_KEY
const headers = {
  Authorization: `Bearer ${MASTER_API_KEY}`,
  "Content-Type": "application/json",
};

const appRes = await fetch(`${API_BASE}/api/v1/applications`, {
  method: "POST",
  headers,
  body: JSON.stringify({
    name: `cli-e2e-app-${Date.now()}`,
    organization_id: ORG_ID,
  }),
});

if (!appRes.ok) {
  console.error(
    `Failed to create application: ${appRes.status} ${await appRes.text()}`
  );
  process.exit(1);
}

const { application_id } = await appRes.json();
console.error(`Created application ${application_id}`);

// Step 3: Create application secret via API
const secretRes = await fetch(`${API_BASE}/api/v1/application_secrets`, {
  method: "POST",
  headers,
  body: JSON.stringify({ application_id }),
});

if (!secretRes.ok) {
  console.error(
    `Failed to create application secret: ${secretRes.status} ${await secretRes.text()}`
  );
  process.exit(1);
}

const { token } = await secretRes.json();
console.error(`Created application secret`);

// Step 4: Output export statements for the parent shell
console.log(`export HOOK0_SECRET="${token}"`);
console.log(`export HOOK0_APPLICATION_ID="${application_id}"`);
console.log(`export HOOK0_API_URL="${API_BASE}/api/v1"`);
