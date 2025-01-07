const { Pool } = require('pg');

const databaseUrl = process.env.DATABASE_URL;
if (!databaseUrl) {
  throw new Error('Missing environment variable DATABASE_URL');
}

const date = new Date().toISOString().replace('Z', '+00:00');
const organizationName = 'e2e-organization-' + date;

const organizationId = process.env.ORGANIZATION_ID;
if (!organizationId) {
  throw new Error('Missing environment variable ORGANIZATION_ID');
}

const pool = new Pool({
  connectionString: databaseUrl,
});

pool.query(
  'INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING RETURNING organization__id;',
  [organizationId, organizationName, organizationId],
  (err, _res) => {
    if (err) {
      throw new Error('[E2E-SETUP] Insert organization failed. Error: ' + err);
    }

    pool.end();
  }
);
