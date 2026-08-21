import * as fs from 'fs';

import { extractApiSurface, REPORT_NAME, REPORT_PATH } from './apiSurface';

fs.writeFileSync(REPORT_PATH, extractApiSurface(), 'utf8');
console.log(`Updated ${REPORT_NAME} — review the diff before committing it.`);
