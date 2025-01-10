const { Pool } = require('pg');
const { TZDate } = require("@date-fns/tz");

const databaseUrl = process.env.DATABASE_URL;
if (!databaseUrl) {
  throw new Error('[E2E-SETUP] Missing environment variable DATABASE_URL');
}

const date = new TZDate();
const organizationName = 'e2e-organization-' + date.toISOString();

const organizationId = process.env.ORGANIZATION_ID;
if (!organizationId) {
  throw new Error('[E2E-SETUP] Missing environment variable ORGANIZATION_ID');
}

const pool = new Pool({
  connectionString: databaseUrl,
});

pool.query(
  'INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;',
  [organizationId, organizationName, organizationId],
  (err, _res) => {
    if (err) {
      throw new Error('[E2E-SETUP] Insert organization failed. Error: ' + err);
    }

    pool.end();
  }
);
