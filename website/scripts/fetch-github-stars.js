#!/usr/bin/env node
/**
 * Fetch the Hook0 GitHub stargazers count at build time -> github-stars.json.
 * Mirrors the oss-friends pattern: data.js reads the file with a try/catch
 * fallback, so a failed or slow fetch never breaks the build — the social-proof
 * bar simply falls back to a plain GitHub link with no count.
 */

const fs = require('fs');
const https = require('https');
const path = require('path');

const REPO = 'hook0/hook0';
const OUTPUT = path.join(__dirname, '..', 'github-stars.json');
const TIMEOUT_MS = 8000;
const MAX_BYTES = 1_000_000;

function skip(reason) {
  console.warn(`[github-stars] skipped (${reason}); social-proof bar will use the plain GitHub link.`);
  process.exit(0);
}

const req = https.get(
  `https://api.github.com/repos/${REPO}`,
  { headers: { 'User-Agent': 'hook0-website-build', Accept: 'application/vnd.github+json' } },
  (res) => {
    if (res.statusCode !== 200) {
      res.resume();
      return skip(`HTTP ${res.statusCode}`);
    }
    let body = '';
    res.on('data', (chunk) => {
      body += chunk;
      if (body.length > MAX_BYTES) req.destroy();
    });
    res.on('end', () => {
      try {
        const stars = JSON.parse(body).stargazers_count;
        if (!Number.isInteger(stars) || stars < 0) return skip('no valid stargazers_count');
        fs.writeFileSync(OUTPUT, `${JSON.stringify({ stars })}\n`);
        console.log(`[github-stars] wrote ${stars} stars to github-stars.json`);
      } catch (e) {
        skip(e.message);
      }
    });
  },
);

req.setTimeout(TIMEOUT_MS, () => {
  req.destroy();
  skip('timeout');
});
req.on('error', (e) => skip(e.message));
