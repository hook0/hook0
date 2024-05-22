const { Pool } = require('pg');

const pool = new Pool({
    user: 'root',
    host: 'localhost',
    database: 'hook0',
    password: 'root',
    port: 5432,
});

function main() {
    if (!pool) {
        throw new Error('No connection to database');
    }

    // Delete truncate iam.organization
    pool.query('DELETE FROM iam.organization CASCADE;', (err, res) => {
        if (err) {
            console.error('Error executing query', err);
            return;
        }

        // Check if done correctly
        pool.query('SELECT * FROM iam.organization;', (err, res) => {
            if (err) {
                console.error('Error executing query', err);
                return;
            }

            if (res.rows.length === 0) {
                console.log('Truncate done correctly');
            } else {
                console.error('Truncate failed. Retrying...');
                main();
            }
        });
    });
}

main();