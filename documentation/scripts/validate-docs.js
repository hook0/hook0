#!/usr/bin/env node

/**
 * Documentation validation script for Hook0
 * Checks for broken internal links and code-example defects:
 *  - broken internal .md links
 *  - duplicate JSON keys in ```json examples
 *  - broken numbered-comment sequences (// 1. // 2. // 4. ...)
 *  - JS/TS functions called with a different argument count than their definition
 */

const fs = require('fs');
const path = require('path');
const glob = require('glob');

const DOCS_ROOT = path.join(__dirname, '..');
const LINK_PATTERN = /\[([^\]]+)\]\(([^)]+)\)/g;
const INTERNAL_LINK_PATTERN = /^(?!http|https|mailto|#)/;

let errors = 0;
let warnings = 0;

/**
 * Check if a file exists relative to the source file
 */
function checkFileExists(sourceFile, linkPath) {
  // Remove any hash fragments
  const cleanPath = linkPath.split('#')[0];
  if (!cleanPath) return true; // Hash-only links are OK

  const sourceDir = path.dirname(sourceFile);
  const targetPath = path.resolve(sourceDir, cleanPath);

  // Check if it's a markdown file
  if (!cleanPath.endsWith('.md')) {
    return true; // Non-markdown links are assumed OK for now
  }

  return fs.existsSync(targetPath);
}

/**
 * Split a markdown/MDX file into fenced code blocks.
 * Returns [{ lang, firstCodeLine, code }]. firstCodeLine is the 1-based line
 * number of the first line *inside* the fence (so offsets map back to the file).
 */
function extractCodeBlocks(content) {
  const lines = content.split('\n');
  const blocks = [];
  let inBlock = false;
  let fenceChar = '';
  let lang = '';
  let firstCodeLine = 0;
  let buf = [];
  for (let i = 0; i < lines.length; i++) {
    const open = lines[i].match(/^\s*(`{3,}|~{3,})(.*)$/);
    if (!inBlock && open) {
      inBlock = true;
      fenceChar = open[1][0];
      lang = open[2].trim().toLowerCase();
      firstCodeLine = i + 2; // next line, 1-based
      buf = [];
    } else if (inBlock && new RegExp(`^\\s*${fenceChar}{3,}\\s*$`).test(lines[i])) {
      blocks.push({ lang, firstCodeLine, code: buf.join('\n') });
      inBlock = false;
    } else if (inBlock) {
      buf.push(lines[i]);
    }
  }
  return blocks;
}

/**
 * Detect duplicate keys at the same object depth inside a ```json block.
 * String literals are stripped before counting braces so that braces inside
 * values (placeholders like "{APP_ID}", JSON-encoded payload strings) do not
 * skew the depth.
 */
function checkDuplicateJsonKeys(block, relativePath) {
  if (!/^(json|jsonc|json5)$/.test(block.lang)) return [];
  const out = [];
  const stack = [new Set()];
  block.code.split('\n').forEach((line, idx) => {
    const km = line.match(/^\s*"([^"]+)"\s*:/);
    if (km) {
      const top = stack[stack.length - 1];
      if (top.has(km[1])) {
        out.push(`${relativePath}:${block.firstCodeLine + idx} - duplicate JSON key "${km[1]}" in the same object`);
      } else {
        top.add(km[1]);
      }
    }
    const stripped = line.replace(/"(?:[^"\\]|\\.)*"/g, '');
    for (const ch of stripped) {
      if (ch === '{' || ch === '[') stack.push(new Set());
      else if ((ch === '}' || ch === ']') && stack.length > 1) stack.pop();
    }
  });
  return out;
}

/**
 * Flag a numbered-comment sequence (// 1. // 2. // 4. ...) that skips or
 * repeats a number. Only forward skips and repeats are flagged; a reset to a
 * lower number is treated as a new list.
 */
function checkCommentNumbering(block, relativePath) {
  const nums = [];
  block.code.split('\n').forEach((line, idx) => {
    const m = line.match(/^\s*(?:\/\/|#)\s*(\d+)\.\s/);
    if (m) nums.push({ n: parseInt(m[1], 10), line: block.firstCodeLine + idx });
  });
  if (nums.length < 3) return [];
  const out = [];
  for (let i = 1; i < nums.length; i++) {
    if (nums[i].n === nums[i - 1].n + 1) continue; // sequential
    if (nums[i].n <= nums[i - 1].n) continue; // reset = new list, ignore
    out.push(`${relativePath}:${nums[i].line} - numbered comment sequence skips from // ${nums[i - 1].n}. to // ${nums[i].n}.`);
  }
  return out;
}

/** Count top-level arguments in a call starting at s[openIdx] === '('. */
function countArgs(s, openIdx) {
  let depth = 0;
  let commas = 0;
  let sawContent = false;
  for (let i = openIdx; i < s.length; i++) {
    const ch = s[i];
    if (ch === '(' || ch === '[' || ch === '{') depth++;
    else if (ch === ')' || ch === ']' || ch === '}') {
      depth--;
      if (depth === 0) return sawContent ? commas + 1 : 0;
    } else if (ch === ',' && depth === 1) commas++;
    else if (depth === 1 && !/\s/.test(ch)) sawContent = true;
  }
  return null; // unbalanced
}

/**
 * In a JS/TS block, flag a function called with a different number of arguments
 * than it was defined with on the same page. Conservative on purpose: only
 * functions defined exactly once, with no default (`=`) or rest (`...`)
 * parameters, are checked. Method calls (obj.fn) are ignored.
 */
function checkFunctionArity(block, relativePath) {
  if (!/^(js|javascript|ts|typescript|jsx|tsx)$/.test(block.lang)) return [];
  const clean = block.code
    .replace(/\/\/[^\n]*/g, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/`(?:[^`\\]|\\.)*`/g, '``')
    .replace(/'(?:[^'\\]|\\.)*'/g, "''")
    .replace(/"(?:[^"\\]|\\.)*"/g, '""');

  const defs = {};
  const defRe = /\bfunction\s+([A-Za-z_$][\w$]*)\s*\(([^)]*)\)/g;
  let m;
  while ((m = defRe.exec(clean))) {
    const name = m[1];
    const params = m[2].trim();
    const list = params === '' ? [] : params.split(',').map((s) => s.trim());
    const optional = list.some((p) => p.includes('=') || p.startsWith('...'));
    if (defs[name]) defs[name].dup = true;
    else defs[name] = { count: list.length, optional, dup: false };
  }

  const out = [];
  for (const [name, d] of Object.entries(defs)) {
    if (d.dup || d.optional) continue;
    const callRe = new RegExp(`(?<![.\\w$])${name}\\s*\\(`, 'g');
    let cm;
    while ((cm = callRe.exec(clean))) {
      const before = clean.slice(Math.max(0, cm.index - 12), cm.index);
      if (/function\s+$/.test(before)) continue; // the definition itself
      const argc = countArgs(clean, cm.index + cm[0].length - 1);
      if (argc === null || argc === d.count) continue;
      const lineNo = block.firstCodeLine + clean.slice(0, cm.index).split('\n').length - 1;
      out.push(`${relativePath}:${lineNo} - ${name}() called with ${argc} argument(s) but defined with ${d.count}`);
    }
  }
  return out;
}

/**
 * Validate a single markdown file
 */
function validateFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  const relativePath = path.relative(DOCS_ROOT, filePath);
  let fileErrors = 0;

  // Check for broken internal links
  // Reset lastIndex to avoid skipping matches (global regex persists state)
  LINK_PATTERN.lastIndex = 0;
  let match;
  while ((match = LINK_PATTERN.exec(content)) !== null) {
    const linkText = match[1];
    const linkPath = match[2];

    if (INTERNAL_LINK_PATTERN.test(linkPath)) {
      if (!checkFileExists(filePath, linkPath)) {
        console.error(`❌ Broken link in ${relativePath}: [${linkText}](${linkPath})`);
        errors++;
        fileErrors++;
      }
    }
  }

  // Check code examples for self-inconsistencies
  const codeBlocks = extractCodeBlocks(content);
  for (const block of codeBlocks) {
    const blockErrors = [
      ...checkDuplicateJsonKeys(block, relativePath),
      ...checkCommentNumbering(block, relativePath),
      ...checkFunctionArity(block, relativePath),
    ];
    for (const err of blockErrors) {
      console.error(`❌ ${err}`);
      errors++;
      fileErrors++;
    }
  }

  // Check for proper frontmatter
  if (!content.startsWith('---') && !content.startsWith('#')) {
    console.warn(`⚠️  Missing frontmatter in ${relativePath}`);
    warnings++;
  }

  // Check for TODO comments
  if (content.includes('TODO') || content.includes('FIXME')) {
    const lines = content.split('\n');
    lines.forEach((line, index) => {
      if (line.includes('TODO') || line.includes('FIXME')) {
        console.warn(`⚠️  ${relativePath}:${index + 1} - Found TODO/FIXME comment`);
        warnings++;
      }
    });
  }

  return fileErrors === 0;
}

/**
 * Main validation function
 */
function validateDocumentation() {
  console.log('🔍 Validating Hook0 documentation...\n');

  // Find all markdown files
  const files = glob.sync('**/*.{md,mdx}', {
    cwd: DOCS_ROOT,
    absolute: true,
    ignore: ['**/node_modules/**', '**/build/**'],
  });

  console.log(`Found ${files.length} markdown files to validate\n`);

  let validFiles = 0;
  files.forEach((file) => {
    if (validateFile(file)) {
      validFiles++;
    }
  });

  // Summary
  console.log('\n' + '='.repeat(50));
  console.log('📊 Validation Summary:');
  console.log(`✅ Valid files: ${validFiles}/${files.length}`);
  console.log(`❌ Errors: ${errors}`);
  console.log(`⚠️  Warnings: ${warnings}`);

  if (errors > 0) {
    console.log('\n❌ Documentation validation failed!');
    process.exit(1);
  } else if (warnings > 0) {
    console.log('\n⚠️  Documentation validation passed with warnings');
  } else {
    console.log('\n✅ Documentation validation passed!');
  }
}

// Run validation
validateDocumentation();
